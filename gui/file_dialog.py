from PySide6.QtWidgets import QDialog, QVBoxLayout, QLabel, QListWidget, QPushButton, QDialogButtonBox, QFileDialog, QHBoxLayout
from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QDragEnterEvent, QDropEvent
from utils.translator import translator

class FileDialog(QDialog):
    files_selected = Signal(list)

    def __init__(self, parent=None, title="Add Files", description="Drag and drop files here.", extensions=None, multi_file=True):
        super().__init__(parent)
        self.setWindowTitle(title)
        self.setMinimumSize(400, 300)
        self.setAcceptDrops(True)

        self.extensions = extensions if extensions else []
        self.multi_file = multi_file
        self.file_paths = []

        layout = QVBoxLayout(self)

        self.description_label = QLabel(description)
        self.description_label.setAlignment(Qt.AlignCenter)
        self.description_label.setWordWrap(True)
        layout.addWidget(self.description_label)

        self.file_list_widget = QListWidget()
        layout.addWidget(self.file_list_widget)

        # --- Button Layout ---
        button_layout = QHBoxLayout()

        self.browse_button = QPushButton(translator.translate("browse_button", "Browse..."))
        self.browse_button.clicked.connect(self.open_file_browser)
        button_layout.addWidget(self.browse_button)

        button_layout.addStretch()

        self.button_box = QDialogButtonBox(QDialogButtonBox.Ok | QDialogButtonBox.Cancel)
        self.button_box.accepted.connect(self.accept)
        self.button_box.rejected.connect(self.reject)
        button_layout.addWidget(self.button_box)
        
        layout.addLayout(button_layout)

    def open_file_browser(self):
        file_filter = f"Files ({' '.join(['*' + ext for ext in self.extensions])})" if self.extensions else "All Files (*)"
        
        if self.multi_file:
            files, _ = QFileDialog.getOpenFileNames(self, "Select Files", "", file_filter)
            if files:
                self.add_files(files)
        else:
            file, _ = QFileDialog.getOpenFileName(self, "Select File", "", file_filter)
            if file:
                self.add_files([file])

    def dragEnterEvent(self, event: QDragEnterEvent):
        if event.mimeData().hasUrls():
            urls = event.mimeData().urls()
            if self.extensions:
                if any(any(url.toLocalFile().lower().endswith(ext) for ext in self.extensions) for url in urls):
                    event.acceptProposedAction()
            else:
                event.acceptProposedAction()

    def dropEvent(self, event: QDropEvent):
        if event.mimeData().hasUrls():
            event.acceptProposedAction()
            urls = event.mimeData().urls()
            paths = [url.toLocalFile() for url in urls]
            self.add_files(paths)
            
    def add_files(self, new_paths):
        if not self.multi_file:
            self.file_paths.clear()

        for file_path in new_paths:
            if self.extensions:
                if any(file_path.lower().endswith(ext) for ext in self.extensions):
                    if self.multi_file:
                        if file_path not in self.file_paths:
                            self.file_paths.append(file_path)
                    else:
                        self.file_paths = [file_path]
                        break 
            else:
                if self.multi_file:
                    if file_path not in self.file_paths:
                        self.file_paths.append(file_path)
                else:
                    self.file_paths = [file_path]
                    break
        
        self.update_list_widget()

    def update_list_widget(self):
        self.file_list_widget.clear()
        self.file_list_widget.addItems(self.file_paths)

    def accept(self):
        self.files_selected.emit(self.file_paths)
        super().accept()
