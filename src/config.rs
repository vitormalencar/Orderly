use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Rule {
    pub name: String,
    pub description: String,
    pub actions: Vec<Action>,
}

#[derive(Debug, Deserialize)]
pub struct Action {
    pub name: String,
    pub path: String,
    pub condition: Condition,
    pub action: String,
}

#[derive(Debug, Deserialize)]
pub struct Condition {
    pub r#type: String,
    pub size: Option<String>,
    pub extension: Option<String>,
    pub date: Option<String>,
}

pub fn load_rules(path: &str) -> Vec<Rule> {
    let content = fs::read_to_string(path).expect("Unable to read file");
    let rules: Vec<Rule> = serde_yaml::from_str(&content).expect("Unable to parse YAML");
    rules
}
