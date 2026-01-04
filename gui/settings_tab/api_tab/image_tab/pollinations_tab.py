import logging
from PySide6.QtWidgets import QWidget, QFormLayout, QLabel, QComboBox, QLineEdit, QSpinBox, QCheckBox, QHBoxLayout
from PySide6.QtCore import QThread, Signal
from utils.settings import settings_manager
from utils.translator import translator
from api.pollinations import PollinationsAPI

class ModelFetcher(QThread):
    models_fetched = Signal(list)

    def run(self):
        api = PollinationsAPI()
        models = api.get_models()
        self.models_fetched.emit(models)

class PollinationsTab(QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
        # Default fallback models
        self.models = ["flux", "flux-realism", "flux-3d", "flux-cablyai", "dall-e-3", "midjourney", "boreal"] 
        self.initUI()
        self.update_fields()
        self.connect_signals()
        
        # Start fetching models in background
        self.fetch_models()

    def fetch_models(self):
        self.fetcher = ModelFetcher()
        self.fetcher.models_fetched.connect(self.update_models_list)
        self.fetcher.start()

    def update_models_list(self, models):
        if models:
            self.models = models
            current_model = self.model_combo.currentText()
            
            self.model_combo.blockSignals(True)
            self.model_combo.clear()
            self.model_combo.addItems(self.models)
            
            # Restore selection if it exists in new list, otherwise default to first
            if current_model in self.models:
                self.model_combo.setCurrentText(current_model)
            elif "flux" in self.models:
                self.model_combo.setCurrentText("flux")
                
            self.model_combo.blockSignals(False)
            
            # If the current selection changed (because previous one wasn't in list), save it
            if self.model_combo.currentText() != current_model:
                self.save_settings()

    def initUI(self):
        layout = QFormLayout(self)

        # Model
        self.model_label = QLabel()
        self.model_combo = QComboBox()
        self.model_combo.addItems(self.models)
        layout.addRow(self.model_label, self.model_combo)

        # Token
        self.token_label = QLabel()
        self.token_input = QLineEdit()
        layout.addRow(self.token_label, self.token_input)

        # Width and Height
        self.size_label = QLabel()
        size_layout = QHBoxLayout()
        self.width_spinbox = QSpinBox()
        self.width_spinbox.setRange(1, 4096)
        self.height_spinbox = QSpinBox()
        self.height_spinbox.setRange(1, 4096)
        size_layout.addWidget(self.width_spinbox)
        size_layout.addWidget(QLabel("x"))
        size_layout.addWidget(self.height_spinbox)
        layout.addRow(self.size_label, size_layout)

        # NoLogo
        self.nologo_checkbox = QCheckBox()
        layout.addRow(self.nologo_checkbox)

        # Enhance
        self.enhance_checkbox = QCheckBox()
        layout.addRow(self.enhance_checkbox)

        # Info Link
        self.info_layout = QHBoxLayout()
        self.info_label = QLabel()
        self.link_label = QLabel('<a href="https://pollinations.ai/play" style="color: #0078d4;">https://pollinations.ai/play</a>')
        self.link_label.setOpenExternalLinks(True)
        self.info_layout.addWidget(self.info_label)
        self.info_layout.addWidget(self.link_label)
        self.info_layout.addStretch()
        layout.addRow("", self.info_layout)

        self.setLayout(layout)

    def connect_signals(self):
        self.model_combo.currentIndexChanged.connect(self.save_settings)
        self.token_input.textChanged.connect(self.save_settings)
        self.width_spinbox.valueChanged.connect(self.save_settings)
        self.height_spinbox.valueChanged.connect(self.save_settings)
        self.nologo_checkbox.stateChanged.connect(self.save_settings)
        self.enhance_checkbox.stateChanged.connect(self.save_settings)
        
    def translate_ui(self):
        self.model_label.setText(translator.translate("pollinations_model_label"))
        self.token_label.setText(translator.translate("pollinations_token_label"))
        self.token_input.setPlaceholderText(translator.translate("pollinations_token_placeholder"))
        self.size_label.setText(translator.translate("image_size_label"))
        self.nologo_checkbox.setText(translator.translate("nologo_label"))
        self.enhance_checkbox.setText(translator.translate("enhance_prompt_label"))
        self.info_label.setText(translator.translate("pollinations_info"))

    def update_fields(self):
        self.model_combo.blockSignals(True)
        self.token_input.blockSignals(True)
        self.width_spinbox.blockSignals(True)
        self.height_spinbox.blockSignals(True)
        self.nologo_checkbox.blockSignals(True)
        self.enhance_checkbox.blockSignals(True)

        pollinations_settings = settings_manager.get("pollinations", {})
        
        # If model is not in current list (which might be just defaults initially), 
        # add it temporarily so it shows up, or just select it if we can
        saved_model = pollinations_settings.get("model", "flux")
        
        # Note: We rely on fetching to populate the full list. 
        # If the saved model is not in the default list, we might want to add it or just wait for fetch.
        # But if we just start, self.models is defaults. 
        if saved_model not in self.models:
             # Just set it, QComboBox might ignore or set to index -1? 
             # Actually QComboBox.setCurrentText works if item exists.
             # If it doesn't exist, we might want to add it?
             # Let's add it if missing from defaults.
             self.models.append(saved_model)
             self.model_combo.addItem(saved_model)
             
        self.model_combo.setCurrentText(saved_model)
        self.token_input.setText(pollinations_settings.get("token", ""))
        self.width_spinbox.setValue(pollinations_settings.get("width", 1280))
        self.height_spinbox.setValue(pollinations_settings.get("height", 720))
        self.nologo_checkbox.setChecked(pollinations_settings.get("nologo", True))
        self.enhance_checkbox.setChecked(pollinations_settings.get("enhance", False))

        self.model_combo.blockSignals(False)
        self.token_input.blockSignals(False)
        self.width_spinbox.blockSignals(False)
        self.height_spinbox.blockSignals(False)
        self.nologo_checkbox.blockSignals(False)
        self.enhance_checkbox.blockSignals(False)

    def save_settings(self):
        pollinations_settings = {
            "model": self.model_combo.currentText(),
            "token": self.token_input.text(),
            "width": self.width_spinbox.value(),
            "height": self.height_spinbox.value(),
            "nologo": self.nologo_checkbox.isChecked(),
            "enhance": self.enhance_checkbox.isChecked(),
        }
        settings_manager.set("pollinations", pollinations_settings)