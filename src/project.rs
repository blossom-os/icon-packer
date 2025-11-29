use std::collections::BTreeMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconProject {
    pub name: String,
    pub theme_name: String,
    #[serde(default)]
    pub theme_comment: String,
    pub output_path: Option<PathBuf>,
    pub fallback_themes: Vec<String>, // Themes to inherit from
    pub icon_replacements: BTreeMap<String, PathBuf>,
    pub size_specific_replacements: BTreeMap<String, BTreeMap<u32, PathBuf>>, // icon_name -> size -> path
    #[serde(default)]
    pub icon_links: BTreeMap<String, bool>, // icon_name -> is_link (for base replacements)
    #[serde(default)]
    pub size_specific_links: BTreeMap<String, BTreeMap<u32, bool>>, // icon_name -> size -> is_link
    #[serde(default)]
    pub icon_categories: BTreeMap<String, String>, // icon_name -> category
}

impl IconProject {
    pub fn new(name: String) -> Self {
        Self {
            name,
            theme_name: String::new(),
            theme_comment: String::new(),
            output_path: None,
            fallback_themes: vec!["hicolor".to_string()], // Default fallback
            icon_replacements: BTreeMap::new(),
            size_specific_replacements: BTreeMap::new(),
            icon_links: BTreeMap::new(),
            size_specific_links: BTreeMap::new(),
            icon_categories: BTreeMap::new(),
        }
    }

    pub fn save(&self, path: &PathBuf) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let project: IconProject = serde_json::from_str(&content)?;
        Ok(project)
    }
}

