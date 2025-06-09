use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;

use crate::Result;

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
    pub pattern: Option<String>, // Added pattern field
}

pub fn load_config(path: &str) -> Result<Rule> {
    let content = fs::read_to_string(path)?;
    let rule: Rule = serde_yaml::from_str(&content)?;
    Ok(rule)
}

pub fn create_example_rule() -> std::io::Result<()> {
    let example_rule = Rule {
        name: "Orderly Sandbox Rules".into(),
        description: "Rules for organizing files in the sandbox environment".into(),
        folders: vec![
            Folder {
                path: "test_folder/Desktop".into(),
                match_type: "all".into(),
                rules: vec![FolderRule {
                    name: "Move all files to Downloads".into(),
                    conditions: vec![Condition {
                        condition_type: "always".into(),
                        value: "".into(),
                    }],
                    actions: vec![Action {
                        action_type: "move".into(),
                        path: Some("test_folder/Downloads".into()),
                        pattern: None,
                    }],
                }],
            },
            Folder {
                path: "test_folder/Downloads".into(),
                match_type: "all".into(),
                rules: vec![
                    FolderRule {
                        name: "Move music files to Music folder".into(),
                        conditions: vec![Condition {
                            condition_type: "extension".into(),
                            value: "mp3,flac,wav,ogg,m4a,wma,aac,aiff,aif".into(),
                        }],
                        actions: vec![Action {
                            action_type: "move".into(),
                            path: Some("test_folder/Music".into()),
                            pattern: None,
                        }],
                    },
                    FolderRule {
                        name: "Move video files to Videos folder".into(),
                        conditions: vec![Condition {
                            condition_type: "extension".into(),
                            value: "mp4,mov,avi,wmv,mkv".into(),
                        }],
                        actions: vec![Action {
                            action_type: "move".into(),
                            path: Some("test_folder/Videos".into()),
                            pattern: None,
                        }],
                    },
                    FolderRule {
                        name: "Move document files to Documents folder".into(),
                        conditions: vec![Condition {
                            condition_type: "extension".into(),
                            value: "pdf,txt,doc,docx,xls,xlsx,ppt,pptx".into(),
                        }],
                        actions: vec![Action {
                            action_type: "move".into(),
                            path: Some("test_folder/Documents".into()),
                            pattern: None,
                        }],
                    },
                    FolderRule {
                        name: "Move picture files to Pictures folder".into(),
                        conditions: vec![Condition {
                            condition_type: "extension".into(),
                            value: "jpg,png,gif".into(),
                        }],
                        actions: vec![Action {
                            action_type: "move".into(),
                            path: Some("test_folder/Pictures".into()),
                            pattern: None,
                        }],
                    },
                ],
            },
            Folder {
                path: "test_folder/Pictures".into(),
                match_type: "all".into(),
                rules: vec![
                    FolderRule {
                        name: "Move wallpapers to Wallpapers subfolder".into(),
                        conditions: vec![Condition {
                            condition_type: "name_contains".into(),
                            value: "wallpaper".into(),
                        }],
                        actions: vec![Action {
                            action_type: "move".into(),
                            path: Some("test_folder/Pictures/Wallpapers".into()),
                            pattern: None,
                        }],
                    },
                    FolderRule {
                        name: "Move clearshots to Wallpapers subfolder".into(),
                        conditions: vec![Condition {
                            condition_type: "name_contains".into(),
                            value: "clearshot".into(),
                        }],
                        actions: vec![Action {
                            action_type: "move".into(),
                            path: Some("test_folder/Pictures/Wallpapers".into()),
                            pattern: None,
                        }],
                    },
                    FolderRule {
                        name: "Move screenshots to Screenshots subfolder".into(),
                        conditions: vec![Condition {
                            condition_type: "name_contains".into(),
                            value: "screenshot".into(),
                        }],
                        actions: vec![Action {
                            action_type: "move".into(),
                            path: Some("test_folder/Pictures/Screenshots".into()),
                            pattern: None,
                        }],
                    },
                    FolderRule {
                        name: "Sort images into year/month subfolders".into(),
                        conditions: vec![Condition {
                            condition_type: "extension".into(),
                            value: "jpg,png,gif".into(),
                        }],
                        actions: vec![Action {
                            action_type: "sort_by_date".into(),
                            path: Some("test_folder/Pictures".into()),
                            pattern: Some("%Y/%b".into()),
                        }],
                    },
                ],
            },
        ],
    };

    let yaml =
        serde_yaml::to_string(&example_rule).expect("Failed to convert example rule to YAML");
    let mut file = fs::File::create("rules/example.yaml")?;
    file.write_all(yaml.as_bytes())?;
    Ok(())
}
