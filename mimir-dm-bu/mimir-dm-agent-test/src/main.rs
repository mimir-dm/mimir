//! Agent Test - End-to-end task completion testing for Mimir
//!
//! This binary runs tasks through the full ChatProcessor with real tool execution
//! and verifies that tasks are actually completed (database state, file changes, etc.)

mod executor;
mod prompts;
mod tasks;
mod verification;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

use executor::AgentTestExecutor;
use tasks::{AgentTask, AgentTaskSet};

#[derive(Parser)]
#[command(name = "agent-test")]
#[command(about = "End-to-end agent task completion testing")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run agent tasks with real tool execution
    Run {
        /// Path to tasks directory or single task file
        #[arg(short, long, default_value = "agent_tasks")]
        tasks: PathBuf,

        /// Provider (ollama, groq)
        #[arg(short, long, default_value = "ollama")]
        provider: String,

        /// Model name
        #[arg(short, long)]
        model: Option<String>,

        /// Ollama base URL
        #[arg(long, default_value = "http://localhost:11434")]
        ollama_url: String,

        /// API key for cloud providers
        #[arg(long)]
        api_key: Option<String>,

        /// Output directory for results
        #[arg(short, long, default_value = "agent_test_results")]
        output: PathBuf,

        /// Run only tasks matching this ID pattern
        #[arg(long)]
        filter: Option<String>,

        /// Keep test database after run (for debugging)
        #[arg(long)]
        keep_db: bool,
    },

    /// List available tasks
    List {
        /// Path to tasks directory
        #[arg(short, long, default_value = "agent_tasks")]
        tasks: PathBuf,
    },

    /// Create default task definitions
    Init {
        /// Output directory for task files
        #[arg(short, long, default_value = "agent_tasks")]
        output: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing - default to debug level for agent_test
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("agent_test=debug".parse().unwrap())
                .add_directive("mimir_dm=debug".parse().unwrap()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            tasks,
            provider,
            model,
            ollama_url,
            api_key,
            output,
            filter,
            keep_db,
        } => {
            run_tests(tasks, provider, model, ollama_url, api_key, output, filter, keep_db).await?;
        }
        Commands::List { tasks } => {
            list_tasks(tasks)?;
        }
        Commands::Init { output } => {
            init_tasks(output)?;
        }
    }

    Ok(())
}

async fn run_tests(
    tasks_path: PathBuf,
    provider: String,
    model: Option<String>,
    ollama_url: String,
    api_key: Option<String>,
    output: PathBuf,
    filter: Option<String>,
    keep_db: bool,
) -> Result<()> {
    println!("{}", "Agent Test Suite".bold().cyan());
    println!("Provider: {}", provider.green());

    // Load tasks
    let tasks = load_tasks(&tasks_path, filter.as_deref())?;
    if tasks.is_empty() {
        println!("{}", "No tasks found!".red());
        return Ok(());
    }

    println!("Loaded {} tasks", tasks.len().to_string().yellow());

    // Create executor
    let executor = AgentTestExecutor::new(provider, model, ollama_url, api_key, keep_db)
        .await
        .context("Failed to create executor")?;

    // Progress bar
    let pb = ProgressBar::new(tasks.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    // Run tasks
    let mut results = Vec::new();
    for (i, task) in tasks.iter().enumerate() {
        pb.set_position(i as u64);
        pb.set_message(format!("{}", task.id));

        let result = executor.run_task(task).await;
        results.push(result);
    }
    pb.finish_with_message("Done!");

    // Save and display results
    std::fs::create_dir_all(&output)?;
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let results_path = output.join(format!("results_{}.json", timestamp));

    let json = serde_json::to_string_pretty(&results)?;
    std::fs::write(&results_path, &json)?;

    println!("\n{}", "Results".bold().underline());
    print_results(&results);

    println!(
        "\nResults saved to: {}",
        results_path.display().to_string().blue()
    );

    Ok(())
}

fn load_tasks(path: &PathBuf, filter: Option<&str>) -> Result<Vec<AgentTask>> {
    let needs_init = !path.exists()
        || (path.is_dir()
            && std::fs::read_dir(path)?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
                .count()
                == 0);

    if needs_init {
        println!(
            "{}",
            "Tasks directory not found. Run 'agent-test init' first.".yellow()
        );
        return Ok(vec![]);
    }

    let mut all_tasks = Vec::new();

    if path.is_file() {
        let task_set = AgentTaskSet::from_file(path)?;
        all_tasks.extend(task_set.tasks);
    } else {
        let task_sets = AgentTaskSet::load_all(path)?;
        for ts in task_sets {
            all_tasks.extend(ts.tasks);
        }
    }

    // Apply filter
    if let Some(pattern) = filter {
        all_tasks.retain(|t| t.id.contains(pattern));
    }

    Ok(all_tasks)
}

fn list_tasks(tasks_path: PathBuf) -> Result<()> {
    println!("{}", "Available Tasks".bold().cyan());

    let tasks = load_tasks(&tasks_path, None)?;
    if tasks.is_empty() {
        println!("{}", "No tasks found. Run 'agent-test init' to create default tasks.".yellow());
        return Ok(());
    }

    for task in &tasks {
        let verify_count = task.verify.as_ref().map_or(0, |v| v.len());
        println!(
            "  {} - {} ({} verifications)",
            task.id.green(),
            task.description,
            verify_count
        );
    }

    println!("\nTotal: {} tasks", tasks.len());

    Ok(())
}

fn init_tasks(output: PathBuf) -> Result<()> {
    println!("{}", "Initializing default tasks".bold().cyan());

    std::fs::create_dir_all(&output)?;

    // Create default task sets
    tasks::create_default_tasks(&output)?;

    println!(
        "Created task files in: {}",
        output.display().to_string().blue()
    );

    Ok(())
}

fn print_results(results: &[executor::TaskResult]) {
    let total = results.len();
    let passed = results.iter().filter(|r| r.success).count();
    let failed = total - passed;

    for result in results {
        let status = if result.success {
            "PASS".green()
        } else {
            "FAIL".red()
        };

        println!("  [{}] {} - {}ms", status, result.task_id, result.duration_ms);

        if !result.success {
            if let Some(ref error) = result.error {
                println!("       Error: {}", error.red());
            }
            for v in &result.verification_results {
                if !v.passed {
                    println!(
                        "       Verification failed: {} - {}",
                        v.check_type,
                        v.message.as_deref().unwrap_or("no details")
                    );
                }
            }
        }
    }

    println!();
    println!(
        "Summary: {} passed, {} failed ({:.0}% pass rate)",
        passed.to_string().green(),
        failed.to_string().red(),
        (passed as f32 / total as f32) * 100.0
    );
}
