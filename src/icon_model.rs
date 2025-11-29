use qmetaobject::{prelude::*, QAbstractListModel, QModelIndex, QVariant, QByteArray, QVariantMap};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use serde::{Deserialize, Serialize};

use crate::icon_catalog::IconCatalog;

#[derive(QObject, Default)]
pub struct IconModel {
    base: qt_base_class!(trait QAbstractListModel),
    catalog: qt_property!(QVariant; NOTIFY catalog_changed),
    catalog_changed: qt_signal!(),
    loading: qt_property!(bool; NOTIFY loading_changed),
    loading_changed: qt_signal!(),
    search_text: qt_property!(QString; NOTIFY search_text_changed),
    search_text_changed: qt_signal!(),
    category_filter: qt_property!(QString; NOTIFY category_filter_changed),
    category_filter_changed: qt_signal!(),
    _catalog_data: Arc<Mutex<Option<IconCatalog>>>,
    _all_icons_data: Arc<Mutex<Vec<IconItem>>>, // Full unfiltered list
    _icons_data: Arc<Mutex<Vec<IconItem>>>,
    _loading_flag: Arc<Mutex<bool>>,
    load_catalog: qt_method!(fn load_catalog(&mut self) {
        self.load_catalog_async();
    }),
    set_search_text: qt_method!(fn set_search_text(&mut self, text: String) {
        self.search_text = text.into();
        self.search_text_changed();
        self.apply_filters();
    }),
    set_category_filter: qt_method!(fn set_category_filter(&mut self, category: String) {
        self.category_filter = category.into();
        self.category_filter_changed();
        self.apply_filters();
    }),
    set_replacement: qt_method!(fn set_replacement(&mut self, icon_name: String, file_path: String) {
        self.set_replacement_internal(icon_name, file_path);
    }),
    clear_replacement: qt_method!(fn clear_replacement(&mut self, icon_name: String) {
        self.clear_replacement_internal(icon_name);
    }),
    get_replacement_path: qt_method!(fn get_replacement_path(&self, icon_name: String) -> QString {
        let icons = self._icons_data.lock().unwrap();
        icons.iter()
            .find(|i| i.name == icon_name)
            .and_then(|i| i.replacement_path.as_ref())
            .map(|s| s.clone())
            .unwrap_or_default()
            .into()
    }),
    update_icons_from_catalog: qt_method!(fn update_icons_from_catalog(&mut self) {
        self.update_icons_from_catalog_internal();
    }),
    
    sync_replacements_from_project: qt_method!(fn sync_replacements_from_project(&mut self, _replacements: QVariantMap) {
        log::info!("sync_replacements_from_project called (handled in QML)");
    }),
    
    get_replaced_icon_names: qt_method!(fn get_replaced_icon_names(&self) -> QString {
        let icons = self._all_icons_data.lock().unwrap();
        let names: Vec<String> = icons.iter()
            .filter(|icon| icon.replacement_path.is_some() && 
                    !icon.replacement_path.as_ref().unwrap().is_empty())
            .map(|icon| icon.name.clone())
            .collect();
        names.join(",").into()
    }),
    
    get_icon_category: qt_method!(fn get_icon_category(&self, icon_name: String) -> QString {
        let icons = self._all_icons_data.lock().unwrap();
        icons.iter()
            .find(|icon| icon.name == icon_name)
            .map(|icon| icon.category.clone())
            .unwrap_or_else(|| "Applications".to_string())
            .into()
    }),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct IconItem {
    name: String,
    category: String,
    has_svg: bool,
    has_png: bool,
    replacement_path: Option<String>,
}

impl QAbstractListModel for IconModel {
    fn row_count(&self) -> i32 {
        self._icons_data.lock().unwrap().len() as i32
    }

    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        let icons = self._icons_data.lock().unwrap();
        let row = index.row();
        if row < 0 || row >= icons.len() as i32 {
            return QVariant::default();
        }
        let icon = &icons[row as usize];
        match role {
            0 => QString::from(icon.name.as_str()).into(),
            1 => QString::from(icon.category.as_str()).into(),
            2 => icon.has_svg.into(),
            3 => icon.has_png.into(),
            4 => QString::from(icon.replacement_path.as_ref().map(|s| s.as_str()).unwrap_or("")).into(),
            _ => QVariant::default(),
        }
    }

    fn role_names(&self) -> HashMap<i32, QByteArray> {
        let mut hash = HashMap::new();
        hash.insert(0, "name".into());
        hash.insert(1, "category".into());
        hash.insert(2, "hasSvg".into());
        hash.insert(3, "hasPng".into());
        hash.insert(4, "replacementPath".into());
        hash
    }
}

