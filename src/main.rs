mod icon_catalog;
mod icon_model;
mod icon_theme;
mod project;
mod project_manager;
mod theme_generator;
mod theme_manager;

use cstr::cstr;
use icon_model::IconModel;
use project_manager::ProjectManager;
use qmetaobject::{prelude::*, QUrl};
use theme_manager::ThemeManager;

qrc!(root_qml,
    "" {
        "qml/main.qml" as "main.qml",
        "qml/welcome_page.qml" as "welcome_page.qml",
        "qml/main_page.qml" as "main_page.qml",
        "qml/icon_details_sheet.qml" as "icon_details_sheet.qml",
        "qml/new_project_dialog.qml" as "new_project_dialog.qml",
        "qml/about_sheet.qml" as "about_sheet.qml",
        "qml/project_settings_dialog.qml" as "project_settings_dialog.qml",
    }
);

fn main() {
    std::env::set_var(
        "QT_LOGGING_RULES",
        "*.debug=false;qml.debug=false;*.warning=true;*.critical=true",
    );

    qmetaobject::log::init_qt_to_rust();
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    log::info!("Starting Icon Packer Application");

    qml_register_type::<IconModel>(cstr!("IconModel"), 1, 0, cstr!("IconModel"));
    qml_register_type::<ProjectManager>(cstr!("ProjectManager"), 1, 0, cstr!("ProjectManager"));
    qml_register_type::<ThemeManager>(cstr!("ThemeManager"), 1, 0, cstr!("ThemeManager"));

    root_qml();

    let mut engine = QmlEngine::new();

    log::info!("Loading QML from qrc:///main.qml");
    engine.load_url(QUrl::from(QString::from("qrc:///main.qml")));

    use qmetaobject::QByteArray;
    engine.invoke_method_noreturn(QByteArray::from("show"), &[]);
    engine.invoke_method_noreturn(QByteArray::from("raise"), &[]);
    engine.invoke_method_noreturn(QByteArray::from("requestActivate"), &[]);

    engine.exec();
}
