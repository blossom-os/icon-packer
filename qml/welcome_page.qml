import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.19 as Kirigami
import ProjectManager 1.0

Kirigami.Page {
    id: welcomePage
    title: "Welcome"
    visible: true

    property ProjectManager projectManager: null
    property var recentProjects: []

    signal newProjectRequested()
    signal openProjectRequested()
    signal recentProjectRequested(string path)

    function updateRecentProjects() {
        if (!projectManager) {
            recentProjects = []
            return
        }
        
        var recent = projectManager.get_recent_projects()
        if (!recent) {
            recentProjects = []
            return
        }
        
        var projects = recent.split("\n").filter(function(p) {
            return p !== ""
        })
        
        recentProjects = projects
    }

    onProjectManagerChanged: updateRecentProjects()
    Component.onCompleted: {
        Qt.callLater(updateRecentProjects)
    }
    
    onVisibleChanged: {
        if (visible) {
            Qt.callLater(updateRecentProjects)
        }
    }

    Rectangle {
        anchors.fill: parent
        color: Kirigami.Theme.backgroundColor

        ColumnLayout {
            anchors.centerIn: parent
            spacing: Kirigami.Units.gridUnit * 2
            width: Math.min(parent.width * 0.65, 700)

            ColumnLayout {
                spacing: Kirigami.Units.mediumSpacing
                Layout.alignment: Qt.AlignHCenter
                Layout.bottomMargin: Kirigami.Units.gridUnit

                Kirigami.Heading {
                    text: "Icon Packer"
                    level: 1
                    font.pointSize: 36
                    font.weight: Font.Bold
                    Layout.alignment: Qt.AlignHCenter
                }

                Label {
                    text: "Create custom icon themes for KDE"
                    font.pointSize: 14
                    opacity: 0.8
                    Layout.alignment: Qt.AlignHCenter
                    Layout.topMargin: -Kirigami.Units.smallSpacing
                }
            }

            RowLayout {
                spacing: Kirigami.Units.largeSpacing
                Layout.alignment: Qt.AlignHCenter
                Layout.topMargin: Kirigami.Units.gridUnit

                Button {
                    text: "New Project"
                    icon.name: "document-new"
                    Layout.preferredWidth: 220
                    Layout.preferredHeight: 44
                    font.pointSize: 13
                    onClicked: welcomePage.newProjectRequested()
                }

                Button {
                    text: "Open Project"
                    icon.name: "document-open"
                    Layout.preferredWidth: 220
                    Layout.preferredHeight: 44
                    font.pointSize: 13
                    onClicked: welcomePage.openProjectRequested()
                }
            }

            Item {
                Layout.preferredHeight: Kirigami.Units.gridUnit * 0.5
            }

            Item {
                Layout.fillWidth: true
                Layout.preferredHeight: 240
                Layout.topMargin: Kirigami.Units.gridUnit
                visible: recentProjectsList.count > 0

                ColumnLayout {
                    anchors.fill: parent
                    spacing: Kirigami.Units.mediumSpacing

                    Label {
                        text: "Recent Projects"
                        font.bold: true
                        font.pointSize: 13
                        Layout.fillWidth: true
                        Layout.leftMargin: Kirigami.Units.smallSpacing
                    }

                    ScrollView {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        clip: true

                        ListView {
                            id: recentProjectsList
                            model: welcomePage.recentProjects
                            spacing: 2

                            delegate: ItemDelegate {
                                width: recentProjectsList.width
                                text: {
                                    var path = modelData
                                    if (!path) return ""
                                    var parts = path.split("/")
                                    return parts.length > 0 ? parts[parts.length - 1] : path
                                }
                                icon.name: "folder"
                                padding: Kirigami.Units.mediumSpacing
                                onClicked: {
                                    if (modelData && modelData !== "") {
                                        welcomePage.recentProjectRequested(modelData)
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
