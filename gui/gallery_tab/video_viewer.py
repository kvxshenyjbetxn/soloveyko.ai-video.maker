import os
from PySide6.QtWidgets import QWidget, QVBoxLayout
from PySide6.QtGui import QColor, QPainter, QKeyEvent
from PySide6.QtCore import Qt, QUrl
from PySide6.QtMultimedia import QMediaPlayer, QAudioOutput
from PySide6.QtMultimediaWidgets import QVideoWidget

class VideoViewer(QWidget):
    def __init__(self, video_path, player=None, parent=None):
        super().__init__(parent)
        self.setAttribute(Qt.WidgetAttribute.WA_DeleteOnClose)
        self.setFocusPolicy(Qt.FocusPolicy.StrongFocus)
        self.setStyleSheet("background-color: rgba(0, 0, 0, 180);")
        self.video_path = video_path
        self.shared_player = player

        self.init_ui()
        self.load_video()

    def init_ui(self):
        self.main_layout = QVBoxLayout(self)
        self.main_layout.setContentsMargins(20, 20, 20, 20)

        self.video_widget = QVideoWidget()
        self.main_layout.addWidget(self.video_widget)

    def load_video(self):
        if self.shared_player:
            self.player = self.shared_player
            # Unmute audio for fullscreen
            self.player.audioOutput().setMuted(False)
        else:
            self.player = QMediaPlayer()
            self._audio_output = QAudioOutput()
            self.player.setAudioOutput(self._audio_output)
            self.player.setSource(QUrl.fromLocalFile(os.path.abspath(self.video_path)))
        
        self.player.setVideoOutput(self.video_widget)
        self.player.mediaStatusChanged.connect(self._on_media_status_changed)
        self.player.errorOccurred.connect(self.on_player_error)

        if self.player.mediaStatus() == QMediaPlayer.MediaStatus.LoadedMedia:
            self.player.play()
        # If not loaded, the status change will trigger play

    def on_player_error(self, error):
        print(f"VideoViewer Player Error ({error}): {self.player.errorString()}")

    def _on_media_status_changed(self, status):
        print(f"VideoViewer Media Status Changed: {status}")
        if status == QMediaPlayer.MediaStatus.LoadedMedia:
            self.player.play()
        elif status == QMediaPlayer.MediaStatus.EndOfMedia:
            self.player.setPosition(0)
            self.player.play()
        elif status == QMediaPlayer.MediaStatus.InvalidMedia:
            print(f"VideoViewer: Invalid media file: {self.video_path}")
            self.on_player_error(self.player.error())
        elif status == QMediaPlayer.MediaStatus.StalledMedia:
            print(f"VideoViewer: Media stalled for file: {self.video_path}")
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
            if self.shared_player:
                # Mute the audio again for thumbnail preview
                self.player.audioOutput().setMuted(True)
        super().closeEvent(event)
