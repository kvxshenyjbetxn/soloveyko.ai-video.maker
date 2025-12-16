import os
import re
import uuid
import base64
from PySide6.QtWidgets import QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QScrollArea, QLabel, QMessageBox
from PySide6.QtCore import Qt, Signal, QObject, QRunnable, QThreadPool
from PySide6.QtGui import QPixmap
from utils.translator import translator
from gui.gallery_tab.collapsible_group import CollapsibleGroup
from gui.gallery_tab.media_thumbnail import MediaThumbnail
from gui.gallery_tab.regenerate_config_dialog import RegenerateConfigDialog
from utils.logger import logger, LogLevel
from api.pollinations import PollinationsAPI
from api.googler import GooglerAPI

class RegenerateImageWorkerSignals(QObject):
    finished = Signal(str, str, str) # old_path, new_path, thumbnail_path
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
                pollinations_config = self.config.get('pollinations_config', {})
                api.model = pollinations_config.get('model', 'flux')
            
            if not api:
                raise ValueError(f"Invalid image generation provider: {provider}")

            logger.log(f"Starting regeneration with {provider} for prompt: '{prompt}'", level=LogLevel.INFO)
            image_data = api.generate_image(prompt, **api_kwargs)

            if not image_data:
                raise ValueError("API returned no data.")

            base_name, _ = os.path.splitext(self.old_image_path)
            new_image_path = f"{base_name}.{file_extension}"
            
            data_to_write = image_data
            if provider == 'googler' and isinstance(image_data, str):
                data_to_write = base64.b64decode(image_data.split(",", 1)[1] if "," in image_data else image_data)

            with open(new_image_path, 'wb') as f:
                f.write(data_to_write)
            
            logger.log(f"Successfully saved regenerated image to {new_image_path}", level=LogLevel.SUCCESS)
            
            # --- Thumbnail Generation ---
            thumbnail_path = ""
            try:
                base, ext = os.path.splitext(new_image_path)
                thumbnail_path = f"{base}_thumb.jpg"
                
                pixmap = QPixmap(new_image_path)
                if not pixmap.isNull():
                    scaled_pixmap = pixmap.scaled(290, 290, Qt.AspectRatioMode.KeepAspectRatio, Qt.TransformationMode.SmoothTransformation)
                    scaled_pixmap.save(thumbnail_path, "JPG", 85)
                else:
                    thumbnail_path = ""
            except Exception as thumb_e:
                thumbnail_path = ""
                logger.log(f"Error generating thumbnail for regenerated image {new_image_path}: {thumb_e}", level=LogLevel.ERROR)
            # --- End Thumbnail Generation ---

            self.signals.finished.emit(self.old_image_path, new_image_path, thumbnail_path)

        except Exception as e:
            logger.log(f"Failed to regenerate image. Error: {e}", level=LogLevel.ERROR)
            self.signals.error.emit(self.old_image_path, str(e))


