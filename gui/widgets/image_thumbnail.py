from PySide6.QtWidgets import QWidget, QVBoxLayout, QHBoxLayout, QToolButton, QFrame
from PySide6.QtCore import Signal, QSize
from PySide6.QtGui import QIcon
from .clickable_label import ClickableLabel

class ImageThumbnail(QWidget):
    delete_requested = Signal()
    regenerate_requested = Signal()
    image_clicked = Signal()

    def __init__(self, image_path, prompt, pixmap, parent=None):
        super().__init__(parent)
        self.image_path = image_path
        self.prompt = prompt

        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(0, 0, 0, 0)
        main_layout.setSpacing(0)

        # Image
        self.image_label = ClickableLabel()
        self.image_label.setPixmap(pixmap)
        self.image_label.clicked.connect(self.image_clicked.emit)
        main_layout.addWidget(self.image_label)

        # Controls Layout
        image_width = pixmap.width()
        controls_layout = QHBoxLayout()
        controls_layout.setContentsMargins(5, 5, 5, 5)

        self.regenerate_button = QToolButton()
        self.regenerate_button.setText("Regen")
        self.regenerate_button.setToolTip("Regenerate")
        self.regenerate_button.clicked.connect(self.regenerate_requested.emit)

        self.delete_button = QToolButton()
        self.delete_button.setText("Delete")
        self.delete_button.setToolTip("Delete")
        self.delete_button.clicked.connect(self.delete_requested.emit)

        controls_layout.addWidget(self.regenerate_button)
        controls_layout.addStretch()
        controls_layout.addWidget(self.delete_button)
        
        # We wrap the layout in a QWidget to set a fixed width
        controls_container = QWidget()
        controls_container.setFixedWidth(image_width)
        controls_container.setLayout(controls_layout)
        
        main_layout.addWidget(controls_container)

    def set_pixmap(self, pixmap):
        # This method is no longer needed as pixmap is passed in constructor
        # but we keep it to avoid breaking old code if it's called somewhere else.
        self.image_label.setPixmap(pixmap)
