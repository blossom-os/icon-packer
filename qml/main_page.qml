import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.19 as Kirigami
import IconModel 1.0
import ProjectManager 1.0

Kirigami.ScrollablePage {
    id: mainPage
    title: "System Icons"
    visible: true
    leftPadding: 0
    rightPadding: 0
    topPadding: 0
    bottomPadding: 0
    
    property IconModel iconModel: null
    property ProjectManager projectManager: null
    signal iconClicked(string iconName)
    
    onIconModelChanged: {
        if (iconList) {
            iconList.model = iconModel
        }
    }
    
    Connections {
        target: projectManager
        enabled: projectManager !== null
        function onCurrent_project_changed() {
        }
    }
    
    ColumnLayout {
        anchors.fill: parent
        spacing: 0
            
            Column {
                Layout.fillWidth: true
                spacing: Kirigami.Units.smallSpacing
                padding: Kirigami.Units.largeSpacing
                
                Kirigami.SearchField {
                    id: searchField
                    width: parent.width
                    placeholderText: "Search icons..."
                    onTextChanged: {
                        if (iconModel) {
                            iconModel.set_search_text(text)
                        }
                    }
                }
                
                RowLayout {
                    width: parent.width
                    
                    Kirigami.Icon {
                        source: "view-filter"
                        Layout.preferredWidth: Kirigami.Units.iconSizes.smallMedium
                        Layout.preferredHeight: Kirigami.Units.iconSizes.smallMedium
                        fallback: ""
                    }
                    
                    ComboBox {
                        id: categoryFilter
                        Layout.fillWidth: true
                        model: ["All Categories", "Applications", "Mimetypes", "Actions", "Places", "Devices", "Status"]
                        onCurrentTextChanged: {
                            if (iconModel) {
                                iconModel.set_category_filter(currentText)
                            }
                        }
                    }
                    
                    CheckBox {
                        id: showReplacedOnly
                        text: "Replaced only"
                    }
                    
                    Kirigami.Icon {
                        source: "view-grid"
                        Layout.preferredWidth: Kirigami.Units.iconSizes.smallMedium
                        Layout.preferredHeight: Kirigami.Units.iconSizes.smallMedium
                        fallback: ""
                    }
                    
                    Slider {
                        id: iconSizeSlider
                        from: 32
                        to: 128
                        value: 64
                        stepSize: 16
                        Layout.fillWidth: true
                    }
                    
                    Label {
                        text: Math.round(iconSizeSlider.value) + "px"
                        Layout.preferredWidth: 50
                    }
                }
                
                BusyIndicator {
                    width: parent.width
                    height: 40
                    running: iconModel ? iconModel.loading : false
                    visible: iconModel ? iconModel.loading : false
                }
            }

            GridView {
                id: iconList
                Layout.fillWidth: true
                Layout.fillHeight: true
                model: iconModel
                clip: true
                cellWidth: Math.round(iconSizeSlider.value) + 40
                cellHeight: Math.round(iconSizeSlider.value) + 60
                
                delegate: Item {
                    width: iconList.cellWidth
                    height: showReplacedOnly.checked ? 
                        ((model.replacementPath && model.replacementPath !== "") ? iconList.cellHeight : 0) :
                        iconList.cellHeight
                    visible: !showReplacedOnly.checked || (model.replacementPath && model.replacementPath !== "")
                    
                    Rectangle {
                        anchors.fill: parent
                        anchors.margins: 4
                        color: iconMouseArea.containsMouse ? Kirigami.Theme.hoverColor : Kirigami.Theme.backgroundColor
                        border.color: model.replacementPath ? Kirigami.Theme.positiveTextColor : Kirigami.Theme.disabledTextColor
                        border.width: model.replacementPath ? 2 : 1
                        radius: 8
                        
                        MouseArea {
                            id: iconMouseArea
                            anchors.fill: parent
                            hoverEnabled: true
                            acceptedButtons: Qt.LeftButton
                            
                            onClicked: function(mouse) {
                                mainPage.iconClicked(model.name)
                            }
                        }
                        
                        Column {
                            anchors.centerIn: parent
                            spacing: 4
                            width: parent.width - 8
                            
                            Item {
                                width: Math.round(iconSizeSlider.value)
                                height: Math.round(iconSizeSlider.value)
                                anchors.horizontalCenter: parent.horizontalCenter
                                
                                Image {
                                    id: iconImage
                                    anchors.fill: parent
                                    source: {
                                        if (model.replacementPath && model.replacementPath !== "") {
                                            return "file://" + model.replacementPath
                                        }
                                        return ""
                                    }
                                    fillMode: Image.PreserveAspectFit
                                    visible: source !== "" && status === Image.Ready
                                    asynchronous: true
                                    cache: false
                                    mipmap: false
                                    smooth: true
                                }
                                
                                Kirigami.Icon {
                                    anchors.fill: parent
                                    source: (model.name && model.name !== "") ? model.name : ""
                                    visible: !iconImage.visible && source !== "" && model.name
                                    fallback: ""
                                    onSourceChanged: {
                                        if (!source || source === "") {
                                            visible = false
                                        }
                                    }
                                }
                                
                                Rectangle {
                                    anchors.fill: parent
                                    color: "transparent"
                                    border.color: Kirigami.Theme.textColor
                                    border.width: 1
                                    radius: 2
                                    visible: !iconImage.visible
                                }
                                
                                Rectangle {
                                    anchors.top: parent.top
                                    anchors.right: parent.right
                                    anchors.topMargin: -4
                                    anchors.rightMargin: -4
                                    width: 16
                                    height: 16
                                    radius: 8
                                    color: Kirigami.Theme.positiveTextColor
                                    visible: model.replacementPath && model.replacementPath !== ""
                                    border.color: Kirigami.Theme.backgroundColor
                                    border.width: 2
                                    
                                    Kirigami.Icon {
                                        anchors.centerIn: parent
                                        source: "emblem-checked"
                                        width: 10
                                        height: 10
                                        color: "white"
                                        fallback: ""
                                    }
                                    
                                    ToolTip {
                                        visible: parent.visible && changedIndicatorMouseArea.containsMouse
                                        text: {
                                            if (!model.replacementPath) return ""
                                            var path = model.replacementPath
                                            var parts = path.split("/")
                                            return "Replaced with: " + (parts.length > 0 ? parts[parts.length - 1] : path)
                                        }
                                        delay: 500
                                        timeout: 3000
                                    }
                                    
                                    MouseArea {
                                        id: changedIndicatorMouseArea
                                        anchors.fill: parent
                                        hoverEnabled: true
                                    }
                                }
                            }
                            
                            Label {
                                text: model.name
                                width: parent.width
                                elide: Text.ElideRight
                                horizontalAlignment: Text.AlignHCenter
                                font.pointSize: 8
                            }
                            
                            Row {
                                anchors.horizontalCenter: parent.horizontalCenter
                                spacing: 4
                                
                                Kirigami.Icon {
                                    source: "image-svg"
                                    width: 12
                                    height: 12
                                    visible: model.hasSvg
                                    fallback: ""
                                }
                                
                                Kirigami.Icon {
                                    source: "image-png"
                                    width: 12
                                    height: 12
                                    visible: model.hasPng
                                    fallback: ""
                                }
                            }
                        }
                    }
                }
            }
        }
    }
