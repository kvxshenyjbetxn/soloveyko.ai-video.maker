import os
from PySide6.QtWidgets import QWidget, QVBoxLayout, QScrollArea, QLabel, QMessageBox
from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QPixmap
from utils.translator import translator
from gui.collapsible_group import CollapsibleGroup
from gui.widgets.image_thumbnail import ImageThumbnail
from gui.dialogs.regenerate_config_dialog import RegenerateConfigDialog
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

        self.total_images_label = QLabel()
        main_layout.addWidget(self.total_images_label, alignment=Qt.AlignmentFlag.AlignRight)

        scroll_area = QScrollArea()
        scroll_area.setWidgetResizable(True)
        scroll_area.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        main_layout.addWidget(scroll_area)

        self.content_widget = QWidget()
        scroll_area.setWidget(self.content_widget)

        self.content_layout = QVBoxLayout(self.content_widget)
        self.content_layout.setAlignment(Qt.AlignmentFlag.AlignTop)

        self.update_total_images_count()

    def add_image(self, task_name, image_path, prompt):
        if task_name not in self.task_groups:
            group = CollapsibleGroup(task_name)
            self.content_layout.addWidget(group)
            self.task_groups[task_name] = group
        
        group = self.task_groups[task_name]

        # Create and scale the pixmap here
        pixmap = QPixmap(image_path)
        scaled_pixmap = pixmap.scaled(200, 200, Qt.AspectRatioMode.KeepAspectRatio, Qt.TransformationMode.SmoothTransformation)
        
        # Pass the scaled pixmap to the thumbnail
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
            # Actual regeneration logic is not implemented yet, as per request.

    def _on_delete_requested(self, thumbnail_widget, group):
        confirm_msg = translator.translate("confirm_image_deletion_message").format(thumbnail_widget.image_path)
        reply = QMessageBox.question(self, 
                                     translator.translate("confirm_deletion_title"), 
                                     confirm_msg,
                                     QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No, 
                                     QMessageBox.StandardButton.No)

        if reply == QMessageBox.StandardButton.Yes:
            try:
                # 1. Delete from disk
                os.remove(thumbnail_widget.image_path)
                logger.log(f"Deleted image from disk: {thumbnail_widget.image_path}", level=LogLevel.INFO)

                # 2. Remove from layout
                group.content_flow_layout.removeWidget(thumbnail_widget)
                thumbnail_widget.deleteLater()

                # 3. Update counts
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
        for group in self.task_groups.values():
            group.translate_ui()
        # Need to re-translate any visible dialogs if they are open during language change
        # For now, this is not implemented as dialogs are modal and short-lived.