impl IconModel {
    fn load_catalog_async(&mut self) {
        self.loading = true.into();
        self.loading_changed();
        *self._loading_flag.lock().unwrap() = true;
        
        let all_icons_data = Arc::clone(&self._all_icons_data);
        let catalog_data = Arc::clone(&self._catalog_data);
        let loading_flag = Arc::clone(&self._loading_flag);

        let cache_path = std::env::var("HOME")
            .map(|home| format!("{}/.local/share/icon-packer/catalog.json", home))
            .unwrap_or_else(|_| ".local/share/icon-packer/catalog.json".to_string());
        
        let cache_path_clone = cache_path.clone();
        thread::spawn(move || {
            if let Ok(cached) = std::fs::read_to_string(&cache_path_clone) {
                if let Ok(icons) = serde_json::from_str::<Vec<IconItem>>(&cached) {
                    log::info!("Loaded {} icons from cache", icons.len());
                    *all_icons_data.lock().unwrap() = icons.clone();
                    *loading_flag.lock().unwrap() = false;
                    log::info!("Cache load complete, loading flag set to false");
                    return;
                } else {
                    log::warn!("Failed to parse cached icons, will reload from disk");
                }
            }
            
            match IconCatalog::discover(None) {
                Ok(catalog) => {
                    *catalog_data.lock().unwrap() = Some(catalog.clone());
                    let mut icons = Vec::new();
                    for icon_meta in catalog.iter() {
                        let has_svg = icon_meta.variants.iter().any(|v| v.format == crate::icon_catalog::IconFormat::Svg);
                        let has_png = icon_meta.variants.iter().any(|v| v.format == crate::icon_catalog::IconFormat::Png);
                        let category = icon_meta.variants.first()
                            .map(|v| v.directory.context.clone())
                            .unwrap_or_else(|| "Generic".to_string());
                        icons.push(IconItem {
                            name: icon_meta.name.clone(),
                            category,
                            has_svg,
                            has_png,
                            replacement_path: None,
                        });
                    }
                    icons.sort_by(|a, b| a.name.cmp(&b.name));
                    
                    if let Ok(json) = serde_json::to_string_pretty(&icons) {
                        if let Some(parent) = std::path::Path::new(&cache_path_clone).parent() {
                            let _ = std::fs::create_dir_all(parent);
                            let _ = std::fs::write(&cache_path_clone, json);
                        }
                    }
                    
                    *all_icons_data.lock().unwrap() = icons.clone();
                    *loading_flag.lock().unwrap() = false;
                    log::info!("Icon discovery complete, {} icons loaded, loading flag set to false", icons.len());
                }
                Err(err) => {
                    log::error!("Failed to load icon catalog: {:?}", err);
                    *loading_flag.lock().unwrap() = false;
                }
            }
        });
    }
    
    fn apply_filters(&mut self) {
        let all_icons = self._all_icons_data.lock().unwrap().clone();
        let search = self.search_text.to_string().to_lowercase();
        let category = self.category_filter.to_string();
        
        let filtered: Vec<IconItem> = all_icons.into_iter()
            .filter(|icon| {
                let matches_search = search.is_empty() || icon.name.to_lowercase().contains(&search);
                let matches_category = category.is_empty() || category == "All Categories" || icon.category == category;
                matches_search && matches_category
            })
            .collect();
        
        let mut data = self._icons_data.lock().unwrap();
        let old_len = data.len();
        let new_len = filtered.len();
        *data = filtered;
        drop(data);
        
        if old_len == 0 && new_len > 0 {
            self.begin_insert_rows(0, (new_len - 1) as i32);
            self.end_insert_rows();
            log::info!("Model notified: inserted {} rows", new_len);
        } else if old_len != new_len {
            self.begin_reset_model();
            self.end_reset_model();
            log::info!("Model reset: {} -> {} rows", old_len, new_len);
        } else if new_len > 0 {
            let start_index = self.row_index(0);
            let end_index = self.row_index((new_len - 1) as i32);
            self.data_changed(start_index, end_index);
        }
    }
    
    fn update_icons_from_catalog_internal(&mut self) {
        let is_loading = *self._loading_flag.lock().unwrap();
        if !is_loading {
            let all_icons_len = self._all_icons_data.lock().unwrap().len();
            let currently_loading: bool = self.loading.into();
            if currently_loading {
                if all_icons_len > 0 {
                    self.apply_filters();
                }
                self.loading = false.into();
                self.loading_changed();
                self.catalog_changed();
                log::info!("Icon loading complete, {} icons available", all_icons_len);
            }
        }
    }

    fn set_replacement_internal(&mut self, icon_name: String, file_path: String) {
        let mut all_icons = self._all_icons_data.lock().unwrap();
        if let Some(icon) = all_icons.iter_mut().find(|i| i.name == icon_name) {
            icon.replacement_path = Some(file_path.clone());
        }
        drop(all_icons);
        
        let mut icons = self._icons_data.lock().unwrap();
        if let Some(icon) = icons.iter_mut().find(|i| i.name == icon_name) {
            icon.replacement_path = Some(file_path);
            let row = icons.iter()
                .position(|i| i.name == icon_name)
                .map(|i| i as i32)
                .unwrap_or(-1);
            drop(icons);
            if row >= 0 {
                let index = self.row_index(row);
                self.data_changed(index, index);
            }
        }
    }

    fn clear_replacement_internal(&mut self, icon_name: String) {
        let mut icons = self._icons_data.lock().unwrap();
        if let Some(icon) = icons.iter_mut().find(|i| i.name == icon_name) {
            icon.replacement_path = None;
            let row = icons.iter()
                .position(|i| i.name == icon_name)
                .map(|i| i as i32)
                .unwrap_or(-1);
            drop(icons);
            if row >= 0 {
                let index = self.row_index(row);
                self.data_changed(index, index);
            }
        }
    }
}

