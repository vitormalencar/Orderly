mod actions;
mod config;
use crate::config::{Action, FolderRule};

use clap::{App, Arg};
use log::{error, info};
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use std::env;

fn main() {
    SimpleLogger::init(LevelFilter::Info, LogConfig::default()).unwrap();

    let matches = App::new("Orderly")
        .version("1.0")
        .author("Vitor Alencar <vitor.alencar@gmail.com>")
        .about("Automates file organization")
        .arg(Arg::new("init").short('i').long("init"))
        .arg(Arg::new("run").short('r').long("run"))
        .get_matches();

    if matches.is_present("init") {
        init_orderly();
    }

    if matches.is_present("run") {
        run_orderly();
    }
}

fn init_orderly() {
    info!("Initializing Orderly...");
    match config::create_example_rule() {
        Ok(_) => info!("Example rule created successfully"),
        Err(e) => error!("Failed to create example rule: {}", e),
    }
}

fn run_orderly() {
    info!("Running Orderly...");
    match config::load_config("rules/example.yaml") {
        Ok(config) => {
            info!("Config loaded: {:#?}", config);
            for folder in config.folders {
                for rule in &folder.rules {
                    handle_conditions(&folder.path, rule);
                }
            }
        }
        Err(e) => error!("Error loading config: {}", e),
    }
}

pub fn handle_conditions(folder_path: &str, rule: &FolderRule) {
    for condition in &rule.conditions {
        if condition.condition_type == "name" {
            let src_path = format!("{}/{}", folder_path, condition.value);
            for action in &rule.actions {
                match action.action_type.as_str() {
                    "delete" => handle_delete(&src_path),
                    "move" => handle_move(&src_path, action),
                    "copy" => handle_copy(&src_path, action),
                    _ => error!("Unknown action type: {}", action.action_type),
                }
            }
        }
    }
}

fn handle_delete(src_path: &str) {
    info!("Deleting file: {}", src_path);
    match actions::delete_file(src_path) {
        Ok(_) => info!("File deleted successfully"),
        Err(e) => error!("Failed to delete file: {}", e),
    }
}

fn handle_move(src_path: &str, action: &Action) {
    let dest_path = action
        .path
        .as_ref()
        .unwrap()
        .replace("~", &env::var("HOME").unwrap());
    info!("Moving file from {} to {}", src_path, dest_path);
    match actions::move_file(src_path, &dest_path) {
        Ok(_) => info!("File moved successfully"),
        Err(e) => error!("Failed to move file: {}", e),
    }
}

fn handle_copy(src_path: &str, action: &Action) {
    let dest_path = action
        .path
        .as_ref()
        .unwrap()
        .replace("~", &env::var("HOME").unwrap());
    info!("Copying file from {} to {}", src_path, dest_path);
    match actions::copy_file(src_path, &dest_path) {
        Ok(_) => info!("File copied successfully"),
        Err(e) => error!("Failed to copy file: {}", e),
    }
}
