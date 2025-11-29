import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Qt.labs.platform 1.1 as Platform
import org.kde.kirigami 2.19 as Kirigami
import IconModel 1.0
import ProjectManager 1.0

Dialog {
    id: dialog
    title: iconName || "Select Icon"
    modal: true
    width: 800
    height: 650
    
    property string iconName: ""
    property IconModel iconModel: null
    property ProjectManager projectManager: null
    
    Connections {
        target: projectManager
        enabled: projectManager !== null
        function onCurrent_project_changed() {
            Qt.callLater(function() {
                baseSvgField.text = ""
                Qt.callLater(function() {
                    var replacements = dialog.projectManager.get_replacements()
                    var path = replacements[dialog.iconName]
                    if (path && path !== "" && path.toLowerCase().endsWith(".svg")) {
                        baseSvgField.text = path.split("/").pop()
                    }
                })
            })
        }
    }
    
    background: Rectangle {
        color: Kirigami.Theme.backgroundColor || "#1e1e1e"
        radius: 15
        border.color: Kirigami.Theme.separatorColor || "#2d2d2d"
        border.width: 1
        layer.enabled: true
        layer.smooth: true
    }
    
    onParentChanged: {
        if (parent) {
            x = (parent.width - width) / 2
            y = (parent.height - height) / 2
        }
    }
    
    contentItem: ScrollView {
        width: dialog.availableWidth
        height: dialog.availableHeight
        
        ColumnLayout {
            width: dialog.availableWidth - Kirigami.Units.largeSpacing * 4
            spacing: Kirigami.Units.largeSpacing * 1.5
            anchors.margins: Kirigami.Units.largeSpacing * 2
            
            Label {
                text: "Base SVG (for all sizes)"
                font.bold: true
                font.pointSize: 13
                Layout.fillWidth: true
            }
            
            Label {
                text: "Assign one SVG that will be used for all sizes"
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                opacity: 0.8
            }
            
            RowLayout {
                Layout.fillWidth: true
                spacing: Kirigami.Units.smallSpacing
                
                TextField {
                    id: baseSvgField
                    Layout.fillWidth: true
                    placeholderText: "No base SVG assigned"
                    readOnly: true
                    text: {
                        if (!dialog.projectManager || !dialog.iconName) return ""
                        var replacements = dialog.projectManager.get_replacements()
                        var path = replacements[dialog.iconName]
                        if (path && path !== "" && path.toLowerCase().endsWith(".svg")) {
                            var parts = path.split("/")
                            return parts.length > 0 ? parts[parts.length - 1] : path
                        }
                        return ""
                    }
                }
                
                RowLayout {
                    Layout.fillWidth: true
                    spacing: Kirigami.Units.smallSpacing
                    
                    Button {
                        text: baseSvgField.text ? "Change" : "Select SVG"
                        Layout.fillWidth: true
                        onClicked: {
                            baseSvgDialog.isLink = false
                            baseSvgDialog.open()
                        }
                    }
                    
                    Button {
                        text: "Link"
                        icon.name: "emblem-symbolic-link"
                        display: Button.TextBesideIcon
                        Layout.fillWidth: true
                        onClicked: {
                            baseSvgDialog.isLink = true
                            baseSvgDialog.open()
                        }
                    }
                    
                    Button {
                        text: "Clear"
                        enabled: baseSvgField.text !== ""
                        onClicked: {
                            if (dialog.projectManager && dialog.iconName) {
                                dialog.projectManager.add_replacement(dialog.iconName, "")
                                if (dialog.iconModel) {
                                    dialog.iconModel.clear_replacement(dialog.iconName)
                                }
                            }
                        }
                    }
                }
            }
            
            Item {
                Layout.preferredHeight: Kirigami.Units.mediumSpacing
            }
            
            Label {
                text: "Individual Size Overrides"
                font.bold: true
                font.pointSize: 13
                Layout.fillWidth: true
            }
            
            Label {
                text: "Override specific sizes with different SVGs (optional)"
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                opacity: 0.8
            }
            
            GridLayout {
                columns: 3
                Layout.fillWidth: true
                rowSpacing: Kirigami.Units.mediumSpacing
                columnSpacing: Kirigami.Units.mediumSpacing
                
                Repeater {
                    model: [
                        {size: 16, label: "16px"},
                        {size: 24, label: "24px"},
                        {size: 32, label: "32px"},
                        {size: 48, label: "48px"},
                        {size: 64, label: "64px"},
                        {size: 128, label: "128px"}
                    ]
                    
                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: Kirigami.Units.smallSpacing
                        
                        Label {
                            text: modelData.label
                            font.bold: true
                            Layout.alignment: Qt.AlignHCenter
                        }
                        
                        TextField {
                            id: sizeField
                            Layout.fillWidth: true
                            placeholderText: "Auto from base"
                            readOnly: true
                            text: {
                                if (!dialog.projectManager || !dialog.iconName) return ""
                                var sizeMap = dialog.projectManager.get_size_replacements(dialog.iconName)
                                if (sizeMap && sizeMap[modelData.size.toString()]) {
                                    var path = sizeMap[modelData.size.toString()]
                                    var parts = path.split("/")
                                    return parts.length > 0 ? parts[parts.length - 1] : path
                                }
                                return ""
                            }
                        }
                        
                        RowLayout {
                            Layout.fillWidth: true
                            spacing: Kirigami.Units.smallSpacing
                            
                            Button {
                                text: sizeField.text ? "Change" : "Set"
                                Layout.fillWidth: true
                                onClicked: {
                                    sizeFileDialog.size = modelData.size
                                    sizeFileDialog.isLink = false
                                    sizeFileDialog.open()
                                }
                            }
                            
                            Button {
                                text: "Link"
                                icon.name: "emblem-symbolic-link"
                                display: Button.TextBesideIcon
                                Layout.fillWidth: true
                                onClicked: {
                                    sizeFileDialog.size = modelData.size
                                    sizeFileDialog.isLink = true
                                    sizeFileDialog.open()
                                }
                            }
                            
                            Button {
                                text: "Clear"
                                Layout.fillWidth: true
                                enabled: sizeField.text !== ""
                                onClicked: {
                                    if (dialog.projectManager && dialog.iconName) {
                                        dialog.projectManager.add_size_replacement(dialog.iconName, modelData.size, "")
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    standardButtons: Dialog.Close
    
    Platform.FileDialog {
        id: baseSvgDialog
        property bool isLink: false
        title: isLink ? "Link Base SVG Icon" : "Select Base SVG Icon"
        nameFilters: ["SVG Files (*.svg)", "All Files (*.*)"]
        onAccepted: {
            if (file && dialog.projectManager && dialog.iconName) {
                var filePath = file.toString().replace("file://", "")
                dialog.projectManager.add_replacement(dialog.iconName, filePath)
                if (isLink) {
                    dialog.projectManager.set_replacement_link(dialog.iconName, true)
                }
                if (dialog.iconModel) {
                    dialog.iconModel.set_replacement(dialog.iconName, filePath)
                    var category = dialog.iconModel.get_icon_category(dialog.iconName)
                    if (dialog.projectManager) {
                        dialog.projectManager.set_icon_category(dialog.iconName, category)
                    }
                } else if (dialog.projectManager) {
                    dialog.projectManager.set_icon_category(dialog.iconName, "Applications")
                }
                Qt.callLater(function() {
                    if (baseSvgField) {
                        baseSvgField.text = filePath.split("/").pop()
                    }
                })
            }
        }
    }
    
    Platform.FileDialog {
        id: sizeFileDialog
        property int size: 16
        property bool isLink: false
        title: (isLink ? "Link" : "Select") + " Icon File for " + sizeFileDialog.size + "px"
        nameFilters: ["SVG Files (*.svg)", "PNG Files (*.png)", "All Files (*.*)"]
        onAccepted: {
            if (file && dialog.projectManager && dialog.iconName) {
                var filePath = file.toString().replace("file://", "")
                dialog.projectManager.add_size_replacement(dialog.iconName, sizeFileDialog.size, filePath)
                if (isLink) {
                    dialog.projectManager.set_size_replacement_link(dialog.iconName, sizeFileDialog.size, true)
                }
                if (dialog.iconModel) {
                    var category = dialog.iconModel.get_icon_category(dialog.iconName)
                    dialog.projectManager.set_icon_category(dialog.iconName, category)
                } else {
                    dialog.projectManager.set_icon_category(dialog.iconName, "Applications")
                }
                if (dialog.iconModel && sizeFileDialog.size === 48) {
                    dialog.iconModel.set_replacement(dialog.iconName, filePath)
                }
                Qt.callLater(function() {
                    if (dialog.projectManager) {
                        dialog.projectManager.current_project_changed()
                    }
                })
            }
        }
    }
}
