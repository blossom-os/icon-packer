import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Qt.labs.platform 1.1 as Platform
import org.kde.kirigami 2.19 as Kirigami

Dialog {
    id: dialog
    title: "New Project"
    standardButtons: Dialog.Ok | Dialog.Cancel
    modal: true
    width: 500
    
    property var onProjectAccepted: null
    
    background: Rectangle {
        color: Kirigami.Theme.backgroundColor
        radius: 15
        border.color: Kirigami.Theme.separatorColor
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
    
    contentItem: Column {
        id: contentColumn
        spacing: Kirigami.Units.largeSpacing
        padding: Kirigami.Units.largeSpacing
        width: dialog.availableWidth
        
        Label {
            text: "Project Name:"
            width: contentColumn.width - contentColumn.padding * 2
        }
        
        TextField {
            id: nameField
            width: contentColumn.width - contentColumn.padding * 2
            placeholderText: "My Icon Theme"
            onAccepted: {
                if (text.length > 0 && outputPathField.text.length > 0) {
                    dialog.accept()
                }
            }
        }
        
        Label {
            text: "Theme Output Path:"
            width: contentColumn.width - contentColumn.padding * 2
        }
        
        Row {
            spacing: Kirigami.Units.smallSpacing
            width: contentColumn.width - contentColumn.padding * 2
            
            TextField {
                id: outputPathField
                width: parent.width - browseButton.width - parent.spacing
                placeholderText: "Select theme folder"
            }
            
            Button {
                id: browseButton
                text: "Browse"
                onClicked: folderDialog.open()
            }
        }
    }
    
    Platform.FolderDialog {
        id: folderDialog
        title: "Select Theme Output Folder"
        onAccepted: {
            if (folder) {
                outputPathField.text = folder.toString().replace("file://", "")
            }
        }
    }
    
    onAccepted: {
        if (nameField.text.length > 0 && outputPathField.text.length > 0 && dialog.onProjectAccepted) {
            dialog.onProjectAccepted(nameField.text, outputPathField.text)
        }
    }
}

