use qmetaobject::{prelude::*, QVariantMap, QString};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::project::IconProject;
use crate::theme_generator::ThemePackGenerator;

#[derive(QObject, Default)]
pub struct ProjectManager {
    base: qt_base_class!(trait QObject),
    current_project: qt_property!(QVariant; NOTIFY current_project_changed),
    current_project_changed: qt_signal!(),
    project_name: qt_property!(QString; NOTIFY project_name_changed),
    project_name_changed: qt_signal!(),
    has_project: qt_property!(bool; NOTIFY has_project_changed),
    has_project_changed: qt_signal!(),
    _project: Arc<Mutex<Option<IconProject>>>,
    
    new_project: qt_method!(fn new_project(&mut self, name: String, output_path: String) {
        let mut project = IconProject::new(name.clone());
        project.theme_name = name.clone();
        
        let base_path = PathBuf::from(output_path.clone());
        let theme_folder = base_path.join(&name);
        project.output_path = Some(theme_folder.clone());
        
        *self._project.lock().unwrap() = Some(project);
        self.project_name = name.into();
        self.has_project = true.into();
        self.project_name_changed();
        self.has_project_changed();
        self.current_project_changed();
        self.generate_theme_live();
    }),
    
    load_project: qt_method!(fn load_project(&mut self, theme_path: String) -> bool {
        let theme_path_buf = PathBuf::from(theme_path.clone());
        
        if !theme_path_buf.is_dir() {
            log::error!("Path is not a directory: {:?}", theme_path_buf);
            return false;
        }
        
        let metadata_path = theme_path_buf.join(".icon-packer-project.json");
        
        if !metadata_path.exists() {
            log::warn!("Metadata file not found at {:?}, creating new project from theme folder", metadata_path);
            let theme_name = theme_path_buf.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Untitled Theme")
                .to_string();
            let mut project = IconProject::new(theme_name.clone());
            project.theme_name = theme_name.clone();
            project.output_path = Some(theme_path_buf.clone());
            *self._project.lock().unwrap() = Some(project.clone());
            self.project_name = theme_name.into();
            self.has_project = true.into();
            self.project_name_changed();
            self.has_project_changed();
            self.current_project_changed();
            if let Err(e) = project.save(&metadata_path) {
                log::warn!("Failed to save metadata file: {:?}", e);
            }
            return true;
        }
        
        match IconProject::load(&metadata_path) {
            Ok(mut project) => {
                project.output_path = Some(theme_path_buf.clone());
                *self._project.lock().unwrap() = Some(project.clone());
                self.project_name = project.name.clone().into();
                self.has_project = true.into();
                self.project_name_changed();
                self.has_project_changed();
                self.current_project_changed();
                true
            }
            Err(e) => {
                log::error!("Failed to load project: {:?}", e);
                false
            }
        }
    }),
    
    add_replacement: qt_method!(fn add_replacement(&mut self, icon_name: String, file_path: String) {
        let needs_generate = {
            let mut project = self._project.lock().unwrap();
            if let Some(ref mut proj) = *project {
                if file_path.is_empty() {
                    proj.icon_replacements.remove(&icon_name);
                    proj.icon_links.remove(&icon_name);
                    proj.icon_categories.remove(&icon_name);
                } else {
                    proj.icon_replacements.insert(icon_name.clone(), PathBuf::from(file_path));
                }
                true
            } else {
                false
            }
        };
        self.current_project_changed();
        if needs_generate {
            self.generate_theme_live();
        }
    }),
    
    set_icon_category: qt_method!(fn set_icon_category(&mut self, icon_name: String, category: String) {
        let mut project = self._project.lock().unwrap();
        if let Some(ref mut proj) = *project {
            proj.icon_categories.insert(icon_name, category);
        }
    }),
    
