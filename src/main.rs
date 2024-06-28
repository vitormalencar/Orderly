use std::{fs, path::PathBuf};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(name = "orderly")]
#[clap(about = "A CLI tool for organizing files", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Orderly in a directory
    Init,
    /// Run Orderly to apply rules
    Run,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            println!("Initializing Orderly...");
            initialize_orderly();
        }
        Commands::Run => {
            println!("Running Orderly...");
            // Add your run logic here
        }
    }
}

fn initialize_orderly() {
    //  just create the directory rules inside the current path
    let config_dir = PathBuf::from("./");
    let rules_dir = config_dir.join("rules");

    if let Err(e) = fs::create_dir_all(&rules_dir) {
        eprintln!("Failed to create directories: {}", e);
        return;
    }

    let example_rule = r#"
  name: Example Rule
  description: An example rule for organizing files
  actions:
    - name: Move to Trash
      path: ~/.Trash
      condition:
        type: file
        size: 100MB
      action:
        type: move
        path: ~/.Trash
  "#;

    let example_rule_path = rules_dir.join("example.yaml");

    if let Err(e) = fs::write(&example_rule_path, example_rule) {
        eprintln!("Failed to write example rule: {}", e);
        return;
    }

    println!("Orderly initialized successfully.");
}
