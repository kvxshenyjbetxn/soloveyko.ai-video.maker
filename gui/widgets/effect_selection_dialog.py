import os
import sys
import shutil
from PySide6.QtWidgets import (
    QDialog, QHBoxLayout, QVBoxLayout, QListWidget, QPushButton, 
    QFileDialog, QLabel, QListWidgetItem, QMessageBox
)
from PySide6.QtMultimedia import QMediaPlayer, QAudioOutput
from PySide6.QtMultimediaWidgets import QVideoWidget
from PySide6.QtCore import QUrl, Qt
from utils.translator import translator

if getattr(sys, 'frozen', False):
    BASE_PATH = sys._MEIPASS
else:
    BASE_PATH = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))

class EffectSelectionDialog(QDialog):
    def __init__(self, parent=None, initial_selection=None):
        super().__init__(parent)
        self.setWindowTitle(translator.translate("effect_selection_title", "Select Overlay Effect"))
        self.resize(800, 500)
        
        self.initial_selection = initial_selection
        self.selected_effect_path = None
        
        # Визначаємо шлях до папки efects
        if getattr(sys, 'frozen', False):
            # Коли скомпільовано: efects поруч з exe файлом
            exe_dir = os.path.dirname(sys.executable)
            self.assets_dir = os.path.join(exe_dir, "efects")
        else:
            # В розробці: assets/efects
            self.assets_dir = os.path.join(BASE_PATH, "assets", "efects")
        
        if not os.path.exists(self.assets_dir):
            try:
                os.makedirs(self.assets_dir)
            except OSError:
                pass # Might be read-only if frozen, but we can't do much

        self.init_ui()
        self.load_effects()

    def init_ui(self):
        main_layout = QHBoxLayout(self)

        # --- Left Panel: List of Effects ---
        left_layout = QVBoxLayout()
        
        self.effect_list = QListWidget()
        self.effect_list.currentItemChanged.connect(self.on_effect_selected)
        left_layout.addWidget(self.effect_list)
        
        self.add_custom_btn = QPushButton(translator.translate("add_custom_effect_button", "Add Custom Effect"))
        self.add_custom_btn.clicked.connect(self.add_custom_effect)
        left_layout.addWidget(self.add_custom_btn)
        
        main_layout.addLayout(left_layout, 1)

        # --- Right Panel: Preview ---
        right_layout = QVBoxLayout()
        
        self.video_widget = QVideoWidget()
        self.video_widget.setStyleSheet("background-color: black;")
        right_layout.addWidget(self.video_widget)
        
        self.player = QMediaPlayer()
        self.audio_output = QAudioOutput()
        self.player.setAudioOutput(self.audio_output)
        self.audio_output.setVolume(0) # Mute effects by default
        self.player.setVideoOutput(self.video_widget)
        self.player.mediaStatusChanged.connect(self._on_media_status_changed)
        
        buttons_layout = QHBoxLayout()
        self.select_btn = QPushButton(translator.translate("select_effect_button", "Select"))
        self.select_btn.clicked.connect(self.accept_selection)
        self.cancel_btn = QPushButton(translator.translate("close_button", "Close"))
        self.cancel_btn.clicked.connect(self.reject)
        
        buttons_layout.addStretch()
        buttons_layout.addWidget(self.select_btn)
        buttons_layout.addWidget(self.cancel_btn)
        
        right_layout.addLayout(buttons_layout)
        
        main_layout.addLayout(right_layout, 2)

    def load_effects(self):
        self.effect_list.clear()
        
        if not os.path.exists(self.assets_dir):
            return

        files = sorted(os.listdir(self.assets_dir))
        for f in files:
            if f.lower().endswith(('.mov', '.mp4', '.webm', '.avi')):
                item = QListWidgetItem(f)
                full_path = os.path.join(self.assets_dir, f)
                item.setData(Qt.UserRole, full_path)
                self.effect_list.addItem(item)
                
                # Pre-select if matches initial
                if self.initial_selection and os.path.normpath(self.initial_selection) == os.path.normpath(full_path):
                    self.effect_list.setCurrentItem(item)

    def on_effect_selected(self, current, previous):
        if not current:
            self.player.stop()
            self.selected_effect_path = None
            return
            
        path = current.data(Qt.UserRole)
        self.selected_effect_path = path
        self.player.setSource(QUrl.fromLocalFile(path))
        self.player.play()

    def _on_media_status_changed(self, status):
        # Loop the video
        if status == QMediaPlayer.MediaStatus.EndOfMedia:
            self.player.setPosition(0)
            self.player.play()

    def add_custom_effect(self):
        file_path, _ = QFileDialog.getOpenFileName(
            self,
            translator.translate("add_custom_effect_button", "Select Video Effect"),
            "",
            translator.translate("custom_effect_filter", "Video Files (*.mov *.mp4 *.webm)")
        )
        
        if file_path:
            filename = os.path.basename(file_path)
            dest_path = os.path.join(self.assets_dir, filename)
            
            try:
                if os.path.abspath(file_path) != os.path.abspath(dest_path):
                    shutil.copy2(file_path, dest_path)
                self.load_effects()
                
                # Select the new item
                items = self.effect_list.findItems(filename, Qt.MatchExactly)
                if items:
                    self.effect_list.setCurrentItem(items[0])
                    
            except Exception as e:
                QMessageBox.critical(self, translator.translate("error"), f"Failed to copy file: {str(e)}")

    def accept_selection(self):
        if self.selected_effect_path:
            self.accept()
        else:
            QMessageBox.warning(self, translator.translate("warning"), translator.translate("no_effect_selected", "No effect selected"))

    def get_selected_effect(self):
        return self.selected_effect_path

    def closeEvent(self, event):
        self.player.stop()
        self.player.setVideoOutput(None) 
        super().closeEvent(event)
