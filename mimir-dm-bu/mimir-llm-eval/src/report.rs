use std::collections::HashMap;
use std::path::Path;

use crate::tasks::{Category, EvalResult};

/// Report generator for evaluation results
pub struct ReportGenerator;

impl ReportGenerator {
    /// Generate a markdown report from evaluation results
    pub fn generate_markdown(
        results: &HashMap<String, Vec<EvalResult>>,
        title: Option<&str>,
    ) -> String {
        let mut report = String::new();
        let date = chrono::Utc::now().format("%Y-%m-%d");

        report.push_str(&format!(
            "# {} - {}\n\n",
            title.unwrap_or("LLM Evaluation Report"),
            date
        ));

        // Summary section
        report.push_str("## Summary\n\n");
        report.push_str("| Model | Tasks Run | Successes | Failures | Avg Response Time |\n");
        report.push_str("|-------|-----------|-----------|----------|-------------------|\n");

        for (model, model_results) in results {
            let total = model_results.len();
            let successes = model_results.iter().filter(|r| r.success).count();
            let failures = total - successes;
            let avg_time: u64 = if total > 0 {
                model_results.iter().map(|r| r.response_time_ms).sum::<u64>() / total as u64
            } else {
                0
            };

            report.push_str(&format!(
                "| {} | {} | {} | {} | {}ms |\n",
                model, total, successes, failures, avg_time
            ));
        }
        report.push('\n');

        // Tool Calling Accuracy section
        report.push_str("## Tool Calling Accuracy\n\n");
        let tool_results = Self::filter_by_category(results, Category::ToolCalling);
        if !tool_results.is_empty() {
            Self::generate_tool_calling_table(&mut report, &tool_results);
        } else {
            report.push_str("*No tool calling tasks were evaluated.*\n\n");
        }

        // Content Quality section
        report.push_str("## Content Quality\n\n");
        let gen_results = Self::filter_by_category(results, Category::Generation);
        if !gen_results.is_empty() {
            Self::generate_quality_table(&mut report, &gen_results);
        } else {
            report.push_str("*No content generation tasks were evaluated.*\n\n");
        }

        // Reasoning section
        report.push_str("## Reasoning & Planning\n\n");
        let reason_results = Self::filter_by_category(results, Category::Reasoning);
        if !reason_results.is_empty() {
            Self::generate_quality_table(&mut report, &reason_results);
        } else {
            report.push_str("*No reasoning tasks were evaluated.*\n\n");
        }

        // Edge Cases section
        report.push_str("## Edge Cases\n\n");
        let edge_results = Self::filter_by_category(results, Category::EdgeCases);
        if !edge_results.is_empty() {
            Self::generate_edge_case_table(&mut report, &edge_results);
        } else {
            report.push_str("*No edge case tasks were evaluated.*\n\n");
        }

        // Detailed Results section
        report.push_str("## Detailed Results\n\n");
        for (model, model_results) in results {
            report.push_str(&format!("### {}\n\n", model));
            for result in model_results {
                report.push_str(&format!("#### Task: {}\n\n", result.task_id));
                report.push_str(&format!("- **Category**: {}\n", result.category));
                report.push_str(&format!("- **Success**: {}\n", result.success));
                report.push_str(&format!("- **Response Time**: {}ms\n", result.response_time_ms));

                if let Some(accuracy) = result.tool_accuracy {
                    report.push_str(&format!("- **Tool Accuracy**: {:.0}%\n", accuracy * 100.0));
                }
                if let Some(score) = result.quality_score {
                    report.push_str(&format!("- **Quality Score**: {}/5\n", score));
                }
                if !result.tools_called.is_empty() {
                    report.push_str("- **Tools Called**: ");
                    let tool_names: Vec<&str> =
                        result.tools_called.iter().map(|t| t.name.as_str()).collect();
                    report.push_str(&tool_names.join(", "));
                    report.push('\n');
                }
                if let Some(ref err) = result.error {
                    report.push_str(&format!("- **Error**: {}\n", err));
                }
                report.push_str("\n**Prompt:**\n```\n");
                report.push_str(&result.prompt);
                report.push_str("\n```\n\n**Response:**\n```\n");
                // Truncate long responses
                let response = if result.response.len() > 500 {
                    format!("{}...[truncated]", &result.response[..500])
                } else {
                    result.response.clone()
                };
                report.push_str(&response);
                report.push_str("\n```\n\n");
            }
        }

        report
    }

    /// Filter results by category
    fn filter_by_category(
        results: &HashMap<String, Vec<EvalResult>>,
        category: Category,
    ) -> HashMap<String, Vec<&EvalResult>> {
        let mut filtered = HashMap::new();
        for (model, model_results) in results {
            let cat_results: Vec<&EvalResult> = model_results
                .iter()
                .filter(|r| r.category == category)
                .collect();
            if !cat_results.is_empty() {
                filtered.insert(model.clone(), cat_results);
            }
        }
        filtered
    }

