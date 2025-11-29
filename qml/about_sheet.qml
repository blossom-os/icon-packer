import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.19 as Kirigami

Kirigami.OverlaySheet {
    title: "About Icon Packer"
    
    ColumnLayout {
        width: parent.width
        
        Label {
            text: "Icon Packer"
            font.pointSize: 16
            Layout.alignment: Qt.AlignHCenter
        }
        
        Label {
            text: "Version 0.1.0"
            Layout.alignment: Qt.AlignHCenter
        }
        
        Label {
            text: "Create custom KDE icon themes from system icons"
            Layout.alignment: Qt.AlignHCenter
            wrapMode: Text.WordWrap
        }
    }
}

