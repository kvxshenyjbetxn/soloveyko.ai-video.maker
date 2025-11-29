from PySide6.QtWidgets import QWidget, QVBoxLayout, QToolButton, QFrame
from PySide6.QtCore import Qt
from utils.flow_layout import FlowLayout
from utils.translator import translator

class CollapsibleGroup(QWidget):
    def __init__(self, title, parent=None):
        super().__init__(parent)
        self.is_expanded = True

        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(0, 0, 0, 0)
        main_layout.setSpacing(0)

        self.toggle_button = QToolButton()
        self.toggle_button.setToolButtonStyle(Qt.ToolButtonStyle.ToolButtonTextBesideIcon)
        self.toggle_button.setStyleSheet("QToolButton { border: none; text-align: left; padding: 5px; }")
        self.toggle_button.setAutoRaise(True)
        self.toggle_button.setArrowType(Qt.ArrowType.DownArrow)
        self.toggle_button.clicked.connect(self.toggle)
        main_layout.addWidget(self.toggle_button)

        self.content_area = QFrame()
        self.content_area.setFrameShape(QFrame.Shape.NoFrame)
        self.content_area.setContentsMargins(5, 5, 5, 5)
        self.content_flow_layout = FlowLayout(self.content_area, hSpacing=10, vSpacing=10)
        
        main_layout.addWidget(self.content_area)

        self.set_title(title)

    def set_title(self, title, image_count=0):
        self.title = title
        text = translator.translate("gallery_task_group_title").format(
            title=title,
            count=image_count
        )
        self.toggle_button.setText(text)

    def add_widget(self, widget):
        self.content_flow_layout.addWidget(widget)
        self.update_title()

    def get_image_count(self):
        return self.content_flow_layout.count()

    def update_title(self):
        self.set_title(self.title, self.get_image_count())

    def toggle(self):
        self.is_expanded = not self.is_expanded
        self.content_area.setVisible(self.is_expanded)
        self.toggle_button.setArrowType(Qt.ArrowType.DownArrow if self.is_expanded else Qt.ArrowType.RightArrow)

    def translate_ui(self):
        self.update_title()
