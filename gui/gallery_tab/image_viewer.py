import os
from PySide6.QtWidgets import QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QStackedWidget, QLabel, QGridLayout
from PySide6.QtGui import QColor, QPainter, QKeyEvent, QPixmap
from PySide6.QtCore import Qt, QUrl

class ImageViewer(QWidget):
    def __init__(self, media_paths, current_index, parent=None):
        super().__init__(parent)
        self.setAttribute(Qt.WidgetAttribute.WA_DeleteOnClose)
        self.setFocusPolicy(Qt.FocusPolicy.StrongFocus)
        self.media_paths = media_paths
        self.current_index = current_index
        self.player = None

        self.init_ui()
        
        # Delay loading to allow the widget to get its final geometry from the parent
        from PySide6.QtCore import QTimer
        QTimer.singleShot(10, self.load_media)

    def init_ui(self):
        self.layout = QGridLayout(self)
        self.layout.setContentsMargins(0, 0, 0, 0) # No margins for maximum size
        self.layout.setSpacing(0)
        
        # Ensure the center (where media is) stretches while sides stay fixed
        self.layout.setColumnStretch(0, 0)
        self.layout.setColumnStretch(1, 1)
        self.layout.setColumnStretch(2, 0)
        self.layout.setRowStretch(0, 1)

        # Media Container (Stacked Widget) - takes the middle column and stretches
        self.media_stack = QStackedWidget()
        
        # Video Widget
        from PySide6.QtMultimediaWidgets import QVideoWidget
        self.video_widget = QVideoWidget()
        self.media_stack.addWidget(self.video_widget)
        
        # Image Widget
        self.image_label = QLabel()
        self.image_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.media_stack.addWidget(self.image_label)
        
        self.layout.addWidget(self.media_stack, 0, 0, 1, 3) # Span all columns

        # Navigation Buttons - placed in front of the media stack conceptually 
        # but technically in side columns of the grid for hit testing.
        # Actually, to keep it simple and BIG, we put buttons in Col 0 and Col 2.
        # To make image "as was", we ensure Col 1 has maximum stretch.
        
        self.prev_button = QPushButton("‹")
        self.prev_button.setFixedSize(60, 120)
        self.prev_button.setCursor(Qt.CursorShape.PointingHandCursor)
        self.prev_button.setStyleSheet(self._button_style())
        self.prev_button.clicked.connect(self.show_prev)
        self.layout.addWidget(self.prev_button, 0, 0, Qt.AlignmentFlag.AlignVCenter | Qt.AlignmentFlag.AlignLeft)

        self.next_button = QPushButton("›")
        self.next_button.setFixedSize(60, 120)
        self.next_button.setCursor(Qt.CursorShape.PointingHandCursor)
        self.next_button.setStyleSheet(self._button_style())
        self.next_button.clicked.connect(self.show_next)
        self.layout.addWidget(self.next_button, 0, 2, Qt.AlignmentFlag.AlignVCenter | Qt.AlignmentFlag.AlignRight)

    def _button_style(self):
        return """
            QPushButton {
                background-color: rgba(255, 255, 255, 25);
                color: rgba(255, 255, 255, 180);
                font-size: 50px;
                border: none;
                border-radius: 8px;
            }
            QPushButton:hover {
                background-color: rgba(255, 255, 255, 50);
                color: white;
            }
            QPushButton:disabled {
                background-color: transparent;
                color: transparent;
            }
        """

    def load_media(self):
        if self.player:
            self.player.stop()
            self.player.deleteLater()
            self.player = None

        if 0 <= self.current_index < len(self.media_paths):
            media_path = self.media_paths[self.current_index]
            is_video = media_path.lower().endswith(('.mp4', '.avi', '.mov', '.webm'))
            
            if is_video:
                self.media_stack.setCurrentWidget(self.video_widget)
                self.load_video(media_path)
            else:
                self.media_stack.setCurrentWidget(self.image_label)
                self.load_image(media_path)
            
            self.update_buttons()

    def load_video(self, video_path):
        from PySide6.QtMultimedia import QMediaPlayer
        self.player = QMediaPlayer()
        self.player.setAudioOutput(None)
        self.player.setVideoOutput(self.video_widget)
        self.player.mediaStatusChanged.connect(self._on_media_status_changed)
        self.player.setSource(QUrl.fromLocalFile(os.path.abspath(video_path)))
        self.player.play()

    def load_image(self, image_path):
        pixmap = QPixmap(image_path)
        if not pixmap.isNull():
             self.current_pixmap = pixmap
             self.update_image_display()

    def update_image_display(self):
        if hasattr(self, 'current_pixmap') and not self.current_pixmap.isNull():
             scaled = self.current_pixmap.scaled(self.image_label.size(), 
                                               Qt.AspectRatioMode.KeepAspectRatio, 
                                               Qt.TransformationMode.SmoothTransformation)
             self.image_label.setPixmap(scaled)

    def update_buttons(self):
        self.prev_button.setEnabled(self.current_index > 0)
        self.next_button.setEnabled(self.current_index < len(self.media_paths) - 1)
        # We don't hide, we let 'disabled' style handle it (transparent) to avoid layout shifts
        if len(self.media_paths) <= 1:
            self.prev_button.hide()
            self.next_button.hide()
        else:
            self.prev_button.show()
            self.next_button.show()

    def show_next(self):
        if self.current_index < len(self.media_paths) - 1:
            self.current_index += 1
            self.load_media()

    def show_prev(self):
        if self.current_index > 0:
            self.current_index -= 1
            self.load_media()

    def paintEvent(self, event):
        painter = QPainter(self)
        painter.fillRect(self.rect(), QColor(0, 0, 0, 190))

    def resizeEvent(self, event):
        super().resizeEvent(event)
        if self.media_stack.currentWidget() == self.image_label:
            self.update_image_display()

    def mousePressEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton:
            # Check if we clicked buttons first
            if self.prev_button.underMouse() or self.next_button.underMouse():
                return
            self.close()

    def keyPressEvent(self, event: QKeyEvent):
        if event.key() == Qt.Key.Key_Escape:
            self.close()
        elif event.key() == Qt.Key.Key_Left:
            self.show_prev()
        elif event.key() == Qt.Key.Key_Right:
            self.show_next()
        else:
            super().keyPressEvent(event)
            
    def _on_media_status_changed(self, status):
        from PySide6.QtMultimedia import QMediaPlayer
        if status == QMediaPlayer.MediaStatus.EndOfMedia:
            self.player.setPosition(0)
            self.player.play()

    def closeEvent(self, event):
        if self.player:
            self.player.stop()
            self.player.setVideoOutput(None)
            self.player.deleteLater()
            self.player = None
        super().closeEvent(event)
