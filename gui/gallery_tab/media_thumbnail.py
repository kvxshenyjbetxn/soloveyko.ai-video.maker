import os
from PySide6.QtWidgets import QWidget, QVBoxLayout, QHBoxLayout, QToolButton, QStackedWidget
from PySide6.QtCore import Signal, QSize, Qt, QUrl, QEvent
from PySide6.QtGui import QIcon, QPixmap, QImage
from PySide6.QtMultimedia import QMediaPlayer, QAudioOutput
from PySide6.QtMultimediaWidgets import QVideoWidget
import cv2

from .clickable_label import ClickableLabel
from .loading_spinner import LoadingSpinner
from utils.translator import translator

class MediaThumbnail(QWidget):
    delete_requested = Signal()
    regenerate_requested = Signal(dict)
    media_clicked = Signal(str, object)

    def __init__(self, media_path, prompt, pixmap, parent_group, parent=None):
        super().__init__(parent)
        self.media_path = media_path
        self.prompt = prompt
        self.parent_group = parent_group
        self.is_video = media_path.lower().endswith(('.mp4', '.avi', '.mov', '.webm'))

        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(0, 0, 0, 0)
        main_layout.setSpacing(0)

        self.media_stack = QStackedWidget()
        self.media_stack.setFixedSize(pixmap.size())
        main_layout.addWidget(self.media_stack)

        self.thumbnail_label = ClickableLabel()
        self.thumbnail_label.setPixmap(pixmap)
        self.thumbnail_label.clicked.connect(self.on_click)
        self.media_stack.addWidget(self.thumbnail_label)

        self.player = None
        self.video_widget = None
        if self.is_video:
            self.video_widget = QVideoWidget()
            self.media_stack.addWidget(self.video_widget)
            self._setup_video_player() # Eagerly create and load

        self.spinner = LoadingSpinner(self.media_stack)
        self.spinner.setFixedSize(pixmap.size())
        self.spinner.stop()

        self._setup_controls(pixmap.width())
        
        controls_container = QWidget()
        controls_container.setFixedWidth(pixmap.width())
        controls_container.setLayout(self.controls_layout)
        main_layout.addWidget(controls_container)

    def _setup_video_player(self):
        self.video_widget.installEventFilter(self)
        self.player = QMediaPlayer()
        self.player.setVideoOutput(self.video_widget)
        self._audio_output = QAudioOutput()
        self.player.setAudioOutput(self._audio_output)
        self._audio_output.setMuted(True)
        self.player.mediaStatusChanged.connect(self._on_media_status_changed)
        self.player.setSource(QUrl.fromLocalFile(os.path.abspath(self.media_path)))

    def on_click(self):
        player_instance = self.player if self.is_video else None
        if player_instance:
            player_instance.stop()
            self.media_stack.setCurrentIndex(self.media_stack.indexOf(self.thumbnail_label))
        self.media_clicked.emit(self.media_path, player_instance)

    def eventFilter(self, obj, event):
        if obj is self.video_widget and event.type() == QEvent.Type.MouseButtonPress:
            self.on_click()
            return True
        return super().eventFilter(obj, event)

    def _on_media_status_changed(self, status):
        if status == QMediaPlayer.MediaStatus.EndOfMedia:
            self.player.setPosition(0)
            self.player.play()

    def _setup_controls(self, image_width):
        self.controls_layout = QHBoxLayout()
        self.controls_layout.setContentsMargins(5, 5, 5, 5)
        self.controls_layout.setSpacing(5)

        icon_size = QSize(18, 18)

        self.regenerate_button = QToolButton()
        self.regenerate_button.setText(translator.translate("thumbnail_regen_button"))
        self.regenerate_button.setToolButtonStyle(Qt.ToolButtonTextOnly)
        self.regenerate_button.setFixedHeight(icon_size.height() + 8)
        self.regenerate_button.setToolTip(translator.translate("thumbnail_regen_button"))
        self.regenerate_button.setStyleSheet("QToolButton { padding: 1px 5px; border: 1px solid #888; border-radius: 4px; }")
        self.regenerate_button.clicked.connect(lambda: self.regenerate_requested.emit({'image_path': self.media_path, 'prompt': self.prompt}))
        if self.is_video:
            self.regenerate_button.hide()

        self.delete_button = QToolButton()
        self.delete_button.setIcon(QIcon("gui/qt_material/resources/source/close.svg"))
        self.delete_button.setIconSize(icon_size)
        self.delete_button.setFixedSize(icon_size + QSize(8, 8))
        self.delete_button.setToolTip(translator.translate("thumbnail_delete_button"))
        self.delete_button.setStyleSheet("QToolButton { border: 1px solid #888; border-radius: 4px; } QToolButton:hover { background-color: #E53935; border-color: #D32F2F; }")
        self.delete_button.clicked.connect(self.delete_requested.emit)

        self.controls_layout.addWidget(self.regenerate_button)
        self.controls_layout.addStretch()
        self.controls_layout.addWidget(self.delete_button)

    def enterEvent(self, event):
        if self.is_video and self.player:
            if not self.player.videoOutput():
                self.player.setVideoOutput(self.video_widget)
            
            self.media_stack.setCurrentIndex(self.media_stack.indexOf(self.video_widget))
            self.player.play()

        super().enterEvent(event)

    def leaveEvent(self, event):
        if self.is_video and self.player:
            self.player.stop()
            self.media_stack.setCurrentIndex(self.media_stack.indexOf(self.thumbnail_label))
        super().leaveEvent(event)

    def set_regenerating_state(self, is_regenerating):
        if is_regenerating:
            self.spinner.start()
            self.regenerate_button.setEnabled(False)
            self.delete_button.setEnabled(False)
        else:
            self.spinner.stop()
            self.regenerate_button.setEnabled(True)
            self.delete_button.setEnabled(True)

    def update_media(self, new_path, new_pixmap):
        self.media_path = new_path
        self.is_video = new_path.lower().endswith(('.mp4', '.avi', '.mov', '.webm'))
        
        self.thumbnail_label.setPixmap(new_pixmap)
        self.media_stack.setFixedSize(new_pixmap.size())

        if self.is_video:
            if not self.video_widget:
                self.video_widget = QVideoWidget()
                self.media_stack.addWidget(self.video_widget)
            if self.player:
                self.player.setSource(QUrl.fromLocalFile(os.path.abspath(self.media_path)))
            self.regenerate_button.hide()
        else:
            if self.player:
                self.player.stop()
                self.player.setVideoOutput(None)
                self.player.deleteLater()
                self.player = None
            if self.video_widget:
                self.media_stack.removeWidget(self.video_widget)
                self.video_widget.deleteLater()
                self.video_widget = None
            self.regenerate_button.show()
        
        self.parent_group.sort_thumbnails()

    def set_pixmap(self, pixmap):
        self.thumbnail_label.setPixmap(pixmap)

    def retranslate_ui(self):
        self.regenerate_button.setText(translator.translate("thumbnail_regen_button"))
        self.regenerate_button.setToolTip(translator.translate("thumbnail_regen_button"))
        self.delete_button.setToolTip(translator.translate("thumbnail_delete_button"))

    @staticmethod
    def get_thumbnail_for_media(media_path):
        if media_path.lower().endswith(('.mp4', '.avi', '.mov', '.webm')):
            try:
                vid = cv2.VideoCapture(media_path)
                if not vid.isOpened():
                    return QPixmap()

                success, frame = vid.read()
                vid.release()
                if success:
                    height, width, channel = frame.shape
                    bytes_per_line = 3 * width
                    q_img = QImage(frame.data, width, height, bytes_per_line, QImage.Format.Format_BGR888)
                    pixmap = QPixmap.fromImage(q_img)
                    return pixmap.scaled(200, 200, Qt.AspectRatioMode.KeepAspectRatio, Qt.TransformationMode.SmoothTransformation)
            except Exception:
                pass

        pixmap = QPixmap(media_path)
        if pixmap.isNull():
             return QPixmap()
        return pixmap.scaled(200, 200, Qt.AspectRatioMode.KeepAspectRatio, Qt.TransformationMode.SmoothTransformation)