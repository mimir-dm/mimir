use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

use mimir_llm_eval::tasks::{Category, EvalConfig, EvalTask, ModelSpec, TaskSet};
use mimir_llm_eval::{EvalRunner, ReportGenerator};

#[derive(Parser)]
#[command(name = "llm-eval")]
#[command(about = "LLM Model Evaluation Suite for D&D Assistant")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run evaluation tasks against a model
    Run {
        /// Provider (ollama, groq)
        #[arg(short, long, default_value = "ollama")]
        provider: String,

        /// Model name/ID
        #[arg(short, long)]
        model: String,

        /// Task category to run (tool_calling, generation, reasoning, edge_cases)
        #[arg(short, long)]
        category: Option<String>,

        /// Path to tasks directory
        #[arg(long, default_value = "tasks")]
        tasks_dir: PathBuf,

        /// Output directory for results
        #[arg(short, long, default_value = "results")]
        output: PathBuf,

        /// API key for cloud providers (or use env var)
        #[arg(long)]
        api_key: Option<String>,

        /// Ollama base URL
        #[arg(long, default_value = "http://localhost:11434")]
        ollama_url: String,
    },

    /// Compare multiple models
    Compare {
        /// Config file with model specifications
        #[arg(short, long)]
        config: PathBuf,

        /// Task category to run
        #[arg(long)]
        category: Option<String>,

        /// Path to tasks directory
        #[arg(long, default_value = "tasks")]
        tasks_dir: PathBuf,

        /// Output directory for results
        #[arg(short, long, default_value = "results")]
        output: PathBuf,
    },

    /// Generate report from existing results
    Report {
        /// Path to results JSON file or directory
        #[arg(short, long)]
        input: PathBuf,

        /// Output markdown file
        #[arg(short, long)]
        output: PathBuf,

        /// Report title
        #[arg(long)]
        title: Option<String>,
    },

    /// List available tasks
    ListTasks {
        /// Path to tasks directory
        #[arg(long, default_value = "tasks")]
        tasks_dir: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("mimir_llm_eval=info".parse().unwrap()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            provider,
            model,
            category,
            tasks_dir,
            output,
            api_key,
            ollama_url,
        } => {
            run_evaluation(provider, model, category, tasks_dir, output, api_key, ollama_url)
                .await?;
        }
        Commands::Compare {
            config,
            category,
            tasks_dir,
            output,
        } => {
            run_comparison(config, category, tasks_dir, output).await?;
        }
        Commands::Report {
            input,
            output,
            title,
        } => {
            generate_report(input, output, title)?;
        }
        Commands::ListTasks { tasks_dir } => {
            list_tasks(tasks_dir)?;
        }
    }

    Ok(())
}

