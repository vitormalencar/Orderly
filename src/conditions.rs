use std::path::Path;

pub trait Condition {
    fn evaluate(&self, path: &Path) -> bool;
}

pub struct Always;

impl Condition for Always {
    fn evaluate(&self, path: &Path) -> bool {
        path.is_file()
    }
}

pub struct NameEquals {
    pub name: String,
}

impl Condition for NameEquals {
    fn evaluate(&self, path: &Path) -> bool {
        path.file_name().unwrap().to_str().unwrap() == self.name
    }
}

pub struct ExtensionIn {
    pub extensions: Vec<String>,
}

impl Condition for ExtensionIn {
    fn evaluate(&self, path: &Path) -> bool {
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
    fn evaluate(&self, path: &Path) -> bool {
        path.to_str().unwrap().contains(&self.substring)
    }
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
        _ => panic!("Unknown condition type: {}", condition_type),
    }
}