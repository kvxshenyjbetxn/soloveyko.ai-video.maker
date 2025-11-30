import os
from PySide6.QtWidgets import QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QScrollArea, QLabel, QMessageBox
from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QPixmap
from utils.translator import translator
from gui.gallery_tab.collapsible_group import CollapsibleGroup
from gui.gallery_tab.image_thumbnail import ImageThumbnail
from gui.gallery_tab.regenerate_config_dialog import RegenerateConfigDialog
from utils.logger import logger, LogLevel

class GalleryTab(QWidget):
    image_clicked = Signal(str)

    def __init__(self):
        super().__init__()
        self.task_groups = {}
        self.init_ui()
        self.retranslate_ui()

    def init_ui(self):
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(10, 10, 10, 10)
        main_layout.setSpacing(10)

        # Top bar layout
        top_bar_layout = QHBoxLayout()
        
        self.load_demo_button = QPushButton()
        self.load_demo_button.clicked.connect(self.load_demo_images)
        top_bar_layout.addWidget(self.load_demo_button)

        top_bar_layout.addStretch()

        self.total_images_label = QLabel()
        top_bar_layout.addWidget(self.total_images_label)
        
        main_layout.addLayout(top_bar_layout)

        # Scroll Area
        scroll_area = QScrollArea()
        scroll_area.setWidgetResizable(True)
        scroll_area.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        main_layout.addWidget(scroll_area)

        self.content_widget = QWidget()
        scroll_area.setWidget(self.content_widget)

        self.content_layout = QVBoxLayout(self.content_widget)
        self.content_layout.setAlignment(Qt.AlignmentFlag.AlignTop)

        self.update_total_images_count()

    def load_demo_images(self):
        demo_dir = "demo"
        if not os.path.isdir(demo_dir):
            logger.log(f"Demo directory '{demo_dir}' not found.", level=LogLevel.WARNING)
            return

        image_files = [f for f in os.listdir(demo_dir) if f.lower().endswith(('.png', '.jpg', '.jpeg'))]
        if not image_files:
            logger.log(f"No images found in demo directory '{demo_dir}'.", level=LogLevel.INFO)
            return
        
        logger.log(f"Loading {len(image_files)} demo images.", level=LogLevel.INFO)
        for i, filename in enumerate(image_files):
            image_path = os.path.join(demo_dir, filename)
            task_name = f"Demo Task {(i % 2) + 1}" # Alternate between two demo tasks
            prompt = f"This is a demo image: {filename}"
            self.add_image(task_name, image_path, prompt)

    def add_image(self, task_name, image_path, prompt):
        if not os.path.exists(image_path):
            logger.log(f"Image path does not exist: {image_path}", level=LogLevel.WARNING)
            return
            
        if task_name not in self.task_groups:
            group = CollapsibleGroup(task_name)
            self.content_layout.addWidget(group)
            self.task_groups[task_name] = group
        
        group = self.task_groups[task_name]

        pixmap = QPixmap(image_path)
        scaled_pixmap = pixmap.scaled(200, 200, Qt.AspectRatioMode.KeepAspectRatio, Qt.TransformationMode.SmoothTransformation)
        
        thumbnail = ImageThumbnail(image_path, prompt, scaled_pixmap)
        
        thumbnail.image_clicked.connect(lambda: self._on_image_clicked(image_path))
        thumbnail.delete_requested.connect(lambda: self._on_delete_requested(thumbnail, group))
        thumbnail.regenerate_requested.connect(lambda: self._on_regenerate_requested(thumbnail))

        group.add_widget(thumbnail)
        self.update_total_images_count()

    def _on_image_clicked(self, image_path):
        self.image_clicked.emit(image_path)

    def _on_regenerate_requested(self, thumbnail_widget):
        dialog = RegenerateConfigDialog(thumbnail_widget.prompt, self)
        if dialog.exec():
            new_values = dialog.get_values()
            logger.log(f"Regeneration requested for image {thumbnail_widget.image_path}:", level=LogLevel.INFO)
            logger.log(f"  - New Provider: {new_values['provider']}", level=LogLevel.INFO)
            logger.log(f"  - New Prompt: {new_values['prompt']}", level=LogLevel.INFO)

    def _on_delete_requested(self, thumbnail_widget, group):
        try:
            # We don't delete demo files, just remove them from the UI
            is_demo_file = os.path.dirname(thumbnail_widget.image_path).endswith('demo')
            if not is_demo_file:
                os.remove(thumbnail_widget.image_path)
                logger.log(f"Deleted image from disk: {thumbnail_widget.image_path}", level=LogLevel.INFO)
            else:
                logger.log(f"Removed demo image from gallery: {thumbnail_widget.image_path}", level=LogLevel.INFO)

            group.content_flow_layout.removeWidget(thumbnail_widget)
            thumbnail_widget.deleteLater()

            group.update_title()
            self.update_total_images_count()

        except OSError as e:
            logger.log(f"Error deleting image file {thumbnail_widget.image_path}: {e}", level=LogLevel.ERROR)
            QMessageBox.critical(self, "Error", f"Could not delete image file:\n{e}")

    def update_total_images_count(self):
        total_count = sum(group.get_image_count() for group in self.task_groups.values())
        self.total_images_label.setText(translator.translate("gallery_total_images_label").format(count=total_count))

    def retranslate_ui(self):
        self.update_total_images_count()
        self.load_demo_button.setText(translator.translate("load_demo_button"))
        for group in self.task_groups.values():
            group.translate_ui()
