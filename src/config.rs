use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize)]
pub struct Rule {
    pub name: String,
    pub description: String,
    pub folders: Vec<Folder>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Folder {
    pub path: String,
    pub match_type: String,
    pub rules: Vec<FolderRule>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FolderRule {
    pub name: String,
    pub conditions: Vec<Condition>,
    pub actions: Vec<Action>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Condition {
    pub condition_type: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Action {
    pub action_type: String,
    pub path: Option<String>,
    pub pattern: Option<String>,  // Added pattern field
}

pub fn load_config(path: &str) -> Result<Rule, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let rule: Rule = serde_yaml::from_str(&content)?;
    Ok(rule)
}

pub fn create_example_rule() -> std::io::Result<()> {
    let example_rule = Rule {
        name: "Example Rule".into(),
        description: "An example rule for organizing files".into(),
        folders: vec![Folder {
            path: "./test_folder".into(),
            match_type: "all".into(),
            rules: vec![
                FolderRule {
                    name: "Move move_me.png".into(),
                    conditions: vec![Condition {
                        condition_type: "name".into(),
                        value: "move_me.png".into(),
                    }],
                    actions: vec![Action {
                        action_type: "move".into(),
                        path: Some("./test_folder/organized".into()),
                        pattern: None,
                    }],
                },
                FolderRule {
                    name: "Move move_me_missing_folder.png".into(),
                    conditions: vec![Condition {
                        condition_type: "name".into(),
                        value: "move_me_missing_folder.png".into(),
                    }],
                    actions: vec![Action {
                        action_type: "move".into(),
                        path: Some("./test_folder/organized/move/missing_folder".into()),
                        pattern: None,
                    }],
                },
                FolderRule {
                    name: "Copy copy_me.png".into(),
                    conditions: vec![Condition {
                        condition_type: "name".into(),
                        value: "copy_me.png".into(),
                    }],
                    actions: vec![Action {
                        action_type: "copy".into(),
                        path: Some("./test_folder/organized".into()),
                        pattern: None,
                    }],
                },
                FolderRule {
                    name: "Copy copy_me_missing_folder.png".into(),
                    conditions: vec![Condition {
                        condition_type: "name".into(),
                        value: "copy_me_missing_folder.png".into(),
                    }],
                    actions: vec![Action {
                        action_type: "copy".into(),
                        path: Some("./test_folder/organized/copy/missing_folder".into()),
                        pattern: None,
                    }],
                },
                FolderRule {
                    name: "Delete delete_me.png".into(),
                    conditions: vec![Condition {
                        condition_type: "name".into(),
                        value: "delete_me.png".into(),
                    }],
                    actions: vec![Action {
                        action_type: "delete".into(),
                        path: None,
                        pattern: None,
                    }],
                },
                FolderRule {
                    name: "Sort images by date".into(),
                    conditions: vec![Condition {
                        condition_type: "extension".into(),
                        value: "jpg,png,gif".into(),
                    }],
                    actions: vec![Action {
                        action_type: "sort_by_date".into(),
                        path: Some("./test_folder/sorted_by_date".into()),
                        pattern: Some("%Y/%m".into()),
                    }],
                },
            ],
        }],
    };

    let yaml = serde_yaml::to_string(&example_rule).unwrap();
    let mut file = fs::File::create("rules/example.yaml")?;
    file.write_all(yaml.as_bytes())?;
    Ok(())
}
