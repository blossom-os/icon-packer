use std::collections::{BTreeMap, HashSet};
use std::ffi::OsStr;
use std::path::PathBuf;

use anyhow::{Context, Result};
use walkdir::WalkDir;

use crate::icon_theme::{IconThemeDefinition, ThemeDirectory};

#[derive(Debug, Clone)]
pub struct IconCatalog {
    icons: BTreeMap<String, IconMetadata>,
}

#[derive(Debug, Clone)]
pub struct IconMetadata {
    pub name: String,
    pub variants: Vec<IconVariant>,
}

#[derive(Debug, Clone)]
pub struct IconVariant {
    #[allow(dead_code)]
    pub theme_name: String,
    pub directory: ThemeDirectory,
    #[allow(dead_code)]
    pub path: PathBuf,
    pub format: IconFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconFormat {
    Png,
    Svg,
    Xpm,
    Other,
}

impl IconCatalog {
    pub fn discover(theme_hint: Option<&str>) -> Result<Self> {
        let mut icons: BTreeMap<String, IconMetadata> = BTreeMap::new();
        let roots = icon_base_dirs();
        let mut seen_themes: HashSet<PathBuf> = HashSet::new();

        for root in roots {
            if !root.exists() {
                continue;
            }
            for entry in
                std::fs::read_dir(&root).with_context(|| format!("Reading {}", root.display()))?
            {
                let entry = entry?;
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                if !seen_themes.insert(path.clone()) {
                    continue;
                }
                match IconThemeDefinition::load_from_directory(&path) {
                    Ok(theme) => {
                        if let Some(filter) = theme_hint {
                            let theme_name_lower = theme.name.to_lowercase();
                            if theme_name_lower != filter.to_lowercase()
                                && theme.directory_name.to_lowercase() != filter.to_lowercase()
                            {
                                continue;
                            }
                        }
                        scan_theme(&theme, &mut icons)?;
                    }
                    Err(err) => {
                        log::debug!("Skipping theme in {}: {}", path.display(), err);
                    }
                }
            }
        }

        Ok(Self { icons })
    }


    pub fn iter(&self) -> impl Iterator<Item = &IconMetadata> {
        self.icons.values()
    }
}

fn scan_theme(
    theme: &IconThemeDefinition,
    catalog: &mut BTreeMap<String, IconMetadata>,
) -> Result<()> {
    for directory in &theme.directories {
        let dir_path = theme.root_path.join(&directory.key);
        if !dir_path.exists() {
            continue;
        }
        for entry in WalkDir::new(&dir_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|res| res.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.into_path();
            let Some(stem) = path.file_stem().and_then(OsStr::to_str) else {
                continue;
            };
            if stem.is_empty() {
                continue;
            }
            let format = IconFormat::from_extension(path.extension());
            if format == IconFormat::Other {
                continue;
            }
            let meta = catalog
                .entry(stem.to_string())
                .or_insert_with(|| IconMetadata {
                    name: stem.to_string(),
                    variants: Vec::new(),
                });
            meta.variants.push(IconVariant {
                theme_name: theme.name.clone(),
                directory: directory.clone(),
                path: path.clone(),
                format,
            });
        }
    }

    Ok(())
}

fn icon_base_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let mut push_unique = |path: PathBuf| {
        if path.exists() && !dirs.contains(&path) {
            dirs.push(path);
        }
    };

    if let Some(home) = home_dir() {
        push_unique(home.join(".local/share/icons"));
        push_unique(home.join(".icons"));
    }

    if let Ok(xdg_data_home) = std::env::var("XDG_DATA_HOME") {
        push_unique(PathBuf::from(xdg_data_home).join("icons"));
    }

    if let Ok(xdg_data_dirs) = std::env::var("XDG_DATA_DIRS") {
        for dir in xdg_data_dirs.split(':') {
            if dir.is_empty() {
                continue;
            }
            push_unique(PathBuf::from(dir).join("icons"));
        }
    } else {
        push_unique(PathBuf::from("/usr/local/share/icons"));
        push_unique(PathBuf::from("/usr/share/icons"));
    }

    push_unique(PathBuf::from("/usr/share/pixmaps"));

    dirs
}

fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME").map(PathBuf::from).ok()
}

impl IconFormat {
    fn from_extension(ext: Option<&OsStr>) -> Self {
        match ext.and_then(OsStr::to_str).map(|s| s.to_lowercase()) {
            Some(ref e) if e == "png" => IconFormat::Png,
            Some(ref e) if e == "svg" => IconFormat::Svg,
            Some(ref e) if e == "xpm" => IconFormat::Xpm,
            _ => IconFormat::Other,
        }
    }
}
