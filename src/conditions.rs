use std::path::Path;

use crate::actions::get_file_name;

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
        match get_file_name(path) {
            Ok(file_name) => file_name == self.name,
            Err(_) => false,
        }
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
        match path.to_str() {
            Some(path) => path.contains(&self.substring),
            None => false,
        }
    }
}

impl From<&crate::config::Condition> for Box<dyn Condition> {
    fn from(condition: &crate::config::Condition) -> Self {
        match condition.condition_type.as_str() {
            "always" => Box::new(Always),
            "name" => Box::new(NameEquals {
                name: condition.value.clone(),
            }),
            "extension" => Box::new(ExtensionIn {
                extensions: condition
                    .value
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
            }),
            "name_contains" => Box::new(NameContains {
                substring: condition.value.clone(),
            }),
            _ => panic!("Unknown condition type: {}", condition.condition_type),
        }
    }
}
