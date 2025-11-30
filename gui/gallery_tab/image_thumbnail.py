from PySide6.QtWidgets import QWidget, QVBoxLayout, QHBoxLayout, QToolButton
from PySide6.QtCore import Signal, QSize, Qt
from PySide6.QtGui import QIcon
from .clickable_label import ClickableLabel
from utils.translator import translator

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
        controls_layout.setSpacing(5)

        icon_size = QSize(18, 18)

        self.regenerate_button = QToolButton()
        self.regenerate_button.setText(translator.translate("thumbnail_regen_button"))
        self.regenerate_button.setToolButtonStyle(Qt.ToolButtonTextOnly)
        regen_button_height = icon_size.height() + 8 # Match delete button height
        # Use a sensible width based on text, but enforce height
        self.regenerate_button.setFixedHeight(regen_button_height)
        self.regenerate_button.setToolTip(translator.translate("thumbnail_regen_button"))
        self.regenerate_button.setStyleSheet("""
            QToolButton { 
                padding: 1px 5px; 
                border: 1px solid #888;
                border-radius: 4px; 
            }
        """)
        self.regenerate_button.clicked.connect(self.regenerate_requested.emit)

        self.delete_button = QToolButton()
        self.delete_button.setIcon(QIcon("gui/qt_material/resources/source/close.svg"))
        self.delete_button.setIconSize(icon_size)
        self.delete_button.setFixedSize(icon_size + QSize(8, 8)) # Icon size + padding
        self.delete_button.setToolTip(translator.translate("thumbnail_delete_button"))
        self.delete_button.setStyleSheet("""
            QToolButton { 
                border: 1px solid #888;
                border-radius: 4px;
            }
            QToolButton:hover { 
                background-color: #E53935;
                border-color: #D32F2F;
                color: white; /* Зробити іконку білою при наведенні для контрасту */
            }
        """)
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
        self.image_label.setPixmap(pixmap)

    def retranslate_ui(self):
        self.regenerate_button.setText(translator.translate("thumbnail_regen_button"))
        self.regenerate_button.setToolTip(translator.translate("thumbnail_regen_button"))
        self.delete_button.setToolTip(translator.translate("thumbnail_delete_button"))
