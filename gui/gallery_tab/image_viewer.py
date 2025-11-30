from PySide6.QtWidgets import QWidget, QLabel, QVBoxLayout
from PySide6.QtGui import QPixmap, QColor, QPainter, QKeyEvent
from PySide6.QtCore import Qt

class ImageViewer(QWidget):
    def __init__(self, image_path, parent=None):
        super().__init__(parent)
        self.setAttribute(Qt.WidgetAttribute.WA_DeleteOnClose)
        self.setFocusPolicy(Qt.FocusPolicy.StrongFocus)
        self.image_path = image_path

        self.init_ui()
        self.load_image()

    def init_ui(self):
        self.main_layout = QVBoxLayout(self)
        self.main_layout.setContentsMargins(20, 20, 20, 20)

        self.image_label = QLabel()
        self.image_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.main_layout.addWidget(self.image_label)

    def load_image(self):
        self.pixmap = QPixmap(self.image_path)
        self.update_image_display()

    def paintEvent(self, event):
        painter = QPainter(self)
        painter.fillRect(self.rect(), QColor(0, 0, 0, 180)) # Semi-transparent background

    def resizeEvent(self, event):
        super().resizeEvent(event)
        self.update_image_display()

    def update_image_display(self):
        if not self.pixmap.isNull():
            scaled_pixmap = self.pixmap.scaled(self.image_label.size(), 
                                               Qt.AspectRatioMode.KeepAspectRatio, 
                                               Qt.TransformationMode.SmoothTransformation)
            self.image_label.setPixmap(scaled_pixmap)
            
    def mousePressEvent(self, event):
        self.close()

    def keyPressEvent(self, event: QKeyEvent):
        if event.key() == Qt.Key.Key_Escape:
            self.close()
        else:
            super().keyPressEvent(event)
