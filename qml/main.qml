import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.platform 1.1 as Platform
import org.kde.kirigami 2.19 as Kirigami
import IconModel 1.0
import ProjectManager 1.0
import ThemeManager 1.0

Kirigami.ApplicationWindow {
    id: root
    visible: true
    title: "Icon Packer"
    width: 1400
    height: 900

    menuBar: MenuBar {
        Menu {
            title: "File"
            Action {
                text: "New Project"
                icon.name: "document-new"
                shortcut: "Ctrl+N"
                onTriggered: openNewProjectDialog()
            }
            Action {
                text: "Open Project"
                icon.name: "document-open"
                shortcut: "Ctrl+O"
                onTriggered: openProjectDialog.open()
            }
            MenuSeparator {}
            Action {
                text: "Quit"
                icon.name: "application-exit"
                shortcut: "Ctrl+Q"
                onTriggered: Qt.quit()
            }
        }
        Menu {
            id: editMenu
            title: "Edit"
            enabled: projectManager !== null && projectManager.hasProject === true
            Action {
                text: "Project Settings"
                icon.name: "configure"
                enabled: projectManager !== null && projectManager.hasProject === true
                onTriggered: openProjectSettingsDialog()
            }
            Action {
                text: "Load Icons"
                icon.name: "view-refresh"
                enabled: projectManager !== null && projectManager.hasProject === true
                onTriggered: iconModel.load_catalog()
            }
        }
        Menu {
            title: "Help"
            Action {
                text: "About"
                icon.name: "help-about"
                onTriggered: aboutSheet.open()
            }
        }
    }

    ProjectManager {
        id: projectManager
    }

    IconModel {
        id: iconModel
        Component.onCompleted: {
            load_catalog()
        }
    }

    ThemeManager {
        id: themeManager
        Component.onCompleted: {
            discover_themes()
        }
    }

    Timer {
        id: loadingTimer
        interval: 50
        running: iconModel.loading
        repeat: true
        onTriggered: {
            iconModel.update_icons_from_catalog()
        }
    }

    Component {
        id: welcomePageComponent
        Loader {
            id: welcomeLoader
            source: "qrc:///welcome_page.qml"
            asynchronous: false

            onItemChanged: {
                if (item) {
                    Qt.callLater(function() {
                        if (item && rootWindow && rootWindow.projectManager) {
                            item.projectManager = rootWindow.projectManager
                            item.newProjectRequested.connect(function() {
                                rootWindow.openNewProjectDialog()
                            })
                            item.openProjectRequested.connect(function() {
                                rootWindow.openProjectDialog.open()
                            })
                            item.recentProjectRequested.connect(function(path) {
                                if (rootWindow.projectManager.load_project(path)) {
                                    rootWindow.projectManager.save_recent_project(path)
                                    var replacements = rootWindow.projectManager.get_replacements()
                                    for (var iconName in replacements) {
                                        var path = replacements[iconName]
                                        if (path && path !== "" && path !== "null" && path !== "undefined") {
                                            rootWindow.iconModel.set_replacement(iconName, path)
                                        }
                                    }
                                    rootWindow.switchToMainPage()
                                }
                            })
                        }
                    })
                }
            }
        }
    }

    Component {
        id: mainPageComponent
        Loader {
            source: "qrc:///main_page.qml"
            asynchronous: false
            onItemChanged: {
                if (item) {
                    Qt.callLater(function() {
                        if (item && root && root.iconModel) {
                            item.iconModel = root.iconModel
                            item.projectManager = root.projectManager
                            item.onIconClicked.connect(function(iconName) {
                                root.openIconDetailsSheet(iconName)
                            })
                        }
                    })
                }
            }
        }
    }

    property var iconDetailsSheetInstance: null

    function openIconDetailsSheet(iconName) {
        if (!iconDetailsSheetInstance) {
            iconDetailsSheetInstance = iconDetailsSheetComponent.createObject(root)
            iconDetailsSheetInstance.iconModel = iconModel
            iconDetailsSheetInstance.projectManager = projectManager
            iconDetailsSheetInstance.themeManager = themeManager
            iconDetailsSheetInstance.onReplacementChanged = function(action, name) {
                if (action === "open") {
                    fileDialog.open()
                }
            }
        }
        iconDetailsSheetInstance.iconName = iconName
        iconDetailsSheetInstance.open()
    }

    Component {
        id: iconDetailsSheetComponent
        Loader {
            source: "qrc:///icon_details_sheet.qml"
        }
    }

    function switchToMainPage() {
        if (pageStack.depth > 0) {
            pageStack.replace(mainPageComponent)
        } else {
            pageStack.push(mainPageComponent)
        }
        Qt.callLater(function() {
            if (pageStack && pageStack.currentItem && pageStack.currentItem.item) {
                pageStack.currentItem.item.iconModel = iconModel
                pageStack.currentItem.item.projectManager = projectManager
            }
        })
    }

    property var newProjectDialog: null

    function openNewProjectDialog() {
        openNewProjectDialogInternal()
    }

    function openNewProjectDialogInternal() {
        if (!newProjectDialog) {
            newProjectDialog = newProjectDialogComponent.createObject(root)
        }

        function setupAndOpenDialog() {
            if (newProjectDialog && newProjectDialog.item) {
                if (!newProjectDialog.item.onProjectAccepted) {
                    newProjectDialog.item.onProjectAccepted = function(name) {
                        projectManager.new_project(name)
                        switchToMainPage()
                    }
                }
                newProjectDialog.item.open()
            }
        }

        if (newProjectDialog.item) {
            setupAndOpenDialog()
        } else {
            var checkLoaded = function() {
                if (newProjectDialog.item) {
                    setupAndOpenDialog()
                } else {
                    Qt.callLater(checkLoaded)
                }
            }
            Qt.callLater(checkLoaded)
        }
    }


    Component {
        id: newProjectDialogComponent
        Loader {
            source: "qrc:///new_project_dialog.qml"
            asynchronous: false
            onItemChanged: {
                if (item) {
                    item.onProjectAccepted = function(name, outputPath) {
                        projectManager.new_project(name, outputPath)
                        var themeFolder = outputPath + "/" + name
                        projectManager.save_recent_project(themeFolder)
                        switchToMainPage()
                    }
                }
            }
        }
    }

    Platform.FileDialog {
        id: fileDialog
        title: "Select Icon File"
        nameFilters: ["Image Files (*.svg *.png *.xpm)", "SVG Files (*.svg)", "PNG Files (*.png)"]
        onAccepted: {
            if (fileDialog.file && iconDetailsSheetInstance && iconDetailsSheetInstance.item) {
                var filePath = fileDialog.file.toString().replace("file://", "")
                iconModel.set_replacement(iconDetailsSheetInstance.item.iconName, filePath)
                projectManager.add_replacement(iconDetailsSheetInstance.item.iconName, filePath)
            }
        }
    }

    Platform.FolderDialog {
        id: openProjectDialog
        title: "Open Theme Folder"
        onAccepted: {
            if (folder) {
                var folderPath = folder.toString().replace("file://", "")
                openProjectDialogInternal(folderPath)
            }
        }
    }

    function openProjectDialogInternal(folderPath) {
        if (projectManager.load_project(folderPath)) {
            projectManager.save_recent_project(folderPath)
            var replacements = projectManager.get_replacements()
            for (var iconName in replacements) {
                var path = replacements[iconName]
                if (path && path !== "" && path !== "null" && path !== "undefined") {
                    iconModel.set_replacement(iconName, path)
                }
            }
            switchToMainPage()
        }
    }

    Component {
        id: aboutSheetComponent
        Loader {
            source: "qrc:///about_sheet.qml"
            asynchronous: false
        }
    }

    property var projectSettingsDialogInstance: null

    function openProjectSettingsDialog() {
        if (!projectSettingsDialogInstance) {
            projectSettingsDialogInstance = projectSettingsDialogComponent.createObject(root)
            if (projectSettingsDialogInstance.item) {
                projectSettingsDialogInstance.item.projectManager = projectManager
                projectSettingsDialogInstance.item.themeManager = themeManager
            }
        }
        if (projectSettingsDialogInstance.item) {
            projectSettingsDialogInstance.item.open()
        }
    }

    Component {
        id: projectSettingsDialogComponent
        Loader {
            source: "qrc:///project_settings_dialog.qml"
            asynchronous: false
            onItemChanged: {
                if (item) {
                    item.projectManager = root.projectManager
                    item.themeManager = root.themeManager
                }
            }
        }
    }

    property var aboutSheet: null

    Component.onCompleted: {
        show()
        raise()
        requestActivate()
        Qt.callLater(function() {
            if (pageStack && projectManager) {
                pageStack.push(welcomePageComponent)
            }
        })
        var aboutSheetLoader = aboutSheetComponent.createObject(root)
        if (aboutSheetLoader.item) {
            aboutSheet = aboutSheetLoader.item
        }
    }
}
