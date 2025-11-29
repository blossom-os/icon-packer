import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Qt.labs.platform 1.1 as Platform
import org.kde.kirigami 2.19 as Kirigami
import ProjectManager 1.0

import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Qt.labs.platform 1.1 as Platform
import org.kde.kirigami 2.19 as Kirigami
import ProjectManager 1.0

Dialog {
    id: dialog
    title: "Generate Icon Theme"
    modal: true
    width: 500
    height: 250
    
    property ProjectManager projectManager
    property var onExport: null
    
    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Kirigami.Units.largeSpacing
        
        Label {
            text: "Theme Name:"
        }
        
        TextField {
            id: themeNameField
            placeholderText: "My Icon Theme"
            text: projectManager ? projectManager.projectName : ""
            Layout.fillWidth: true
        }
        
        Label {
            text: "Output Directory:"
        }
        
        RowLayout {
            Layout.fillWidth: true
            
            TextField {
                id: outputPathField
                placeholderText: "Select output directory"
                Layout.fillWidth: true
            }
            
            Button {
                text: "Browse"
                onClicked: folderDialog.open()
            }
        }
        
        RowLayout {
            Layout.fillWidth: true
            
            Button {
                text: "Generate"
                icon.name: "document-export"
                Layout.fillWidth: true
                onClicked: {
                    if (projectManager && themeNameField.text && outputPathField.text) {
                        var success = projectManager.generate_theme(themeNameField.text, outputPathField.text)
                        if (success) {
                            if (dialog.onExport) {
                                dialog.onExport(outputPathField.text)
                            }
                            dialog.close()
                        }
                    }
                }
            }
            
            Button {
                text: "Cancel"
                Layout.fillWidth: true
                onClicked: dialog.close()
            }
        }
    }
    
    Platform.FolderDialog {
        id: folderDialog
        title: "Select Output Directory"
        onAccepted: {
            if (folder) {
                outputPathField.text = folder.toString().replace("file://", "")
            }
        }
    }
}

