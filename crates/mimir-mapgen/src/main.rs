use std::path::PathBuf;
use std::time::Instant;

use clap::{Parser, Subcommand};

use mimir_mapgen::biomes;
use mimir_mapgen::pipeline::{generate, validate_config, MapConfig};

/// Declarative Dungeondraft map generator
#[derive(Parser)]
#[command(name = "mimir-mapgen", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a .dungeondraft_map from a YAML config or biome preset
    Generate {
        /// Path to the YAML configuration file
        config: Option<String>,

        /// Output file path
        #[arg(short, long, default_value = "output.dungeondraft_map")]
        output: String,

        /// Random seed override (takes precedence over config file seed)
        #[arg(short, long)]
        seed: Option<u64>,

        /// Generate from a biome preset instead of a config file
        #[arg(short, long)]
        preset: Option<String>,
    },

    /// Validate a YAML config without generating
    Validate {
        /// Path to the YAML configuration file
        config: String,
    },

    /// List available biome presets
    ListPresets,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            config,
            output,
            seed,
            preset,
        } => {
            let map_config = match (config, preset) {
                (Some(_), Some(_)) => {
                    eprintln!("Error: Cannot specify both a config file and --preset");
                    std::process::exit(1);
                }
                (None, None) => {
                    eprintln!("Error: Provide a config file path or use --preset <name>");
                    std::process::exit(1);
                }
                (Some(path), None) => match load_config(&path) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Error loading config: {e}");
                        std::process::exit(1);
                    }
                },
                (None, Some(name)) => match biomes::get_preset(&name) {
                    Some(p) => p.config,
                    None => {
                        eprintln!("Unknown preset '{}'. Use 'list-presets' to see available presets.", name);
                        std::process::exit(1);
                    }
                },
            };

            // Validate
            let errors = validate_config(&map_config);
            if !errors.is_empty() {
                eprintln!("Config validation failed:");
                for e in &errors {
                    eprintln!("  - [{}] {}", e.field, e.message);
                }
                std::process::exit(1);
            }

            // Generate
            eprintln!("Generating map '{}'...", map_config.name);
            let start = Instant::now();
            let result = generate(&map_config, seed);
            let elapsed = start.elapsed();

            // Write output
            let output_path = PathBuf::from(&output);
            match result.map.to_json() {
                Ok(json) => {
                    if let Err(e) = std::fs::write(&output_path, &json) {
                        eprintln!("Error writing output: {e}");
                        std::process::exit(2);
                    }
                }
                Err(e) => {
                    eprintln!("Error serializing map: {e}");
                    std::process::exit(2);
                }
            }

            eprintln!("Done in {:.2}s", elapsed.as_secs_f64());
            eprintln!("  Objects placed: {}", result.stats.objects_placed);
            eprintln!("  Paths generated: {}", result.stats.paths_generated);
            eprintln!("  Water polygons: {}", result.stats.water_polygons);
            eprintln!("  Contour paths: {}", result.stats.contour_paths);
            eprintln!("  Output: {}", output_path.display());
        }

        Commands::Validate { config } => {
            let map_config = match load_config(&config) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error loading config: {e}");
                    std::process::exit(1);
                }
            };

            let errors = validate_config(&map_config);
            if errors.is_empty() {
                println!("Config is valid.");
            } else {
                eprintln!("Validation errors:");
                for e in &errors {
                    eprintln!("  - [{}] {}", e.field, e.message);
                }
                std::process::exit(1);
            }
        }

        Commands::ListPresets => {
            let presets = biomes::list_presets();
            println!("Available biome presets:\n");
            for p in &presets {
                println!(
                    "  {:<12} {}x{} — {}",
                    p.name, p.default_size.0, p.default_size.1, p.description
                );
            }
        }
    }
}

fn load_config(path: &str) -> Result<MapConfig, Box<dyn std::error::Error>> {
    let contents = std::fs::read_to_string(path)?;
    let config: MapConfig = serde_yaml::from_str(&contents)?;
    Ok(config)
}
