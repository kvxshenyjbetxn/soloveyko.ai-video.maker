import os
from PySide6.QtWidgets import QWidget, QVBoxLayout
from PySide6.QtGui import QColor, QPainter, QKeyEvent
from PySide6.QtCore import Qt, QUrl
from PySide6.QtMultimedia import QMediaPlayer, QAudioOutput
from PySide6.QtMultimediaWidgets import QVideoWidget

class VideoViewer(QWidget):
    def __init__(self, video_path, parent=None):
        super().__init__(parent)
        self.setAttribute(Qt.WidgetAttribute.WA_DeleteOnClose)
        self.setFocusPolicy(Qt.FocusPolicy.StrongFocus)
        self.setStyleSheet("background-color: rgba(0, 0, 0, 180);")
        self.video_path = video_path

        self.init_ui()
        self.load_video()

    def init_ui(self):
        self.main_layout = QVBoxLayout(self)
        self.main_layout.setContentsMargins(20, 20, 20, 20)

        self.video_widget = QVideoWidget()
        self.main_layout.addWidget(self.video_widget)

    def load_video(self):
        self.player = QMediaPlayer()
        self.player.setAudioOutput(None)
        self.player.setVideoOutput(self.video_widget)
        self.player.mediaStatusChanged.connect(self._on_media_status_changed)
        self.player.errorOccurred.connect(self.on_player_error)
        self.player.setSource(QUrl.fromLocalFile(os.path.abspath(self.video_path)))

    def on_player_error(self, error):
        # TODO: Log this to the main logger instead of printing
        pass

    def _on_media_status_changed(self, status):
        if status == QMediaPlayer.MediaStatus.LoadedMedia:
            self.player.play()
        elif status == QMediaPlayer.MediaStatus.EndOfMedia:
            self.player.setPosition(0)
            self.player.play()
        elif status == QMediaPlayer.MediaStatus.InvalidMedia or status == QMediaPlayer.MediaStatus.StalledMedia:
            self.on_player_error(self.player.error())

    def mousePressEvent(self, event):
        self.close()

    def keyPressEvent(self, event: QKeyEvent):
        if event.key() == Qt.Key.Key_Escape:
            self.close()
        else:
            super().keyPressEvent(event)
            
    def closeEvent(self, event):
        if self.player:
            self.player.stop()
            self.player.setVideoOutput(None)
            self.player.deleteLater()
            self.player = None
        super().closeEvent(event)
