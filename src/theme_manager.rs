use std::path::PathBuf;
use qmetaobject::{prelude::*, QString, QVariantMap};
use crate::icon_theme::IconThemeDefinition;

#[derive(QObject, Default)]
pub struct ThemeManager {
    base: qt_base_class!(trait QObject),
    available_themes: qt_property!(QVariant; NOTIFY available_themes_changed),
    available_themes_changed: qt_signal!(),
    _themes: std::sync::Arc<std::sync::Mutex<Vec<ThemeInfo>>>,
    
    discover_themes: qt_method!(fn discover_themes(&mut self) {
        self.discover_themes_internal();
    }),
    
    get_theme_names: qt_method!(fn get_theme_names(&self) -> QString {
        let themes = self._themes.lock().unwrap();
        let names: Vec<String> = themes.iter().map(|t| t.name.clone()).collect();
        names.join(",").into()
    }),
    
    get_icon_path: qt_method!(fn get_icon_path(&self, theme_name: String, icon_name: String, size: u32) -> QString {
        let themes = self._themes.lock().unwrap();
        if let Some(theme) = themes.iter().find(|t| t.name == theme_name || t.directory_name == theme_name) {
            // Try to find the icon in the theme by scanning directories
            match crate::icon_theme::IconThemeDefinition::load_from_directory(&theme.path) {
                Ok(theme_def) => {
                    // Search through theme directories for the icon
                    for directory in &theme_def.directories {
                        let dir_path = theme_def.root_path.join(&directory.key);
                        if !dir_path.exists() {
                            continue;
                        }
                        // Try different extensions
                        for ext in &["svg", "png", "xpm"] {
                            let icon_path = dir_path.join(format!("{}.{}", icon_name, ext));
                            if icon_path.exists() {
                                // Check if size matches (for fixed/sized directories)
                                let matches_size = match directory.dir_type {
                                    crate::icon_theme::DirectoryType::Fixed => {
                                        directory.size.map(|s| s == size).unwrap_or(true)
                                    }
                                    crate::icon_theme::DirectoryType::Scaled => {
                                        directory.min_size.map(|min| size >= min).unwrap_or(true)
                                            && directory.max_size.map(|max| size <= max).unwrap_or(true)
                                    }
                                    crate::icon_theme::DirectoryType::Threshold => {
                                        directory.size.map(|s| (size as i32 - s as i32).abs() <= directory.threshold.unwrap_or(2) as i32).unwrap_or(true)
                                    }
                                };
                                if matches_size {
                                    return QString::from(icon_path.to_string_lossy().as_ref());
                                }
                            }
                        }
                    }
                }
                Err(_) => {}
            }
        }
        QString::default()
    }),
}

#[derive(Clone, Debug)]
struct ThemeInfo {
    name: String,
    directory_name: String,
    path: PathBuf,
    #[allow(dead_code)]
    inherits: Vec<String>,
}

impl ThemeManager {
    fn discover_themes_internal(&mut self) {
        let mut themes = Vec::new();
        let roots = icon_base_dirs();
        
        for root in roots {
            if !root.exists() {
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(&root) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    match IconThemeDefinition::load_from_directory(&path) {
                        Ok(theme) => {
                            themes.push(ThemeInfo {
                                name: theme.name.clone(),
                                directory_name: theme.directory_name.clone(),
                                path: theme.root_path.clone(),
                                inherits: theme.inherits.clone(),
                            });
                        }
                        Err(_) => {
                            // Skip invalid themes
                        }
                    }
                }
            }
        }
        
        themes.sort_by(|a, b| a.name.cmp(&b.name));
        *self._themes.lock().unwrap() = themes.clone();
        
        let mut map = QVariantMap::default();
        for theme in themes {
            map.insert(
                QString::from(theme.name.as_str()).into(),
                QString::from(theme.directory_name.as_str()).into()
            );
        }
        self.available_themes = map.into();
        self.available_themes_changed();
    }
}

fn icon_base_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    
    // User's local icons
    if let Ok(home) = std::env::var("HOME") {
        let home_path = PathBuf::from(&home);
        dirs.push(home_path.join(".icons"));
        dirs.push(PathBuf::from(home).join(".local/share/icons"));
    }
    
    // System-wide icons
    if let Ok(xdg_dirs) = std::env::var("XDG_DATA_DIRS") {
        for dir in xdg_dirs.split(':') {
            dirs.push(PathBuf::from(dir).join("icons"));
        }
    }
    
    // Fallback system paths
    dirs.push(PathBuf::from("/usr/share/icons"));
    dirs.push(PathBuf::from("/usr/local/share/icons"));
    
    dirs
}