async fn run_evaluation(
    provider: String,
    model: String,
    category: Option<String>,
    tasks_dir: PathBuf,
    output: PathBuf,
    api_key: Option<String>,
    ollama_url: String,
) -> Result<()> {
    println!("{}", "LLM Evaluation Suite".bold().cyan());
    println!("Provider: {}, Model: {}", provider.green(), model.green());

    // Build model spec
    let mut config = HashMap::new();
    if provider == "ollama" {
        config.insert("base_url".to_string(), ollama_url);
    }
    if let Some(key) = api_key {
        config.insert("api_key".to_string(), key);
    }

    let spec = ModelSpec {
        provider: provider.clone(),
        model: model.clone(),
        name: None,
        config,
    };

    // Load tasks
    let tasks = load_tasks(&tasks_dir, category.as_deref())?;
    if tasks.is_empty() {
        println!("{}", "No tasks found!".red());
        return Ok(());
    }

    println!("Loaded {} tasks", tasks.len().to_string().yellow());

    // Create runner
    let runner = EvalRunner::new();

    // Progress bar
    let pb = ProgressBar::new(tasks.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    // Run evaluation
    let callback = |current: usize, _total: usize, task_id: &str| {
        pb.set_position(current as u64);
        pb.set_message(task_id.to_string());
    };

    let results = runner.run_all_tasks(&tasks, &spec, Some(&callback)).await;

    pb.finish_with_message("Done!");

    // Save results
    std::fs::create_dir_all(&output)?;
    let model_key = format!("{}_{}", provider, model.replace([':', '/'], "_"));
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");

    let json_path = output.join(format!("{}_{}.json", model_key, timestamp));
    let md_path = output.join(format!("{}_{}.md", model_key, timestamp));

    let mut all_results = HashMap::new();
    all_results.insert(format!("{}:{}", provider, model), results);

    ReportGenerator::save_results(&all_results, &json_path)?;
    ReportGenerator::save_markdown(&all_results, &md_path, None)?;

    println!("\n{}", "Results saved:".bold());
    println!("  JSON: {}", json_path.display().to_string().blue());
    println!("  Markdown: {}", md_path.display().to_string().blue());

    // Print summary
    print_summary(&all_results);

    Ok(())
}

async fn run_comparison(
    config_path: PathBuf,
    category: Option<String>,
    tasks_dir: PathBuf,
    output: PathBuf,
) -> Result<()> {
    println!("{}", "LLM Model Comparison".bold().cyan());

    // Load config
    let config = EvalConfig::from_file(&config_path)
        .context("Failed to load config file")?;

    println!("Comparing {} models", config.models.len().to_string().yellow());

    // Load tasks
    let tasks = load_tasks(&tasks_dir, category.as_deref())?;
    if tasks.is_empty() {
        println!("{}", "No tasks found!".red());
        return Ok(());
    }

    println!("Loaded {} tasks", tasks.len().to_string().yellow());

    // Create runner
    let runner = EvalRunner::new();

    // Run comparison
    let total_tasks = tasks.len() * config.models.len();
    let pb = ProgressBar::new(total_tasks as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    use std::sync::atomic::{AtomicU64, Ordering};
    let completed = AtomicU64::new(0);
    let callback = |model: &str, _current: usize, _total: usize, task_id: &str| {
        let c = completed.fetch_add(1, Ordering::Relaxed) + 1;
        pb.set_position(c);
        pb.set_message(format!("{}: {}", model, task_id));
    };

    let results = runner
        .run_comparison(&tasks, &config.models, Some(&callback))
        .await;

    pb.finish_with_message("Done!");

    // Save results
    std::fs::create_dir_all(&output)?;
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");

    let json_path = output.join(format!("comparison_{}.json", timestamp));
    let md_path = output.join(format!("comparison_{}.md", timestamp));

    ReportGenerator::save_results(&results, &json_path)?;
    ReportGenerator::save_markdown(&results, &md_path, Some("Model Comparison Report"))?;

    println!("\n{}", "Results saved:".bold());
    println!("  JSON: {}", json_path.display().to_string().blue());
    println!("  Markdown: {}", md_path.display().to_string().blue());

    print_summary(&results);

    Ok(())
}

fn generate_report(input: PathBuf, output: PathBuf, title: Option<String>) -> Result<()> {
    println!("{}", "Generating Report".bold().cyan());

    let results = ReportGenerator::load_results(&input)
        .context("Failed to load results")?;

    ReportGenerator::save_markdown(&results, &output, title.as_deref())?;

    println!("Report saved to: {}", output.display().to_string().blue());

    Ok(())
}

fn list_tasks(tasks_dir: PathBuf) -> Result<()> {
    println!("{}", "Available Tasks".bold().cyan());

    let needs_tasks = !tasks_dir.exists()
        || (tasks_dir.exists()
            && std::fs::read_dir(&tasks_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
                .count() == 0);

    if needs_tasks {
        println!("{}", "Tasks directory not found or empty. Creating default tasks...".yellow());
        create_default_tasks(&tasks_dir)?;
    }

    let task_sets = TaskSet::load_all(&tasks_dir)?;

    for task_set in task_sets {
        println!("\n{}: {}", task_set.name.bold(), task_set.description);
        for task in &task_set.tasks {
            println!(
                "  {} [{}] - {}",
                task.id.green(),
                task.category.as_str().blue(),
                task.description
            );
        }
    }

    Ok(())
}

fn load_tasks(tasks_dir: &PathBuf, category: Option<&str>) -> Result<Vec<EvalTask>> {
    let needs_tasks = !tasks_dir.exists()
        || std::fs::read_dir(tasks_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
            .count() == 0;

    if needs_tasks {
        println!("{}", "Tasks directory not found or empty. Creating default tasks...".yellow());
        create_default_tasks(tasks_dir)?;
    }

    let task_sets = TaskSet::load_all(tasks_dir)?;
    let mut all_tasks: Vec<EvalTask> = task_sets.into_iter().flat_map(|ts| ts.tasks).collect();

    // Filter by category if specified
    if let Some(cat_str) = category {
        let filter_cat = match cat_str {
            "tool_calling" => Category::ToolCalling,
            "generation" => Category::Generation,
            "reasoning" => Category::Reasoning,
            "edge_cases" => Category::EdgeCases,
            _ => anyhow::bail!("Unknown category: {}", cat_str),
        };
        all_tasks.retain(|t| t.category == filter_cat);
    }

    Ok(all_tasks)
}

fn create_default_tasks(tasks_dir: &PathBuf) -> Result<()> {
    std::fs::create_dir_all(tasks_dir)?;

    // Tool calling tasks
    let tool_calling = TaskSet {
        name: "Tool Calling".to_string(),
        description: "Objective tasks testing tool calling accuracy".to_string(),
        tasks: vec![
            EvalTask {
                id: "spell_lookup".to_string(),
                category: Category::ToolCalling,
                prompt: "What does the Fireball spell do?".to_string(),
                expected_tools: vec!["search_spells".to_string()],
                evaluation_criteria: vec![],
                description: "Tests spell lookup tool usage".to_string(),
                expect_no_tools: false,
            },
            EvalTask {
                id: "monster_stats".to_string(),
                category: Category::ToolCalling,
                prompt: "What are the stats for a Goblin?".to_string(),
                expected_tools: vec!["get_monster_details".to_string()],
                evaluation_criteria: vec![],
                description: "Tests monster lookup tool usage".to_string(),
                expect_no_tools: false,
            },
            EvalTask {
                id: "player_list".to_string(),
                category: Category::ToolCalling,
                prompt: "Who are the players in my campaign?".to_string(),
                expected_tools: vec!["list_players".to_string()],
                evaluation_criteria: vec![],
                description: "Tests player listing tool usage".to_string(),
                expect_no_tools: false,
            },
            EvalTask {
                id: "character_info".to_string(),
                category: Category::ToolCalling,
                prompt: "Show me Thorin's character details and inventory.".to_string(),
                expected_tools: vec!["get_character_details".to_string()],
                evaluation_criteria: vec![],
                description: "Tests character detail lookup".to_string(),
                expect_no_tools: false,
            },
        ],
    };

    // Content generation tasks
    let generation = TaskSet {
        name: "Content Generation".to_string(),
        description: "Subjective tasks testing creative content quality".to_string(),
        tasks: vec![
            EvalTask {
                id: "npc_creation".to_string(),
                category: Category::Generation,
                prompt: "Create a mysterious merchant NPC who deals in rare magical items. Include their appearance, personality, and a secret they're hiding.".to_string(),
                expected_tools: vec![],
                evaluation_criteria: vec![
                    "Creativity and originality".to_string(),
                    "D&D appropriateness".to_string(),
                    "Useful details for gameplay".to_string(),
                    "Compelling secret/hook".to_string(),
                ],
                description: "Tests NPC creation quality".to_string(),
                expect_no_tools: false,
            },
            EvalTask {
                id: "encounter_design".to_string(),
                category: Category::Generation,
                prompt: "Design a forest ambush encounter for a level 3 party of 4 players. Include enemy composition, tactics, and terrain features.".to_string(),
                expected_tools: vec![],
                evaluation_criteria: vec![
                    "Appropriate challenge level".to_string(),
                    "Tactical complexity".to_string(),
                    "Interesting terrain use".to_string(),
                    "Narrative potential".to_string(),
                ],
                description: "Tests encounter design quality".to_string(),
                expect_no_tools: false,
            },
            EvalTask {
                id: "location_description".to_string(),
                category: Category::Generation,
                prompt: "Describe a haunted tavern called 'The Weeping Willow' that the party just entered.".to_string(),
                expected_tools: vec![],
                evaluation_criteria: vec![
                    "Atmospheric description".to_string(),
                    "Sensory details".to_string(),
                    "Adventure hooks".to_string(),
                    "Usable in play".to_string(),
                ],
                description: "Tests location description quality".to_string(),
                expect_no_tools: false,
            },
        ],
    };

    // Reasoning tasks
    let reasoning = TaskSet {
        name: "Reasoning".to_string(),
        description: "Tasks testing rules knowledge and tactical reasoning".to_string(),
        tasks: vec![
            EvalTask {
                id: "combat_advice".to_string(),
                category: Category::Reasoning,
                prompt: "My rogue is adjacent to an enemy with the Sentinel feat. I want to use Cunning Action to disengage and move away. Will this work? What are my options?".to_string(),
                expected_tools: vec![],
                evaluation_criteria: vec![
                    "Rules accuracy".to_string(),
                    "Tactical insight".to_string(),
                    "Clear explanation".to_string(),
                ],
                description: "Tests combat rules reasoning".to_string(),
                expect_no_tools: false,
            },
            EvalTask {
                id: "build_advice".to_string(),
                category: Category::Reasoning,
                prompt: "I'm building a Paladin focused on mounted combat. What feats and fighting style should I consider?".to_string(),
                expected_tools: vec![],
                evaluation_criteria: vec![
                    "Build synergy".to_string(),
                    "Rules accuracy".to_string(),
                    "Practical advice".to_string(),
                ],
                description: "Tests character build knowledge".to_string(),
                expect_no_tools: false,
            },
            EvalTask {
                id: "rule_clarification".to_string(),
                category: Category::Reasoning,
                prompt: "How does concentration work in D&D 5e? What happens if I take damage while concentrating?".to_string(),
                expected_tools: vec![],
                evaluation_criteria: vec![
                    "Rules accuracy".to_string(),
                    "Completeness".to_string(),
                    "Clear explanation".to_string(),
                ],
                description: "Tests rules explanation quality".to_string(),
                expect_no_tools: false,
            },
        ],
    };

    // Edge cases
    let edge_cases = TaskSet {
        name: "Edge Cases".to_string(),
        description: "Tasks testing model behavior at boundaries".to_string(),
        tasks: vec![
            EvalTask {
                id: "simple_greeting".to_string(),
                category: Category::EdgeCases,
                prompt: "Hello!".to_string(),
                expected_tools: vec![],
                evaluation_criteria: vec![],
                description: "Should NOT call any tools".to_string(),
                expect_no_tools: true,
            },
            EvalTask {
                id: "ambiguous_request".to_string(),
                category: Category::EdgeCases,
                prompt: "Help me with my character.".to_string(),
                expected_tools: vec![],
                evaluation_criteria: vec![
                    "Asks clarifying questions".to_string(),
                    "Doesn't assume too much".to_string(),
                ],
                description: "Tests handling of ambiguous requests".to_string(),
                expect_no_tools: false,
            },
            EvalTask {
                id: "impossible_request".to_string(),
                category: Category::EdgeCases,
                prompt: "Roll a d20 for me.".to_string(),
                expected_tools: vec![],
                evaluation_criteria: vec![
                    "Explains limitation".to_string(),
                    "Offers alternative".to_string(),
                ],
                description: "Tests handling of impossible requests".to_string(),
                expect_no_tools: false,
            },
        ],
    };

    // Save task sets
    let tool_path = tasks_dir.join("tool_calling.json");
    let gen_path = tasks_dir.join("generation.json");
    let reason_path = tasks_dir.join("reasoning.json");
    let edge_path = tasks_dir.join("edge_cases.json");

    std::fs::write(&tool_path, serde_json::to_string_pretty(&tool_calling)?)?;
    std::fs::write(&gen_path, serde_json::to_string_pretty(&generation)?)?;
    std::fs::write(&reason_path, serde_json::to_string_pretty(&reasoning)?)?;
    std::fs::write(&edge_path, serde_json::to_string_pretty(&edge_cases)?)?;

    println!("Created default task files in {}", tasks_dir.display());

    Ok(())
}

fn print_summary(results: &HashMap<String, Vec<mimir_llm_eval::EvalResult>>) {
    println!("\n{}", "Summary".bold().underline());

    for (model, model_results) in results {
        println!("\n{}", model.bold());

        let total = model_results.len();
        let successes = model_results.iter().filter(|r| r.success).count();
        let avg_time: u64 = if total > 0 {
            model_results.iter().map(|r| r.response_time_ms).sum::<u64>() / total as u64
        } else {
            0
        };

        // Tool calling accuracy
        let tool_results: Vec<_> = model_results
            .iter()
            .filter(|r| r.category == Category::ToolCalling)
            .collect();
        if !tool_results.is_empty() {
            let avg_accuracy: f32 = tool_results
                .iter()
                .filter_map(|r| r.tool_accuracy)
                .sum::<f32>()
                / tool_results.len() as f32;
            println!(
                "  Tool Calling: {:.0}% accuracy",
                (avg_accuracy * 100.0).to_string().green()
            );
        }

        println!(
            "  Success Rate: {}/{} ({:.0}%)",
            successes,
            total,
            (successes as f32 / total as f32 * 100.0)
        );
        println!("  Avg Response Time: {}ms", avg_time);
    }
}
