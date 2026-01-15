from PySide6.QtWidgets import QWidget, QVBoxLayout, QToolButton, QFrame
from PySide6.QtCore import Qt, QSize
from utils.flow_layout import FlowLayout
from utils.translator import translator

class CollapsibleGroup(QWidget):
    def __init__(self, title, parent=None, use_flow_layout=False, is_language_group=False, parent_group=None):
        super().__init__(parent)
        self.is_expanded = True
        self.title = title
        self.use_flow_layout = use_flow_layout
        self.is_language_group = is_language_group
        self.parent_group = parent_group
        self.language_groups = {}

        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(0, 0, 0, 0)
        main_layout.setSpacing(0)

        self.toggle_button = QToolButton()
        self.toggle_button.setIconSize(QSize(12, 12))
        self.toggle_button.setToolButtonStyle(Qt.ToolButtonStyle.ToolButtonTextBesideIcon)
        self.toggle_button.setStyleSheet("QToolButton { border: 1px solid rgba(255, 255, 255, 0.1); text-align: left; padding: 2px; border-radius: 4px; font-size: 11px; max-height: 20px; }")
        self.toggle_button.setAutoRaise(True)
        self.toggle_button.setArrowType(Qt.ArrowType.DownArrow)
        self.toggle_button.clicked.connect(self.toggle)
        main_layout.addWidget(self.toggle_button)

        self.content_area = QFrame()
        self.content_area.setFrameShape(QFrame.Shape.NoFrame)
        self.content_area.setContentsMargins(5, 5, 5, 5)

        if self.use_flow_layout:
            self.content_layout = FlowLayout(self.content_area, hSpacing=10, vSpacing=2)
        else:
            self.content_layout = QVBoxLayout(self.content_area)
            self.content_layout.setContentsMargins(0, 5, 0, 0)
            self.content_layout.setSpacing(5)
            self.content_layout.setAlignment(Qt.AlignmentFlag.AlignTop)

        main_layout.addWidget(self.content_area)
        self.set_title(self.title)

    def set_title(self, title, image_count=0, translation_key=None):
        self.title = title
        if translation_key:
            key = translation_key
        else:
            key = "gallery_language_group_title" if self.is_language_group else "gallery_task_group_title"
            
        text = translator.translate(key).format(
            title=title,
            count=image_count
        )
        self.toggle_button.setText(text)

    def add_widget(self, widget):
        self.content_layout.addWidget(widget)
        self.update_title()
        if self.use_flow_layout:
            self.sort_thumbnails()

    def get_media_count(self):
        if self.use_flow_layout:
            return self.content_layout.count()
        
        count = 0
        for i in range(self.content_layout.count()):
            item = self.content_layout.itemAt(i)
            if item and item.widget() and isinstance(item.widget(), CollapsibleGroup):
                count += item.widget().get_media_count()
        return count

    def find_thumbnail_by_path(self, path):
        for i in range(self.content_layout.count()):
            widget = self.content_layout.itemAt(i).widget()
            if self.use_flow_layout:
                if hasattr(widget, 'media_path') and widget.media_path == path:
                    return widget
            elif isinstance(widget, CollapsibleGroup):
                found = widget.find_thumbnail_by_path(path)
                if found:
                    return found
        return None

    def sort_thumbnails(self):
        if not self.use_flow_layout:
            return
            
        thumbnails = []
        for i in range(self.content_layout.count()):
            item = self.content_layout.itemAt(i)
            if item and item.widget():
                thumbnails.append(item.widget())
        
        import re
        def natural_sort_key(s):
            return [int(text) if text.isdigit() else text.lower()
                    for text in re.split('([0-9]+)', s)]

        thumbnails.sort(key=lambda x: natural_sort_key(x.media_path))

        while self.content_layout.count():
            item = self.content_layout.takeAt(0)
            if item.widget():
                item.widget().setParent(None)
        
        for thumb in thumbnails:
            self.content_layout.addWidget(thumb)

    def update_title(self):
        self.set_title(self.title, self.get_media_count())
        if self.parent_group:
            self.parent_group.update_title()

    def toggle(self):
        self.is_expanded = not self.is_expanded
        self.content_area.setVisible(self.is_expanded)
        self.toggle_button.setArrowType(Qt.ArrowType.DownArrow if self.is_expanded else Qt.ArrowType.RightArrow)

    def translate_ui(self):
        self.update_title()
        if not self.use_flow_layout:
            for i in range(self.content_layout.count()):
                item = self.content_layout.itemAt(i)
                if item and item.widget() and isinstance(item.widget(), CollapsibleGroup):
                    item.widget().translate_ui()