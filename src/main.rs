mod actions;
mod config;
use crate::config::{Action, FolderRule};

use clap::{App, Arg};
use log::{error, info};
use notify::{recommended_watcher, RecursiveMode, Watcher};
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use std::env;
use std::path::Path;
use std::sync::mpsc::channel;

fn main() {
    SimpleLogger::init(LevelFilter::Info, LogConfig::default()).unwrap();

    let matches = App::new("Orderly")
        .version("1.0")
        .author("Vitor Alencar <vitor.alencar@gmail.com>")
        .about("Automates file organization")
        .arg(Arg::new("init").short('i').long("init"))
        .arg(Arg::new("run").short('r').long("run"))
        .arg(Arg::new("watch").short('w').long("watch"))
        .get_matches();

    if matches.is_present("init") {
        init_orderly();
    }

    if matches.is_present("run") {
        run_orderly();
    }

    if matches.is_present("watch") {
        watch_orderly();
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

fn watch_orderly() {
    info!("Watching for changes...");
    match config::load_config("rules/example.yaml") {
        Ok(config) => {
            let (tx, rx) = channel();
            let mut watcher = recommended_watcher(move |res| tx.send(res).unwrap()).unwrap();

            for folder in config.folders {
                let path = Path::new(&folder.path);
                watcher.watch(path, RecursiveMode::Recursive).unwrap();
            }

            loop {
                match rx.recv() {
                    Ok(event) => match event {
                        Ok(notify::Event { kind, paths, .. }) => {
                            for path in paths {
                                info!("File change detected: {:?}, {:?}", path, kind);
                            }
                            run_orderly();
                        }
                        Err(e) => error!("Watch error: {:?}", e),
                    },
                    Err(e) => error!("Watch error: {:?}", e),
                }
            }
        }
        Err(e) => error!("Error loading config: {}", e),
    }
}


fn handle_conditions(folder_path: &str, rule: &FolderRule) {
    let folder = Path::new(folder_path);
    for entry in folder.read_dir().expect("read_dir call failed") {
        if let Ok(entry) = entry {
            let src_path = entry.path();

            for condition in &rule.conditions {
                match condition.condition_type.as_str() {
                    "always" => {
                        for action in &rule.actions {
                            execute_action(&src_path, action);
                        }
                    }
                    "name" => {
                        if src_path.file_name().unwrap().to_str().unwrap() == condition.value {
                            for action in &rule.actions {
                                execute_action(&src_path, action);
                            }
                        }
                    }
                    "extension" => {
                        if has_extension(&src_path, &condition.value) {
                            for action in &rule.actions {
                                execute_action(&src_path, action);
                            }
                        }
                    }
                    "name_contains" => {
                        if src_path.to_str().unwrap().contains(&condition.value) {
                            for action in &rule.actions {
                                execute_action(&src_path, action);
                            }
                        }
                    }
                    _ => error!("Unknown condition type: {}", condition.condition_type),
                }
            }
        }
    }
}

fn has_extension(file_path: &Path, extensions: &str) -> bool {
    let ext = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");
    extensions.split(',').any(|e| e.trim() == ext)
}

fn execute_action(src_path: &Path, action: &Action) {
    match action.action_type.as_str() {
        "delete" => handle_delete(src_path),
        "move" => handle_move(src_path, action),
        "copy" => handle_copy(src_path, action),
        "sort_by_date" => handle_sort_by_date(src_path, action),
        _ => error!("Unknown action type: {}", action.action_type),
    }
}

fn handle_delete(src_path: &Path) {
    info!("Deleting file: {}", src_path.display());
    match actions::delete_file(src_path.to_str().unwrap()) {
        Ok(_) => info!("File deleted successfully"),
        Err(e) => error!("Failed to delete file: {}", e),
    }
}

fn handle_move(src_path: &Path, action: &Action) {
    let dest_path = action
        .path
        .as_ref()
        .unwrap()
        .replace("~", &env::var("HOME").unwrap());
    info!("Moving file from {} to {}", src_path.display(), dest_path);
    match actions::move_file(src_path.to_str().unwrap(), &dest_path) {
        Ok(_) => info!("File moved successfully"),
        Err(e) => error!("Failed to move file: {}", e),
    }
}

fn handle_copy(src_path: &Path, action: &Action) {
    let dest_path = action
        .path
        .as_ref()
        .unwrap()
        .replace("~", &env::var("HOME").unwrap());
    info!("Copying file from {} to {}", src_path.display(), dest_path);
    match actions::copy_file(src_path.to_str().unwrap(), &dest_path) {
        Ok(_) => info!("File copied successfully"),
        Err(e) => error!("Failed to copy file: {}", e),
    }
}

fn handle_sort_by_date(src_path: &Path, action: &Action) {
    let base_path = action
        .path
        .as_ref()
        .unwrap()
        .replace("~", &env::var("HOME").unwrap());
    let pattern = action.pattern.as_ref().unwrap();
    info!(
        "Sorting file by date from {} to {}",
        src_path.display(),
        base_path
    );
    match actions::sort_file_by_date(src_path.to_str().unwrap(), &base_path, pattern) {
        Ok(_) => info!("File sorted by date successfully"),
        Err(e) => error!("Failed to sort file by date: {}", e),
    }
}
