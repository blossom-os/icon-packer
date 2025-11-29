use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use configparser::ini::Ini;

#[derive(Debug, Clone)]
pub struct IconThemeDefinition {
    pub directory_name: String,
    pub name: String,
    pub inherits: Vec<String>,
    pub directories: Vec<ThemeDirectory>,
    pub root_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ThemeDirectory {
    pub key: String,
    pub context: String,
    pub dir_type: DirectoryType,
    pub size: Option<u32>,
    pub min_size: Option<u32>,
    pub max_size: Option<u32>,
    pub threshold: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectoryType {
    Fixed,
    Scaled,
    Threshold,
}

impl IconThemeDefinition {
    pub fn load_from_directory(theme_dir: &Path) -> Result<Self> {
        let index_path = theme_dir.join("index.theme");
        if !index_path.exists() {
            anyhow::bail!("Missing index.theme in {}", theme_dir.to_string_lossy());
        }
        let mut conf = Ini::new();
        conf.load(index_path.to_str().unwrap())
            .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", index_path.display(), e))
            .with_context(|| format!("Failed to read {}", index_path.display()))?;
        
        let icon_theme_section = "Icon Theme";
        let directories = parse_directory_list(conf.get(icon_theme_section, "Directories").as_ref());
        let scaled_directories = parse_directory_list(conf.get(icon_theme_section, "ScaledDirectories").as_ref());
        let all_directories: Vec<String> = directories
            .into_iter()
            .chain(scaled_directories.into_iter())
            .collect();

        let dir_defs = all_directories
            .into_iter()
            .filter_map(|dir_name| parse_directory_section(&conf, &dir_name))
            .collect::<Vec<_>>();

        Ok(Self {
            directory_name: theme_dir
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| String::from("<unknown>")),
            name: conf.get(icon_theme_section, "Name")
                .unwrap_or_else(|| {
                    theme_dir
                        .file_name()
                        .map(|f| f.to_string_lossy().to_string())
                        .unwrap_or_else(|| String::from("Unnamed Theme"))
                }),
            inherits: parse_directory_list_from_opt(conf.get(icon_theme_section, "Inherits")),
            directories: dir_defs,
            root_path: theme_dir.to_path_buf(),
        })
    }
}

fn parse_directory_list(input: Option<&String>) -> Vec<String> {
    input
        .map(|value| {
            value
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn parse_directory_list_from_opt(input: Option<String>) -> Vec<String> {
    input
        .as_ref()
        .map(|value| {
            value
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn parse_directory_section(conf: &Ini, dir_name: &str) -> Option<ThemeDirectory> {
    let dir_type_str = conf.get(dir_name, "Type");
    let dir_type = dir_type_str
        .as_ref()
        .map(|v| DirectoryType::from_str(v))
        .unwrap_or(DirectoryType::Threshold);
    let context = conf.get(dir_name, "Context")
        .unwrap_or_else(|| String::from("Generic"));
    Some(ThemeDirectory {
        key: dir_name.to_string(),
        context,
        dir_type,
        size: conf.get(dir_name, "Size").and_then(|v| v.parse::<u32>().ok()),
        min_size: conf.get(dir_name, "MinSize").and_then(|v| v.parse::<u32>().ok()),
        max_size: conf.get(dir_name, "MaxSize").and_then(|v| v.parse::<u32>().ok()),
        threshold: conf.get(dir_name, "Threshold").and_then(|v| v.parse::<u32>().ok()),
    })
}

impl DirectoryType {
    fn from_str(input: &str) -> Self {
        match input {
            "Fixed" => DirectoryType::Fixed,
            "Scaled" => DirectoryType::Scaled,
            _ => DirectoryType::Threshold,
        }
    }
}
