use chrono::{DateTime, Local};
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, ContentArrangement, Table};

/// Outcome of a rule matching (and possibly acting on) a single file.
#[derive(Debug, Clone)]
pub enum Status {
    /// Dry-run: would have executed but did nothing.
    Planned,
    /// Real run: action completed successfully.
    Executed,
    /// Action was attempted and failed.
    Error(String),
}

impl Status {
    fn label(&self) -> &'static str {
        match self {
            Status::Planned => "PLANNED",
            Status::Executed => "EXECUTED",
            Status::Error(_) => "ERROR",
        }
    }

    fn detail(&self) -> Option<&str> {
        match self {
            Status::Error(m) => Some(m.as_str()),
            _ => None,
        }
    }

    fn color(&self) -> Color {
        match self {
            Status::Planned => Color::Yellow,
            Status::Executed => Color::Green,
            Status::Error(_) => Color::Red,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuleExecution {
    pub folder: String,
    pub file: String,
    pub rule: String,
    pub condition: String,
    pub action: String,
    pub destination: Option<String>,
    pub status: Status,
    pub when: DateTime<Local>,
}

#[derive(Debug, Default)]
pub struct Report {
    pub entries: Vec<RuleExecution>,
}

impl Report {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, row: RuleExecution) {
        self.entries.push(row);
    }

    /// Render the collected outcomes as a comfy-table. The `header` is printed
    /// above the table and typically reflects the run mode
    /// (e.g. "DRY-RUN (no files were changed)").
    pub fn print(&self, header: &str) {
        if self.entries.is_empty() {
            println!("\n{}\n(no rules matched any files)\n", header);
            return;
        }

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                "When", "Folder", "File", "Rule", "Condition", "Action", "Destination", "Status",
            ]);

        for e in &self.entries {
            let status_text = match e.status.detail() {
                Some(d) => format!("{}: {}", e.status.label(), d),
                None => e.status.label().to_string(),
            };
            table.add_row(vec![
                Cell::new(e.when.format("%H:%M:%S").to_string()),
                Cell::new(shorten_path(&e.folder)),
                Cell::new(&e.file),
                Cell::new(&e.rule),
                Cell::new(&e.condition),
                Cell::new(&e.action),
                Cell::new(e.destination.as_deref().unwrap_or("")),
                Cell::new(status_text).fg(e.status.color()),
            ]);
        }

        println!("\n{}\n{}\n", header, table);
        self.print_summary();
    }

    fn print_summary(&self) {
        let (mut planned, mut executed, mut errored) = (0, 0, 0);
        for e in &self.entries {
            match e.status {
                Status::Planned => planned += 1,
                Status::Executed => executed += 1,
                Status::Error(_) => errored += 1,
            }
        }
        println!(
            "Summary: {} planned, {} executed, {} errored ({} total)\n",
            planned,
            executed,
            errored,
            self.entries.len()
        );
    }
}

/// Replace the current user's home prefix with `~` to keep the table narrow.
fn shorten_path(path: &str) -> String {
    if let Ok(home) = std::env::var("HOME") {
        if let Some(rest) = path.strip_prefix(&home) {
            return format!("~{}", rest);
        }
    }
    path.to_string()
}
