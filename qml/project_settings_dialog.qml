import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.19 as Kirigami
import ProjectManager 1.0
import ThemeManager 1.0

Dialog {
    id: dialog
    title: "Project Settings"
    modal: true
    width: 600
    height: 550
    
    property ProjectManager projectManager: null
    property ThemeManager themeManager: null
    
    background: Rectangle {
        color: Kirigami.Theme.backgroundColor
        radius: 15
        border.color: Kirigami.Theme.separatorColor
        border.width: 1
        layer.enabled: true
        layer.smooth: true
    }
    
    contentItem: ColumnLayout {
        spacing: Kirigami.Units.largeSpacing
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.margins: Kirigami.Units.largeSpacing
        
        Label {
            text: "Theme Name"
            font.bold: true
            Layout.fillWidth: true
        }
        
        TextField {
            id: themeNameField
            Layout.fillWidth: true
            placeholderText: "Enter theme display name"
            Component.onCompleted: updateThemeFields()
            onTextChanged: {
                if (projectManager && !updatingFields) {
                    projectManager.set_theme_name(text)
                }
            }
        }
        
        Item {
            Layout.preferredHeight: Kirigami.Units.mediumSpacing
        }
        
        Label {
            text: "Theme Comment"
            font.bold: true
            Layout.fillWidth: true
        }
        
        TextArea {
            id: themeCommentField
            Layout.fillWidth: true
            Layout.preferredHeight: 80
            placeholderText: "Enter theme description/comment"
            wrapMode: TextArea.Wrap
            Component.onCompleted: updateThemeFields()
            onTextChanged: {
                if (projectManager && !updatingFields) {
                    projectManager.set_theme_comment(text)
                }
            }
        }
        
        Item {
            Layout.preferredHeight: Kirigami.Units.mediumSpacing
        }
        
        Label {
            text: "Fallback Themes"
            font.bold: true
            Layout.fillWidth: true
        }
        
        Label {
            text: "Select themes to inherit from (comma-separated). Icons not found in your theme will fall back to these themes in order."
            Layout.fillWidth: true
            wrapMode: Text.WordWrap
        }
        
        ScrollView {
            Layout.fillWidth: true
            Layout.preferredHeight: 200
            clip: true
            
            ListView {
                id: themeList
                width: parent.width
                model: {
                    if (!themeManager) return []
                    var themes = themeManager.get_theme_names()
                    return themes ? themes.split(",") : []
                }
                
                delegate: CheckDelegate {
                    width: themeList.width
                    text: modelData
                    checked: {
                        if (!projectManager) return false
                        var fallbacks = projectManager.get_fallback_themes()
                        if (!fallbacks) return false
                        var fallbackList = fallbacks.split(",")
                        for (var i = 0; i < fallbackList.length; i++) {
                            if (fallbackList[i].trim() === modelData) {
                                return true
                            }
                        }
                        return false
                    }
                    
                    onToggled: {
                        updateFallbacks()
                    }
                }
            }
        }
        
        TextField {
            id: fallbackField
            Layout.fillWidth: true
            placeholderText: "e.g., hicolor,breeze"
            text: projectManager ? projectManager.get_fallback_themes() : ""
            onTextChanged: {
                if (projectManager) {
                    projectManager.set_fallback_themes(text)
                }
            }
        }
        
        Label {
            text: "Enter theme names separated by commas"
            Layout.fillWidth: true
            font.pointSize: 9
            color: Kirigami.Theme.disabledTextColor
        }
    }
    
    standardButtons: Dialog.Close
    
    property bool updatingFields: false
    
    function updateThemeFields() {
        if (!projectManager) return
        updatingFields = true
        themeNameField.text = projectManager.get_theme_name()
        themeCommentField.text = projectManager.get_theme_comment()
        updatingFields = false
    }
    
    function updateFallbacks() {
        var selected = []
        for (var i = 0; i < themeList.count; i++) {
            var item = themeList.itemAtIndex(i)
            if (item && item.checked) {
                selected.push(item.text)
            }
        }
        if (projectManager) {
            var themesStr = selected.length > 0 ? selected.join(",") : "hicolor"
            projectManager.set_fallback_themes(themesStr)
            fallbackField.text = themesStr
        }
    }
    
    onProjectManagerChanged: {
        Qt.callLater(updateThemeFields)
    }
    
    onVisibleChanged: {
        if (visible) {
            Qt.callLater(updateThemeFields)
        }
    }
    
    Component.onCompleted: {
        updateThemeFields()
        if (projectManager) {
            fallbackField.text = projectManager.get_fallback_themes() || "hicolor"
        }
    }
}