class GalleryTab(QWidget):
    media_clicked = Signal(str)
    image_deleted = Signal(str)
    image_regenerated = Signal(str, str) # old_path, new_path
    continue_montage_requested = Signal()

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

        top_bar_layout = QHBoxLayout()
        
        #self.load_demo_button = QPushButton()
        #self.load_demo_button.clicked.connect(self.load_demo_media)
        #top_bar_layout.addWidget(self.load_demo_button)

        top_bar_layout.addStretch()

        self.total_images_label = QLabel()
        top_bar_layout.addWidget(self.total_images_label)
        
        main_layout.addLayout(top_bar_layout)

        scroll_area = QScrollArea()
        scroll_area.setWidgetResizable(True)
        scroll_area.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        main_layout.addWidget(scroll_area)

        self.content_widget = QWidget()
        scroll_area.setWidget(self.content_widget)

        self.content_layout = QVBoxLayout(self.content_widget)
        self.content_layout.setAlignment(Qt.AlignmentFlag.AlignTop)

        self.continue_button = QPushButton()
        self.continue_button.clicked.connect(self._on_continue_clicked)
        self.continue_button.hide()
        main_layout.addWidget(self.continue_button)

        self.update_total_media_count()

    def clear_gallery(self):
        """Removes all images and groups from the gallery view."""
        # Clear the data model
        self.task_groups.clear()

        # Clear the UI by deleting all widgets in the content layout
        while self.content_layout.count():
            child = self.content_layout.takeAt(0)
            if child.widget():
                child.widget().deleteLater()
        
        self.update_total_media_count()
        logger.log("Gallery has been cleared.", level=LogLevel.INFO)

    def load_demo_media(self):
        demo_dir = "demo"
        if not os.path.isdir(demo_dir):
            logger.log(f"Demo directory '{demo_dir}' not found.", level=LogLevel.WARNING)
            return

        media_files = [f for f in os.listdir(demo_dir) if f.lower().endswith(('.png', '.jpg', '.jpeg', '.mp4', '.webm'))]
        media_files.sort(key=lambda s: [int(text) if text.isdigit() else text.lower() for text in re.split('([0-9]+)', s)])
        if not media_files:
            logger.log(f"No media found in demo directory '{demo_dir}'.", level=LogLevel.INFO)
            return
        
        logger.log(f"Loading {len(media_files)} demo media files.", level=LogLevel.INFO)
        for i, filename in enumerate(media_files):
            media_path = os.path.join(demo_dir, filename)
            task_name = f"Demo Task {(i % 4) // 2 + 1}"
            language = f"Lang {(i % 2) + 1}"
            prompt = f"This is a demo media file: {filename}"
            self.add_media(task_name, language, media_path, prompt)

    def add_media(self, task_name, language, media_path, prompt, thumbnail_path=None):
        if not os.path.exists(media_path):
            logger.log(f"Media path does not exist: {media_path}", level=LogLevel.WARNING)
            return

        if task_name not in self.task_groups:
            task_group = CollapsibleGroup(task_name)
            self.content_layout.addWidget(task_group)
            self.task_groups[task_name] = task_group
        
        task_group = self.task_groups[task_name]

        if language not in task_group.language_groups:
            lang_group = CollapsibleGroup(language, use_flow_layout=True, is_language_group=True, parent_group=task_group)
            task_group.add_widget(lang_group)
            task_group.language_groups[language] = lang_group

        lang_group = task_group.language_groups[language]

        # --- Optimized Thumbnail Loading ---
        pixmap = QPixmap()
        if thumbnail_path and os.path.exists(thumbnail_path):
            pixmap = QPixmap(thumbnail_path)
        else:
            # Fallback to the old, slow method if thumbnail is missing
            logger.log(f"Thumbnail not found at '{thumbnail_path}', falling back to on-the-fly generation for {media_path}", level=LogLevel.WARNING)
            pixmap = MediaThumbnail.get_thumbnail_for_media(media_path)
        # --- End Optimized Thumbnail Loading ---

        if pixmap.isNull():
            logger.log(f"Failed to load media or generate thumbnail for: {media_path}", level=LogLevel.WARNING)
            return
        
        thumbnail = MediaThumbnail(media_path, prompt, pixmap, lang_group)
        
        thumbnail.media_clicked.connect(self._on_media_clicked)
        thumbnail.delete_requested.connect(lambda: self._on_delete_requested(thumbnail, lang_group))
        thumbnail.regenerate_requested.connect(self._on_regenerate_requested)

        lang_group.add_widget(thumbnail)
        self.update_total_media_count()

    def _on_media_clicked(self, media_path):
        self.media_clicked.emit(media_path)

    def _on_regenerate_requested(self, image_data):
        dialog = RegenerateConfigDialog(image_data, self)
        if dialog.exec():
            new_values = dialog.get_values()
            
            thumbnail_to_update = None
            for group in self.task_groups.values():
                thumbnail = group.find_thumbnail_by_path(image_data['image_path'])
                if thumbnail:
                    thumbnail_to_update = thumbnail
                    break
            
            if thumbnail_to_update:
                thumbnail_to_update.set_regenerating_state(True)

            worker = RegenerateImageWorker(image_data['image_path'], new_values)
            worker.signals.finished.connect(self._on_regeneration_finished)
            worker.signals.error.connect(self._on_regeneration_error)
            self.threadpool.start(worker)

    def _on_regeneration_finished(self, old_path, new_path, thumbnail_path):
        self.image_regenerated.emit(old_path, new_path)
        for group in self.task_groups.values():
            thumbnail = group.find_thumbnail_by_path(old_path)
            if thumbnail:
                thumbnail.set_regenerating_state(False)
                
                new_pixmap = QPixmap()
                if thumbnail_path and os.path.exists(thumbnail_path):
                    new_pixmap = QPixmap(thumbnail_path)
                else:
                    # Fallback to the old, slow method if thumbnail is missing
                    new_pixmap = MediaThumbnail.get_thumbnail_for_media(new_path)

                thumbnail.update_media(new_path, new_pixmap)
                logger.log(f"Updated thumbnail for {old_path} with new image {new_path}", level=LogLevel.INFO)
                
                if old_path != new_path:
                    try:
                        # Also remove the old thumbnail if it exists
                        old_thumb_base, _ = os.path.splitext(old_path)
                        old_thumb_path = f"{old_thumb_base}_thumb.jpg"
                        if os.path.exists(old_thumb_path):
                            os.remove(old_thumb_path)

                        if os.path.exists(old_path):
                            os.remove(old_path)
                            logger.log(f"Deleted old image file: {old_path}", level=LogLevel.INFO)
                    except OSError as e:
                        logger.log(f"Error deleting old image file {old_path}: {e}", level=LogLevel.ERROR)
                break

    def _on_regeneration_error(self, old_path, error_message):
        for group in self.task_groups.values():
            thumbnail = group.find_thumbnail_by_path(old_path)
            if thumbnail:
                thumbnail.set_regenerating_state(False)
                break
        
        QMessageBox.critical(self, "Regeneration Failed", f"Could not regenerate image for '{os.path.basename(old_path)}':\n\n{error_message}")


    def _on_delete_requested(self, thumbnail_widget, lang_group):
        try:
            media_path = thumbnail_widget.media_path
            self.image_deleted.emit(media_path) # Повідомити про видалення

            is_demo_file = os.path.dirname(media_path).endswith('demo')
            if not is_demo_file:
                os.remove(media_path)
                logger.log(f"Deleted image from disk: {media_path}", level=LogLevel.INFO)
            else:
                logger.log(f"Removed demo image from gallery: {media_path}", level=LogLevel.INFO)

            lang_group.content_layout.removeWidget(thumbnail_widget)
            thumbnail_widget.deleteLater()

            lang_group.update_title()
            self.update_total_media_count()

            task_group = lang_group.parent_group
            if lang_group.get_media_count() == 0:
                task_group.content_layout.removeWidget(lang_group)
                lang_group.deleteLater()
                del task_group.language_groups[lang_group.title]

            if task_group.get_media_count() == 0:
                self.content_layout.removeWidget(task_group)
                task_group.deleteLater()
                del self.task_groups[task_group.title]

        except OSError as e:
            logger.log(f"Error deleting image file {thumbnail_widget.image_path}: {e}", level=LogLevel.ERROR)
            QMessageBox.critical(self, "Error", f"Could not delete image file:\n{e}")

    def update_total_media_count(self):
        total_count = sum(group.get_media_count() for group in self.task_groups.values())
        self.total_images_label.setText(translator.translate("gallery_total_media_label").format(count=total_count))

    def retranslate_ui(self):
        self.update_total_media_count()
        #self.load_demo_button.setText(translator.translate("load_demo_button"))
        self.continue_button.setText(translator.translate("continue_montage_button"))
        for group in self.task_groups.values():
            group.translate_ui()

    def show_continue_button(self):
        self.continue_button.show()

    def _on_continue_clicked(self):
        self.continue_montage_requested.emit()
        self.continue_button.hide()

    def update_thumbnail(self, old_path, new_path):
        """Finds a thumbnail by the old path and updates it to the new one."""
        for task_group in self.task_groups.values():
            thumbnail = task_group.find_thumbnail_by_path(old_path)
            if thumbnail:
                new_pixmap = MediaThumbnail.get_thumbnail_for_media(new_path)
                if not new_pixmap.isNull():
                    thumbnail.update_media(new_path, new_pixmap)
                    logger.log(f"Updated thumbnail from image to video: {os.path.basename(new_path)}", level=LogLevel.INFO)
                else:
                    logger.log(f"Failed to create new pixmap for {new_path}", level=LogLevel.WARNING)
                return # Found and processed, so exit