    /// Generate tool calling accuracy table
    fn generate_tool_calling_table(report: &mut String, results: &HashMap<String, Vec<&EvalResult>>) {
        // Get all unique task IDs
        let mut task_ids: Vec<String> = results
            .values()
            .flat_map(|v| v.iter().map(|r| r.task_id.clone()))
            .collect();
        task_ids.sort();
        task_ids.dedup();

        // Header
        report.push_str("| Model |");
        for task_id in &task_ids {
            report.push_str(&format!(" {} |", task_id));
        }
        report.push_str(" Overall |\n");

        report.push_str("|-------|");
        for _ in &task_ids {
            report.push_str("------|");
        }
        report.push_str("---------|\n");

        // Data rows
        for (model, model_results) in results {
            report.push_str(&format!("| {} |", model));

            let mut total_accuracy = 0.0;
            let mut count = 0;

            for task_id in &task_ids {
                if let Some(result) = model_results.iter().find(|r| &r.task_id == task_id) {
                    if let Some(accuracy) = result.tool_accuracy {
                        report.push_str(&format!(" {:.0}% |", accuracy * 100.0));
                        total_accuracy += accuracy;
                        count += 1;
                    } else {
                        report.push_str(" - |");
                    }
                } else {
                    report.push_str(" - |");
                }
            }

            let overall = if count > 0 {
                total_accuracy / count as f32
            } else {
                0.0
            };
            report.push_str(&format!(" {:.0}% |\n", overall * 100.0));
        }
        report.push('\n');
    }

    /// Generate quality score table for subjective tasks
    fn generate_quality_table(report: &mut String, results: &HashMap<String, Vec<&EvalResult>>) {
        let mut task_ids: Vec<String> = results
            .values()
            .flat_map(|v| v.iter().map(|r| r.task_id.clone()))
            .collect();
        task_ids.sort();
        task_ids.dedup();

        report.push_str("| Model |");
        for task_id in &task_ids {
            report.push_str(&format!(" {} |", task_id));
        }
        report.push_str(" Avg |\n");

        report.push_str("|-------|");
        for _ in &task_ids {
            report.push_str("------|");
        }
        report.push_str("-----|\n");

        for (model, model_results) in results {
            report.push_str(&format!("| {} |", model));

            let mut total_score = 0u32;
            let mut count = 0u32;

            for task_id in &task_ids {
                if let Some(result) = model_results.iter().find(|r| &r.task_id == task_id) {
                    if let Some(score) = result.quality_score {
                        report.push_str(&format!(" {}/5 |", score));
                        total_score += score as u32;
                        count += 1;
                    } else {
                        report.push_str(" - |");
                    }
                } else {
                    report.push_str(" - |");
                }
            }

            let avg = if count > 0 {
                total_score as f32 / count as f32
            } else {
                0.0
            };
            report.push_str(&format!(" {:.1} |\n", avg));
        }
        report.push('\n');
    }

    /// Generate edge case results table
    fn generate_edge_case_table(report: &mut String, results: &HashMap<String, Vec<&EvalResult>>) {
        let mut task_ids: Vec<String> = results
            .values()
            .flat_map(|v| v.iter().map(|r| r.task_id.clone()))
            .collect();
        task_ids.sort();
        task_ids.dedup();

        report.push_str("| Model |");
        for task_id in &task_ids {
            report.push_str(&format!(" {} |", task_id));
        }
        report.push('\n');

        report.push_str("|-------|");
        for _ in &task_ids {
            report.push_str("------|");
        }
        report.push('\n');

        for (model, model_results) in results {
            report.push_str(&format!("| {} |", model));
            for task_id in &task_ids {
                if let Some(result) = model_results.iter().find(|r| &r.task_id == task_id) {
                    let status = if result.success { "Pass" } else { "Fail" };
                    report.push_str(&format!(" {} |", status));
                } else {
                    report.push_str(" - |");
                }
            }
            report.push('\n');
        }
        report.push('\n');
    }

    /// Save results to JSON file
    pub fn save_results(
        results: &HashMap<String, Vec<EvalResult>>,
        path: &Path,
    ) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(results)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load results from JSON file
    pub fn load_results(path: &Path) -> anyhow::Result<HashMap<String, Vec<EvalResult>>> {
        let content = std::fs::read_to_string(path)?;
        let results = serde_json::from_str(&content)?;
        Ok(results)
    }

    /// Save markdown report to file
    pub fn save_markdown(
        results: &HashMap<String, Vec<EvalResult>>,
        path: &Path,
        title: Option<&str>,
    ) -> anyhow::Result<()> {
        let report = Self::generate_markdown(results, title);
        std::fs::write(path, report)?;
        Ok(())
    }
}