    set_replacement_link: qt_method!(fn set_replacement_link(&mut self, icon_name: String, is_link: bool) {
        let needs_generate = {
            let mut project = self._project.lock().unwrap();
            if let Some(ref mut proj) = *project {
                if proj.icon_replacements.contains_key(&icon_name) {
                    proj.icon_links.insert(icon_name, is_link);
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };
        self.current_project_changed();
        if needs_generate {
            self.generate_theme_live();
        }
    }),
    
    set_size_replacement_link: qt_method!(fn set_size_replacement_link(&mut self, icon_name: String, size: u32, is_link: bool) {
        let needs_generate = {
            let mut project = self._project.lock().unwrap();
            if let Some(ref mut proj) = *project {
                if let Some(size_map) = proj.size_specific_replacements.get(&icon_name) {
                    if size_map.contains_key(&size) {
                        proj.size_specific_links
                            .entry(icon_name)
                            .or_insert_with(BTreeMap::new)
                            .insert(size, is_link);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        };
        self.current_project_changed();
        if needs_generate {
            self.generate_theme_live();
        }
    }),
    
    save_recent_project: qt_method!(fn save_recent_project(&self, theme_path: String) {
        let recent_path = std::env::var("HOME")
            .map(|home| format!("{}/.local/share/icon-packer/recent-projects.json", home))
            .unwrap_or_else(|_| ".local/share/icon-packer/recent-projects.json".to_string());
        
        let path_buf = PathBuf::from(recent_path.clone());
        if let Some(parent) = path_buf.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        
        let mut recent: Vec<String> = Vec::new();
        if let Ok(content) = std::fs::read_to_string(&path_buf) {
            if let Ok(parsed) = serde_json::from_str::<Vec<String>>(&content) {
                recent = parsed;
            }
        }
        
        recent.retain(|p| p != &theme_path);
        recent.insert(0, theme_path);
        recent.truncate(10);
        
        if let Ok(json) = serde_json::to_string_pretty(&recent) {
            let _ = std::fs::write(&path_buf, json);
        }
    }),
    
    get_recent_projects: qt_method!(fn get_recent_projects(&self) -> QString {
        let recent_path = std::env::var("HOME")
            .map(|home| format!("{}/.local/share/icon-packer/recent-projects.json", home))
            .unwrap_or_else(|_| ".local/share/icon-packer/recent-projects.json".to_string());
        
        if let Ok(content) = std::fs::read_to_string(&recent_path) {
            if let Ok(recent) = serde_json::from_str::<Vec<String>>(&content) {
                return recent.join("\n").into();
            }
        }
        QString::default()
    }),
    
    get_replacements: qt_method!(fn get_replacements(&self) -> QVariantMap {
        let project = self._project.lock().unwrap();
        let mut map = QVariantMap::default();
        if let Some(ref proj) = *project {
            for (name, path) in &proj.icon_replacements {
                map.insert(QString::from(name.as_str()).into(), QString::from(path.to_string_lossy().as_ref()).into());
            }
        }
        map
    }),
    
    get_replaced_icon_names: qt_method!(fn get_replaced_icon_names(&self) -> QString {
        let project = self._project.lock().unwrap();
        if let Some(ref proj) = *project {
            let names: Vec<String> = proj.icon_replacements.iter()
                .filter(|(_, path)| !path.to_string_lossy().is_empty())
                .map(|(name, _)| name.clone())
                .collect();
            names.join(",").into()
        } else {
            QString::default()
        }
    }),
    
    get_fallback_themes: qt_method!(fn get_fallback_themes(&self) -> QString {
        let project = self._project.lock().unwrap();
        if let Some(ref proj) = *project {
            proj.fallback_themes.join(",").into()
        } else {
            QString::default()
        }
    }),
    
    set_fallback_themes: qt_method!(fn set_fallback_themes(&mut self, themes: QString) {
        let needs_autosave = {
            let mut project = self._project.lock().unwrap();
            if let Some(ref mut proj) = *project {
                proj.fallback_themes = themes.to_string()
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                true
            } else {
                false
            }
        };
        self.current_project_changed();
        if needs_autosave {
            self.generate_theme_live();
        }
    }),
    
    get_theme_name: qt_method!(fn get_theme_name(&self) -> QString {
        let project = self._project.lock().unwrap();
        if let Some(ref proj) = *project {
            if !proj.theme_name.is_empty() {
                proj.theme_name.clone().into()
            } else {
                proj.name.clone().into()
            }
        } else {
            QString::default()
        }
    }),
    
    set_theme_name: qt_method!(fn set_theme_name(&mut self, name: String) {
        let needs_autosave = {
            let mut project = self._project.lock().unwrap();
            if let Some(ref mut proj) = *project {
                proj.theme_name = name;
                true
            } else {
                false
            }
        };
        self.current_project_changed();
        if needs_autosave {
            self.generate_theme_live();
        }
    }),
    
    get_theme_comment: qt_method!(fn get_theme_comment(&self) -> QString {
        let project = self._project.lock().unwrap();
        if let Some(ref proj) = *project {
            if !proj.theme_comment.is_empty() {
                proj.theme_comment.clone().into()
            } else {
                "Icon theme generated by icon-packer".to_string().into()
            }
        } else {
            QString::default()
        }
    }),
    
    set_theme_comment: qt_method!(fn set_theme_comment(&mut self, comment: String) {
        let needs_autosave = {
            let mut project = self._project.lock().unwrap();
            if let Some(ref mut proj) = *project {
                proj.theme_comment = comment;
                true
            } else {
                false
            }
        };
        self.current_project_changed();
        if needs_autosave {
            self.generate_theme_live();
        }
    }),
    
    add_size_replacement: qt_method!(fn add_size_replacement(&mut self, icon_name: String, size: u32, file_path: String) {
        let needs_generate = {
            let mut project = self._project.lock().unwrap();
            if let Some(ref mut proj) = *project {
                let size_map = proj.size_specific_replacements
                    .entry(icon_name.clone())
                    .or_insert_with(BTreeMap::new);
                if file_path.is_empty() {
                    size_map.remove(&size);
                    if size_map.is_empty() {
                        proj.size_specific_replacements.remove(&icon_name);
                    }
                } else {
                    size_map.insert(size, PathBuf::from(file_path));
                }
                true
            } else {
                false
            }
        };
        self.current_project_changed();
        if needs_generate {
            self.generate_theme_live();
        }
    }),
    
    get_size_replacements: qt_method!(fn get_size_replacements(&self, icon_name: String) -> QVariantMap {
        let project = self._project.lock().unwrap();
        let mut map = QVariantMap::default();
        if let Some(ref proj) = *project {
            if let Some(size_map) = proj.size_specific_replacements.get(&icon_name) {
                for (size, path) in size_map {
                    map.insert(
                        QString::from(size.to_string().as_str()).into(),
                        QString::from(path.to_string_lossy().as_ref()).into()
                    );
                }
            }
        }
        map
    }),
    
    generate_theme: qt_method!(fn generate_theme(&self, theme_name: String, output_path: String) -> bool {
        let project = self._project.lock().unwrap();
        if let Some(ref proj) = *project {
            let output_dir = PathBuf::from(output_path.clone());
            let mut generator = ThemePackGenerator::new(theme_name.clone(), output_dir);
            
            let theme_comment = if !proj.theme_comment.is_empty() {
                proj.theme_comment.clone()
            } else {
                "Icon theme generated by icon-packer".to_string()
            };
            generator.set_theme_comment(theme_comment);
            
            generator.set_fallback_themes(proj.fallback_themes.clone());
            
            for (icon_name, path) in &proj.icon_replacements {
                generator.add_replacement(icon_name.clone(), path.clone());
            }
            
            for (icon_name, size_map) in &proj.size_specific_replacements {
                for (size, path) in size_map {
                    generator.add_size_replacement(icon_name.clone(), *size, path.clone());
                }
            }
            
            match generator.generate() {
                Ok(_) => {
                    log::info!("Theme generated successfully to: {}", output_path);
                    true
                }
                Err(e) => {
                    log::error!("Failed to generate theme: {:?}", e);
                    false
                }
            }
        } else {
            false
        }
    }),
    
    
    generate_theme_live: qt_method!(fn generate_theme_live(&mut self) {
        let (output_path, theme_name, fallback_themes, icon_replacements, size_specific_replacements, icon_links, size_specific_links, icon_categories, project_clone) = {
            let project = self._project.lock().unwrap();
            if let Some(ref proj) = *project {
                if let Some(ref output_path) = proj.output_path {
                    let theme_name = if !proj.theme_name.is_empty() {
                        proj.theme_name.clone()
                    } else {
                        proj.name.clone()
                    };
                    (
                        Some(output_path.clone()),
                        theme_name,
                        proj.fallback_themes.clone(),
                        proj.icon_replacements.clone(),
                        proj.size_specific_replacements.clone(),
                        proj.icon_links.clone(),
                        proj.size_specific_links.clone(),
                        proj.icon_categories.clone(),
                        Some(proj.clone()),
                    )
                } else {
                    (None, String::new(), Vec::new(), BTreeMap::new(), BTreeMap::new(), BTreeMap::new(), BTreeMap::new(), BTreeMap::new(), None)
                }
            } else {
                (None, String::new(), Vec::new(), BTreeMap::new(), BTreeMap::new(), BTreeMap::new(), BTreeMap::new(), BTreeMap::new(), None)
            }
        };
        
        if let Some(output_path) = output_path {
            let mut generator = ThemePackGenerator::new(theme_name.clone(), output_path.clone());
            generator.set_fallback_themes(fallback_themes.clone());
            
            for (icon_name, path) in icon_replacements {
                generator.add_replacement(icon_name.clone(), path);
                if let Some(&is_link) = icon_links.get(&icon_name) {
                    generator.set_replacement_link(icon_name.clone(), is_link);
                }
                let category = icon_categories.get(&icon_name)
                    .cloned()
                    .unwrap_or_else(|| "Applications".to_string());
                generator.set_icon_category(icon_name.clone(), category);
            }
            
            for (icon_name, size_map) in size_specific_replacements {
                let category = icon_categories.get(&icon_name)
                    .cloned()
                    .unwrap_or_else(|| "Applications".to_string());
                generator.set_icon_category(icon_name.clone(), category);
                for (size, path) in size_map {
                    generator.add_size_replacement(icon_name.clone(), size, path);
                    if let Some(link_map) = size_specific_links.get(&icon_name) {
                        if let Some(&is_link) = link_map.get(&size) {
                            generator.set_size_replacement_link(icon_name.clone(), size, is_link);
                        }
                    }
                }
            }
            
            if let Err(e) = generator.generate() {
                log::error!("Failed to generate theme live: {:?}", e);
            } else {
                log::debug!("Theme generated live to: {}", output_path.display());
            }
            
            if let Some(ref proj) = project_clone {
                let metadata_path = output_path.join(".icon-packer-project.json");
                if let Err(e) = proj.save(&metadata_path) {
                    log::warn!("Failed to auto-save project metadata: {:?}", e);
                } else {
                    log::debug!("Project metadata auto-saved to: {:?}", metadata_path);
                }
            }
        }
    }),
}

