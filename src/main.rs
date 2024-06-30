mod actions;
mod config;

use clap::{App, Arg};
use log::{error, info};
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};

fn main() {
    SimpleLogger::init(LevelFilter::Info, LogConfig::default()).unwrap();

    let matches = App::new("Orderly")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("Automates file organization")
        .arg(Arg::new("init").short('i').long("init"))
        .arg(Arg::new("run").short('r').long("run"))
        .get_matches();

    if matches.is_present("init") {
        info!("Initializing Orderly...");
        config::create_example_rule().expect("Failed to create example rule");
    }

    if matches.is_present("run") {
        info!("Running Orderly...");
        match config::load_config("rules/example.yaml") {
            Ok(config) => {
                info!("Config loaded: {:#?}", config);
                for folder in config.folders {
                    for rule in &folder.rules {
                        for condition in &rule.conditions {
                            if condition.condition_type == "name" {
                                let src_path = format!("{}/{}", folder.path, condition.value);
                                if condition.value == "delete_me.png" {
                                    for action in &rule.actions {
                                        if action.action_type == "delete" {
                                            info!("Deleting file: {}", src_path);
                                            match actions::delete_file(&src_path) {
                                                Ok(_) => info!("File deleted successfully"),
                                                Err(e) => error!("Failed to delete file: {}", e),
                                            }
                                        }
                                    }
                                } else if condition.value == "move_me.png" {
                                    for action in &rule.actions {
                                        if action.action_type == "move" {
                                            let dest_path = action
                                                .path
                                                .as_ref()
                                                .unwrap()
                                                .replace("~", &std::env::var("HOME").unwrap());
                                            info!("Moving file from {} to {}", src_path, dest_path);
                                            match actions::move_file(&src_path, &dest_path) {
                                                Ok(_) => info!("File moved successfully"),
                                                Err(e) => error!("Failed to move file: {}", e),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => error!("Error loading config: {}", e),
        }
    }
}
