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
        self.player.setVideoOutput(self.video_widget)
        self._audio_output = QAudioOutput()
        self.player.setAudioOutput(self._audio_output)
        self.player.setSource(QUrl.fromLocalFile(os.path.abspath(self.video_path)))
        self.player.mediaStatusChanged.connect(self._on_media_status_changed)
        self.player.errorOccurred.connect(self.on_player_error)

    def on_player_error(self, error):
        print(f"Player Error: {self.player.errorString()}")

    def _on_media_status_changed(self, status):
        if status == QMediaPlayer.MediaStatus.LoadedMedia:
            self.player.play()
        elif status == QMediaPlayer.MediaStatus.EndOfMedia:
            self.player.setPosition(0)
            self.player.play()

    def paintEvent(self, event):
        painter = QPainter(self)
        painter.fillRect(self.rect(), QColor(0, 0, 0, 180))

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
        super().closeEvent(event)
