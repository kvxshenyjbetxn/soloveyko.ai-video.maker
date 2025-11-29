from PySide6.QtWidgets import QWidget, QVBoxLayout, QScrollArea, QLabel
from PySide6.QtCore import Qt
from PySide6.QtGui import QPixmap
from utils.translator import translator
from gui.collapsible_group import CollapsibleGroup

class GalleryTab(QWidget):
    def __init__(self):
        super().__init__()
        self.task_groups = {}
        self.init_ui()
        self.retranslate_ui()

    def init_ui(self):
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(10, 10, 10, 10)
        main_layout.setSpacing(10)

        # Total images count label
        self.total_images_label = QLabel()
        main_layout.addWidget(self.total_images_label, alignment=Qt.AlignmentFlag.AlignRight)

        scroll_area = QScrollArea()
        scroll_area.setWidgetResizable(True)
        scroll_area.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        main_layout.addWidget(scroll_area)

        self.content_widget = QWidget()
        scroll_area.setWidget(self.content_widget)

        self.content_layout = QVBoxLayout(self.content_widget)
        self.content_layout.setAlignment(Qt.AlignmentFlag.AlignTop)

        self.update_total_images_count()

    def add_image(self, task_name, image_path):
        if task_name not in self.task_groups:
            group = CollapsibleGroup(task_name)
            self.content_layout.addWidget(group)
            self.task_groups[task_name] = group
        
        group = self.task_groups[task_name]
        
        # Create a widget to display the image
        image_label = QLabel()
        pixmap = QPixmap(image_path)
        # Scale image to a thumbnail size, keeping aspect ratio
        image_label.setPixmap(pixmap.scaled(200, 200, Qt.AspectRatioMode.KeepAspectRatio, Qt.TransformationMode.SmoothTransformation))
        
        group.add_widget(image_label)
        self.update_total_images_count()

    def update_total_images_count(self):
        total_count = sum(group.get_image_count() for group in self.task_groups.values())
        self.total_images_label.setText(translator.translate("gallery_total_images_label").format(count=total_count))

    def retranslate_ui(self):
        self.update_total_images_count()
        for group in self.task_groups.values():
            group.translate_ui()
