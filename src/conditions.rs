use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Per-folder evaluation context. Populated lazily in main.rs only for folders
/// whose rules actually reference a context-dependent condition (e.g.
/// `is_duplicate`); folders without such conditions pay zero overhead.
#[derive(Debug, Default)]
pub struct RuleContext {
    /// Absolute paths of files that are content-hash duplicates of another file
    /// already seen in the same folder. The oldest (earliest mtime) file with a
    /// given hash is treated as canonical and is NOT in this set.
    pub duplicates: HashSet<PathBuf>,
}

pub trait Condition {
    fn evaluate(&self, path: &Path, ctx: &RuleContext) -> bool;
}

pub struct Always;

impl Condition for Always {
    fn evaluate(&self, path: &Path, _ctx: &RuleContext) -> bool {
        path.is_file()
    }
}

pub struct NameEquals {
    pub name: String,
}

impl Condition for NameEquals {
    fn evaluate(&self, path: &Path, _ctx: &RuleContext) -> bool {
        path.file_name().unwrap().to_str().unwrap() == self.name
    }
}

pub struct ExtensionIn {
    pub extensions: Vec<String>,
}

impl Condition for ExtensionIn {
    fn evaluate(&self, path: &Path, _ctx: &RuleContext) -> bool {
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            self.extensions.contains(&ext.to_string())
        } else {
            false
        }
    }
}

pub struct NameContains {
    pub substring: String,
}

impl Condition for NameContains {
    fn evaluate(&self, path: &Path, _ctx: &RuleContext) -> bool {
        path.to_str().unwrap().contains(&self.substring)
    }
}

/// True when `path` is a content-hash duplicate of an older file in the same
/// folder. Relies on `RuleContext::duplicates` being pre-populated by the
/// runner (see `find_duplicates` in main.rs).
pub struct IsDuplicate;

impl Condition for IsDuplicate {
    fn evaluate(&self, path: &Path, ctx: &RuleContext) -> bool {
        ctx.duplicates.contains(path)
    }
}

/// True when the file's mtime is at least `duration` in the past. Used to
/// implement YAML rules like `older_than: 30d` for scheduled trash cleanups.
pub struct OlderThan {
    pub duration: Duration,
}

impl Condition for OlderThan {
    fn evaluate(&self, path: &Path, _ctx: &RuleContext) -> bool {
        path.metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| SystemTime::now().duration_since(t).ok())
            .map(|age| age >= self.duration)
            .unwrap_or(false)
    }
}

/// Parse durations like "30d", "14d", "2w", "6h", "45m", "90s".
///
/// Units: `s` seconds, `m` minutes, `h` hours, `d` days, `w` weeks.
/// Months and years are intentionally unsupported to avoid ambiguity.
pub fn parse_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("empty duration".to_string());
    }

    let (num_part, unit) = match s.chars().last() {
        Some(c) if c.is_alphabetic() => (&s[..s.len() - 1], c),
        _ => return Err(format!("duration '{}' is missing a unit (s/m/h/d/w)", s)),
    };

    let n: u64 = num_part
        .parse()
        .map_err(|_| format!("invalid number in duration '{}'", s))?;

    let seconds = match unit {
        's' => n,
        'm' => n * 60,
        'h' => n * 60 * 60,
        'd' => n * 60 * 60 * 24,
        'w' => n * 60 * 60 * 24 * 7,
        other => {
            return Err(format!(
                "unknown duration unit '{}' in '{}' (use s/m/h/d/w)",
                other, s
            ))
        }
    };
    Ok(Duration::from_secs(seconds))
}

pub fn create_condition(condition_type: &str, value: &str) -> Box<dyn Condition> {
    match condition_type {
        "always" => Box::new(Always),
        "name" => Box::new(NameEquals {
            name: value.to_string(),
        }),
        "extension" => Box::new(ExtensionIn {
            extensions: value.split(',').map(|s| s.trim().to_string()).collect(),
        }),
        "name_contains" => Box::new(NameContains {
            substring: value.to_string(),
        }),
        "is_duplicate" => Box::new(IsDuplicate),
        "older_than" => Box::new(OlderThan {
            duration: parse_duration(value)
                .unwrap_or_else(|e| panic!("invalid `older_than` value: {}", e)),
        }),
        _ => panic!("Unknown condition type: {}", condition_type),
    }
}

/// Returns true if any rule in `rules` references a condition that needs the
/// duplicate-detection pre-pass. Used by main.rs to decide whether to invoke
/// the (potentially expensive) `find_duplicates` scan for a given folder.
pub fn needs_duplicate_scan(conditions: &[String]) -> bool {
    conditions.iter().any(|c| c == "is_duplicate")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_duration_units() {
        assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(5 * 60));
        assert_eq!(parse_duration("6h").unwrap(), Duration::from_secs(6 * 3600));
        assert_eq!(
            parse_duration("14d").unwrap(),
            Duration::from_secs(14 * 86400)
        );
        assert_eq!(
            parse_duration("2w").unwrap(),
            Duration::from_secs(2 * 7 * 86400)
        );
    }

    #[test]
    fn parse_duration_rejects_bad_inputs() {
        assert!(parse_duration("").is_err());
        assert!(parse_duration("30").is_err());
        assert!(parse_duration("abc").is_err());
        assert!(parse_duration("5x").is_err());
        assert!(parse_duration("1y").is_err());
    }
}
