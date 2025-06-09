mod actions;
mod conditions;
mod config;
use crate::conditions::create_condition;
use crate::config::{Action, FolderRule};

use clap::{App, Arg};
use log::{error, info, warn};
use notify::{recommended_watcher, RecursiveMode, Watcher};
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::mpsc::channel;

const MAX_MOVEMENTS: usize = 10;

#[cfg(target_os = "macos")]
static HOME: &str = env!("HOME");
#[cfg(target_os = "windows")]
static HOME: &str = env!("USERPROFILE");

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
    if let Err(e) = config::create_example_rule() {
        error!("Failed to create example rule: {}", e);
    }
}

fn run_orderly() {
    info!("Running Orderly...");
    let mut processed_files = HashSet::new();
    let mut file_movements = HashMap::new();
    let mut ignored_rules = HashSet::new();

    match config::load_config("rules/example.yaml") {
        Ok(config) => {
            // info!("Config loaded: {:#?}", config);
            for folder in config.folders {
                for rule in &folder.rules {
                    if !ignored_rules.contains(&rule.name) {
                        if let Err(e) = handle_conditions(
                            &folder.path,
                            rule,
                            &mut processed_files,
                            &mut file_movements,
                        ) {
                            warn!("Ignoring rule '{}': {}", rule.name, e);
                            log_error(&format!("Ignoring rule '{}': {}", rule.name, e));
                            ignored_rules.insert(rule.name.clone());
                        }
                    }
                }
            }
        }
        Err(e) => error!("Error loading config: {}", e),
    }
}

fn watch_orderly() {
    info!("Running initial organization...");
    run_orderly(); // Perform the initial run

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

fn handle_conditions(
    folder_path: &str,
    rule: &FolderRule,
    processed_files: &mut HashSet<String>,
    file_movements: &mut HashMap<String, usize>,
) -> Result<(), String> {
    let folder = Path::new(folder_path);
    if !folder.exists() {
        let msg = format!("Directory does not exist: {}", folder.display());
        error!("{}", msg);
        return Err(msg);
    }

    let entries = folder.read_dir().map_err(|e| {
        let msg = format!("Failed to read directory {}: {}", folder.display(), e);
        error!("{}", msg);
        msg
    })?;

    for entry in entries {
        if let Ok(entry) = entry {
            let src_path = entry.path();
            let src_path_str = src_path.to_str().unwrap().to_string();

            if processed_files.contains(&src_path_str) {
                info!("Skipping already processed file: {}", src_path.display());
                continue;
            }

            for condition in &rule.conditions {
                let cond = create_condition(&condition.condition_type, &condition.value);
                if cond.evaluate(&src_path) {
                    processed_files.insert(src_path_str.clone());

                    let movement_count = file_movements.entry(src_path_str.clone()).or_insert(0);
                    *movement_count += 1;

                    if *movement_count > MAX_MOVEMENTS {
                        let msg = format!(
                            "Potential infinite loop detected for file: {}",
                            src_path.display()
                        );
                        warn!("{}", msg);
                        log_error(&msg);
                        return Err(msg);
                    }

                    for action in &rule.actions {
                        execute_action(&src_path, action, processed_files, file_movements);
                    }
                }
            }
        }
    }
    Ok(())
}

fn execute_action(
    src_path: &Path,
    action: &Action,
    processed_files: &mut HashSet<String>,
    file_movements: &mut HashMap<String, usize>,
) {
    match action.action_type.as_str() {
        "delete" => handle_delete(src_path),
        "move" => handle_move(src_path, action, processed_files, file_movements),
        "copy" => handle_copy(src_path, action, processed_files, file_movements),
        "sort_by_date" => handle_sort_by_date(src_path, action, processed_files),
        _ => log_error(&format!("Unknown action type: {}", action.action_type)),
    }
}

fn handle_delete(src_path: &Path) {
    info!("Deleting file: {}", src_path.display());
    if let Err(e) = actions::delete_file(src_path.to_str().unwrap()) {
        log_error(&format!("Failed to delete file: {}", e));
    }
}

fn handle_move(
    src_path: &Path,
    action: &Action,
    processed_files: &mut HashSet<String>,
    file_movements: &mut HashMap<String, usize>,
) {
    let dest_path = action.path.as_ref().unwrap().replace("~", &HOME);

    info!("Moving file from {} to {}", src_path.display(), dest_path);
    if let Err(e) = actions::move_file(src_path.to_str().unwrap(), &dest_path) {
        log_error(&format!("Failed to move file: {}", e));
    } else {
        processed_files.insert(dest_path.clone());
        let dest_path_str = dest_path.to_string();
        let movement_count = file_movements.entry(dest_path_str).or_insert(0);
        *movement_count += 1;
    }
}

fn handle_copy(
    src_path: &Path,
    action: &Action,
    processed_files: &mut HashSet<String>,
    file_movements: &mut HashMap<String, usize>,
) {
    let dest_path = action
        .path
        .as_ref()
        .unwrap()
        .replace("~", &env::var("HOME").unwrap());
    info!("Copying file from {} to {}", src_path.display(), dest_path);
    if let Err(e) = actions::copy_file(src_path.to_str().unwrap(), &dest_path) {
        log_error(&format!("Failed to copy file: {}", e));
    } else {
        processed_files.insert(dest_path.clone());
        let dest_path_str = dest_path.to_string();
        let movement_count = file_movements.entry(dest_path_str).or_insert(0);
        *movement_count += 1;
    }
}

fn handle_sort_by_date(src_path: &Path, action: &Action, processed_files: &mut HashSet<String>) {
    let base_path = action.path.as_ref().unwrap().replace("~", &HOME);

    let pattern = action.pattern.as_ref().unwrap();
    info!(
        "Sorting file by date from {} to {}",
        src_path.display(),
        base_path
    );
    if let Err(e) = actions::sort_file_by_date(src_path.to_str().unwrap(), &base_path, pattern) {
        log_error(&format!("Failed to sort file by date: {}", e));
    } else {
        let dest_path = Path::new(&base_path).join(src_path.file_name().unwrap());
        processed_files.insert(dest_path.to_str().unwrap().to_string());
    }
}

fn log_error(message: &str) {
    let log_file_path = "error.log";
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(log_file_path)
        .unwrap();
    writeln!(file, "{}", message).unwrap();
}
