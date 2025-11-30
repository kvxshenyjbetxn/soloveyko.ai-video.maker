import os
import uuid
import base64
from PySide6.QtWidgets import QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QScrollArea, QLabel, QMessageBox
from PySide6.QtCore import Qt, Signal, QObject, QRunnable, QThreadPool
from PySide6.QtGui import QPixmap
from utils.translator import translator
from gui.gallery_tab.collapsible_group import CollapsibleGroup
from gui.gallery_tab.image_thumbnail import ImageThumbnail
from gui.gallery_tab.regenerate_config_dialog import RegenerateConfigDialog
from utils.logger import logger, LogLevel
from api.pollinations import PollinationsAPI
from api.googler import GooglerAPI

class RegenerateImageWorkerSignals(QObject):
    finished = Signal(str, str) # old_path, new_path
    error = Signal(str, str)    # old_path, error_message

class RegenerateImageWorker(QRunnable):
    def __init__(self, old_image_path, regen_config):
        super().__init__()
        self.signals = RegenerateImageWorkerSignals()
        self.old_image_path = old_image_path
        self.config = regen_config
        self.googler_api = GooglerAPI()
        self.pollinations_api = PollinationsAPI()

    def run(self):
        try:
            provider = self.config['provider']
            prompt = self.config['prompt']
            
            api = None
            api_kwargs = {}
            file_extension = 'png'

            if provider == 'googler':
                api = self.googler_api
                googler_config = self.config.get('googler_config', {})
                api_kwargs['aspect_ratio'] = googler_config.get('aspect_ratio', 'IMAGE_ASPECT_RATIO_LANDSCAPE')
                api_kwargs['seed'] = googler_config.get('seed')
                api_kwargs['negative_prompt'] = googler_config.get('negative_prompt')
                file_extension = 'jpg'
            elif provider == 'pollinations':
                api = self.pollinations_api
                # Add any pollinations specific kwargs here in the future
            
            if not api:
                raise ValueError(f"Invalid image generation provider: {provider}")

            logger.log(f"Starting regeneration with {provider} for prompt: '{prompt}'", level=LogLevel.INFO)
            image_data = api.generate_image(prompt, **api_kwargs)

            if not image_data:
                raise ValueError("API returned no data.")

            # Save to a new file, overwriting the old one (or creating a new one if extension changes)
            base_name, _ = os.path.splitext(self.old_image_path)
            new_image_path = f"{base_name}.{file_extension}"
            
            data_to_write = image_data
            if provider == 'googler' and isinstance(image_data, str):
                data_to_write = base64.b64decode(image_data.split(",", 1)[1] if "," in image_data else image_data)

            with open(new_image_path, 'wb') as f:
                f.write(data_to_write)
            
            logger.log(f"Successfully saved regenerated image to {new_image_path}", level=LogLevel.SUCCESS)
            self.signals.finished.emit(self.old_image_path, new_image_path)

        except Exception as e:
            logger.log(f"Failed to regenerate image. Error: {e}", level=LogLevel.ERROR)
            self.signals.error.emit(self.old_image_path, str(e))


class GalleryTab(QWidget):
    image_clicked = Signal(str)

    def __init__(self):
        super().__init__()
        self.task_groups = {}
        self.threadpool = QThreadPool()
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
        thumbnail.regenerate_requested.connect(self._on_regenerate_requested)

        group.add_widget(thumbnail)
        self.update_total_images_count()

    def _on_image_clicked(self, image_path):
        self.image_clicked.emit(image_path)

    def _on_regenerate_requested(self, image_data):
        dialog = RegenerateConfigDialog(image_data, self)
        if dialog.exec():
            new_values = dialog.get_values()
            
            logger.log(f"  - New Config: {new_values}", level=LogLevel.INFO)

            worker = RegenerateImageWorker(image_data['image_path'], new_values)
            worker.signals.finished.connect(self._on_regeneration_finished)
            worker.signals.error.connect(self._on_regeneration_error)
            self.threadpool.start(worker)

    def _on_regeneration_finished(self, old_path, new_path):
        # Find the thumbnail and update it
        for group in self.task_groups.values():
            thumbnail = group.find_thumbnail_by_path(old_path)
            if thumbnail:
                thumbnail.update_image(new_path)
                logger.log(f"Updated thumbnail for {old_path} with new image {new_path}", level=LogLevel.INFO)
                
                # If the path changed (e.g. extension), delete the old image file
                if old_path != new_path:
                    try:
                        if os.path.exists(old_path):
                            os.remove(old_path)
                            logger.log(f"Deleted old image file: {old_path}", level=LogLevel.INFO)
                    except OSError as e:
                        logger.log(f"Error deleting old image file {old_path}: {e}", level=LogLevel.ERROR)
                break

    def _on_regeneration_error(self, old_path, error_message):
        QMessageBox.critical(self, "Regeneration Failed", f"Could not regenerate image for '{os.path.basename(old_path)}':\n\n{error_message}")


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
