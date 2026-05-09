mod actions;
mod conditions;
mod config;
mod report;
use crate::conditions::{create_condition, needs_duplicate_scan, RuleContext};
use crate::config::{Action, FolderRule};
use crate::report::{Report, RuleExecution, Status};

use chrono::Local;
use clap::{App, Arg};
use log::{error, info, warn};
use notify::{recommended_watcher, RecursiveMode, Watcher};
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::SystemTime;

const MAX_MOVEMENTS: usize = 10;

/// macOS folder metadata; never organize or react to these as "real" files.
fn is_ignored_metadata_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|name| name == ".DS_Store")
}

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
        .arg(Arg::new("dry-run").short('d').long("dry-run"))
        .get_matches();

    let dry_run = matches.is_present("dry-run");

    if matches.is_present("init") {
        init_orderly();
    }

    if matches.is_present("run") {
        run_orderly(dry_run);
    }

    if matches.is_present("watch") {
        watch_orderly(dry_run);
    }
}

fn init_orderly() {
    info!("Initializing Orderly...");
    if let Err(e) = config::create_example_rule() {
        error!("Failed to create example rule: {}", e);
    }
}

fn run_orderly(dry_run: bool) {
    if dry_run {
        info!("Running Orderly (DRY RUN — no filesystem changes)...");
    } else {
        info!("Running Orderly...");
    }
    let mut processed_files = HashSet::new();
    let mut file_movements = HashMap::new();
    let mut ignored_rules = HashSet::new();
    let mut report = Report::new();

    match config::load_config("rules/example.yaml") {
        Ok(config) => {
            for folder in config.folders {
                let ctx = build_rule_context(&folder);
                for rule in &folder.rules {
                    if !ignored_rules.contains(&rule.name) {
                        if let Err(e) = handle_conditions(
                            &folder.path,
                            rule,
                            &ctx,
                            &mut processed_files,
                            &mut file_movements,
                            dry_run,
                            &mut report,
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

    let header = if dry_run {
        "DRY-RUN REPORT — no files were changed"
    } else {
        "RUN REPORT"
    };
    report.print(header);
}

/// Build a `RuleContext` for a folder. Expensive passes (e.g. content-hash
/// duplicate detection) are only run if at least one rule in the folder opts
/// into a condition that requires them, so the cost is zero for folders that
/// don't use them.
fn build_rule_context(folder: &config::Folder) -> RuleContext {
    let condition_types: Vec<String> = folder
        .rules
        .iter()
        .flat_map(|r| r.conditions.iter().map(|c| c.condition_type.clone()))
        .collect();

    let mut ctx = RuleContext::default();
    if needs_duplicate_scan(&condition_types) {
        let folder_path = Path::new(&folder.path);
        if folder_path.exists() {
            info!("Scanning {} for duplicates...", folder.path);
            ctx.duplicates = find_duplicates(folder_path);
            info!(
                "Found {} duplicate file(s) in {}",
                ctx.duplicates.len(),
                folder.path
            );
        }
    }
    ctx
}

/// Return the set of paths inside `folder` that are content-hash duplicates of
/// another file in the same folder. For each hash the OLDEST file (earliest
/// mtime) is kept as canonical and is NOT included in the returned set.
///
/// Optimization: files are bucketed by size first, and only buckets with 2+
/// entries are hashed. For a typical Downloads folder this skips the vast
/// majority of files without ever opening them.
fn find_duplicates(folder: &Path) -> HashSet<PathBuf> {
    let mut by_size: HashMap<u64, Vec<(PathBuf, SystemTime)>> = HashMap::new();

    let entries = match folder.read_dir() {
        Ok(e) => e,
        Err(e) => {
            warn!("find_duplicates: cannot read {}: {}", folder.display(), e);
            return HashSet::new();
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if is_ignored_metadata_file(&path) {
            continue;
        }
        let meta = match path.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        // Empty files are noise: treating them as duplicates of each other is
        // rarely what the user wants.
        if meta.len() == 0 {
            continue;
        }
        let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        by_size.entry(meta.len()).or_default().push((path, mtime));
    }

    let mut duplicates = HashSet::new();
    for (_size, mut paths) in by_size {
        if paths.len() < 2 {
            continue;
        }
        // Sort oldest first so `seen.insert(..).is_some()` flags the newer
        // copies as duplicates and the oldest one stays canonical.
        paths.sort_by_key(|(_, t)| *t);

        let mut seen: HashMap<[u8; 32], ()> = HashMap::new();
        for (path, _) in paths {
            let bytes = match std::fs::read(&path) {
                Ok(b) => b,
                Err(e) => {
                    warn!("find_duplicates: cannot read {}: {}", path.display(), e);
                    continue;
                }
            };
            let hash: [u8; 32] = *blake3::hash(&bytes).as_bytes();
            if seen.insert(hash, ()).is_some() {
                duplicates.insert(path);
            }
        }
    }
    duplicates
}

fn watch_orderly(dry_run: bool) {
    info!("Running initial organization...");
    run_orderly(dry_run); // Perform the initial run

    info!("Watching for changes...");
    match config::load_config("rules/example.yaml") {
        Ok(config) => {
            let (tx, rx) = channel();
            let mut watcher = recommended_watcher(move |res| tx.send(res).unwrap()).unwrap();

            for folder in config.folders {
                let path = Path::new(&folder.path);
                if !path.exists() {
                    warn!(
                        "Skipping watch for missing folder: {}",
                        path.display()
                    );
                    continue;
                }
                if let Err(e) = watcher.watch(path, RecursiveMode::Recursive) {
                    error!("Failed to watch {}: {}", path.display(), e);
                }
            }

            loop {
                match rx.recv() {
                    Ok(event) => match event {
                        Ok(notify::Event { kind, paths, .. }) => {
                            let has_meaningful_path =
                                paths.iter().any(|p| !is_ignored_metadata_file(p));
                            if !paths.is_empty() && !has_meaningful_path {
                                continue;
                            }
                            for path in &paths {
                                info!("File change detected: {:?}, {:?}", path, kind);
                            }
                            run_orderly(dry_run);
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
    ctx: &RuleContext,
    processed_files: &mut HashSet<String>,
    file_movements: &mut HashMap<String, usize>,
    dry_run: bool,
    report: &mut Report,
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
            // Orderly is a file-organization tool; subdirectories (e.g. the
            // `CleanShot/` folder inside `~/Movies`) must never be fed to
            // `move`/`copy`/`delete`/`sort_by_date` — those actions all
            // assume regular files. Without this filter, a rule like
            // `name_contains: CleanShot` would match the `CleanShot`
            // directory itself and try to move it into itself.
            if !src_path.is_file() {
                continue;
            }
            if is_ignored_metadata_file(&src_path) {
                continue;
            }
            let src_path_str = src_path.to_str().unwrap().to_string();

            if processed_files.contains(&src_path_str) {
                info!("Skipping already processed file: {}", src_path.display());
                continue;
            }

            for condition in &rule.conditions {
                let cond = create_condition(&condition.condition_type, &condition.value);
                if cond.evaluate(&src_path, ctx) {
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

                    let condition_desc = describe_condition(condition);
                    for action in &rule.actions {
                        execute_action(
                            &src_path,
                            action,
                            processed_files,
                            file_movements,
                            dry_run,
                            report,
                            folder_path,
                            &rule.name,
                            &condition_desc,
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

fn describe_condition(condition: &config::Condition) -> String {
    if condition.value.is_empty() {
        condition.condition_type.clone()
    } else {
        format!("{}={}", condition.condition_type, condition.value)
    }
}

fn file_display_name(src_path: &Path) -> String {
    src_path
        .file_name()
        .and_then(|n| n.to_str())
        .map(String::from)
        .unwrap_or_else(|| src_path.display().to_string())
}

fn execute_action(
    src_path: &Path,
    action: &Action,
    processed_files: &mut HashSet<String>,
    file_movements: &mut HashMap<String, usize>,
    dry_run: bool,
    report: &mut Report,
    folder_path: &str,
    rule_name: &str,
    condition_desc: &str,
) {
    match action.action_type.as_str() {
        "delete" => handle_delete(src_path, dry_run, report, folder_path, rule_name, condition_desc),
        "move" => handle_move(
            src_path,
            action,
            processed_files,
            file_movements,
            dry_run,
            report,
            folder_path,
            rule_name,
            condition_desc,
        ),
        "copy" => handle_copy(
            src_path,
            action,
            processed_files,
            file_movements,
            dry_run,
            report,
            folder_path,
            rule_name,
            condition_desc,
        ),
        "sort_by_date" => handle_sort_by_date(
            src_path,
            action,
            processed_files,
            dry_run,
            report,
            folder_path,
            rule_name,
            condition_desc,
        ),
        other => {
            log_error(&format!("Unknown action type: {}", other));
            report.record(RuleExecution {
                folder: folder_path.to_string(),
                file: file_display_name(src_path),
                rule: rule_name.to_string(),
                condition: condition_desc.to_string(),
                action: action.action_type.clone(),
                destination: None,
                status: Status::Error(format!("unknown action type: {}", other)),
                when: Local::now(),
            });
        }
    }
}

fn handle_delete(
    src_path: &Path,
    dry_run: bool,
    report: &mut Report,
    folder_path: &str,
    rule_name: &str,
    condition_desc: &str,
) {
    let file = file_display_name(src_path);
    let destination = Some("<trash>".to_string());

    if dry_run {
        info!("[dry-run] Would delete file: {}", src_path.display());
        report.record(RuleExecution {
            folder: folder_path.to_string(),
            file,
            rule: rule_name.to_string(),
            condition: condition_desc.to_string(),
            action: "delete".to_string(),
            destination,
            status: Status::Planned,
            when: Local::now(),
        });
        return;
    }

    info!("Deleting file: {}", src_path.display());
    let status = match actions::delete_file(src_path.to_str().unwrap()) {
        Ok(()) => Status::Executed,
        Err(e) => {
            log_error(&format!("Failed to delete file: {}", e));
            Status::Error(e.to_string())
        }
    };
    report.record(RuleExecution {
        folder: folder_path.to_string(),
        file,
        rule: rule_name.to_string(),
        condition: condition_desc.to_string(),
        action: "delete".to_string(),
        destination,
        status,
        when: Local::now(),
    });
}

fn handle_move(
    src_path: &Path,
    action: &Action,
    processed_files: &mut HashSet<String>,
    file_movements: &mut HashMap<String, usize>,
    dry_run: bool,
    report: &mut Report,
    folder_path: &str,
    rule_name: &str,
    condition_desc: &str,
) {
    let dest_path = action.path.as_ref().unwrap().replace("~", &HOME);
    let file = file_display_name(src_path);

    if dry_run {
        info!(
            "[dry-run] Would move {} to {}",
            src_path.display(),
            dest_path
        );
        report.record(RuleExecution {
            folder: folder_path.to_string(),
            file,
            rule: rule_name.to_string(),
            condition: condition_desc.to_string(),
            action: "move".to_string(),
            destination: Some(dest_path),
            status: Status::Planned,
            when: Local::now(),
        });
        return;
    }

    info!("Moving file from {} to {}", src_path.display(), dest_path);
    let status = match actions::move_file(src_path.to_str().unwrap(), &dest_path) {
        Ok(()) => {
            processed_files.insert(dest_path.clone());
            let movement_count = file_movements.entry(dest_path.clone()).or_insert(0);
            *movement_count += 1;
            Status::Executed
        }
        Err(e) => {
            log_error(&format!("Failed to move file: {}", e));
            Status::Error(e.to_string())
        }
    };
    report.record(RuleExecution {
        folder: folder_path.to_string(),
        file,
        rule: rule_name.to_string(),
        condition: condition_desc.to_string(),
        action: "move".to_string(),
        destination: Some(dest_path),
        status,
        when: Local::now(),
    });
}

fn handle_copy(
    src_path: &Path,
    action: &Action,
    processed_files: &mut HashSet<String>,
    file_movements: &mut HashMap<String, usize>,
    dry_run: bool,
    report: &mut Report,
    folder_path: &str,
    rule_name: &str,
    condition_desc: &str,
) {
    let dest_path = action
        .path
        .as_ref()
        .unwrap()
        .replace("~", &env::var("HOME").unwrap());
    let file = file_display_name(src_path);

    if dry_run {
        info!(
            "[dry-run] Would copy {} to {}",
            src_path.display(),
            dest_path
        );
        report.record(RuleExecution {
            folder: folder_path.to_string(),
            file,
            rule: rule_name.to_string(),
            condition: condition_desc.to_string(),
            action: "copy".to_string(),
            destination: Some(dest_path),
            status: Status::Planned,
            when: Local::now(),
        });
        return;
    }

    info!("Copying file from {} to {}", src_path.display(), dest_path);
    let status = match actions::copy_file(src_path.to_str().unwrap(), &dest_path) {
        Ok(()) => {
            processed_files.insert(dest_path.clone());
            let movement_count = file_movements.entry(dest_path.clone()).or_insert(0);
            *movement_count += 1;
            Status::Executed
        }
        Err(e) => {
            log_error(&format!("Failed to copy file: {}", e));
            Status::Error(e.to_string())
        }
    };
    report.record(RuleExecution {
        folder: folder_path.to_string(),
        file,
        rule: rule_name.to_string(),
        condition: condition_desc.to_string(),
        action: "copy".to_string(),
        destination: Some(dest_path),
        status,
        when: Local::now(),
    });
}

fn handle_sort_by_date(
    src_path: &Path,
    action: &Action,
    processed_files: &mut HashSet<String>,
    dry_run: bool,
    report: &mut Report,
    folder_path: &str,
    rule_name: &str,
    condition_desc: &str,
) {
    let base_path = action.path.as_ref().unwrap().replace("~", &HOME);
    let pattern = action.pattern.as_ref().unwrap();
    let file = file_display_name(src_path);

    if dry_run {
        let destination = actions::sort_by_date_destination(src_path, &base_path, pattern)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|e| format!("<error computing dest: {}>", e));
        info!(
            "[dry-run] Would sort-by-date {} -> {}",
            src_path.display(),
            destination
        );
        report.record(RuleExecution {
            folder: folder_path.to_string(),
            file,
            rule: rule_name.to_string(),
            condition: condition_desc.to_string(),
            action: "sort_by_date".to_string(),
            destination: Some(destination),
            status: Status::Planned,
            when: Local::now(),
        });
        return;
    }

    info!(
        "Sorting file by date from {} to {}",
        src_path.display(),
        base_path
    );
    let destination = actions::sort_by_date_destination(src_path, &base_path, pattern)
        .map(|p| p.display().to_string())
        .ok();
    let status = match actions::sort_file_by_date(src_path.to_str().unwrap(), &base_path, pattern) {
        Ok(()) => {
            if let Some(ref d) = destination {
                processed_files.insert(d.clone());
            }
            Status::Executed
        }
        Err(e) => {
            log_error(&format!("Failed to sort file by date: {}", e));
            Status::Error(e.to_string())
        }
    };
    report.record(RuleExecution {
        folder: folder_path.to_string(),
        file,
        rule: rule_name.to_string(),
        condition: condition_desc.to_string(),
        action: "sort_by_date".to_string(),
        destination,
        status,
        when: Local::now(),
    });
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
