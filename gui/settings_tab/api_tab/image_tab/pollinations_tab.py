from PySide6.QtCore import QThread, Signal
from utils.settings import settings_manager
from utils.translator import translator
from api.pollinations import PollinationsAPI
from gui.widgets.help_label import HelpLabel
from PySide6.QtWidgets import QWidget, QFormLayout, QLabel, QComboBox, QLineEdit, QSpinBox, QCheckBox, QHBoxLayout

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
        self.model_help = HelpLabel("pollinations_model_label")
        self.model_label = QLabel()
        model_label_container = QWidget()
        model_label_layout = QHBoxLayout(model_label_container)
        model_label_layout.setContentsMargins(0, 0, 0, 0)
        model_label_layout.setSpacing(5)
        model_label_layout.addWidget(self.model_help)
        model_label_layout.addWidget(self.model_label)
        
        self.model_combo = QComboBox()
        self.model_combo.addItems(self.models)
        layout.addRow(model_label_container, self.model_combo)

        # Token (API)
        self.token_help = HelpLabel("pollinations_token_label")
        self.token_label = QLabel()
        token_label_container = QWidget()
        token_label_layout = QHBoxLayout(token_label_container)
        token_label_layout.setContentsMargins(0, 0, 0, 0)
        token_label_layout.setSpacing(5)
        token_label_layout.addWidget(self.token_help)
        token_label_layout.addWidget(self.token_label)
        
        self.token_input = QLineEdit()
        layout.addRow(token_label_container, self.token_input)

        # Width and Height
        self.size_help = HelpLabel("image_size_label")
        self.size_label = QLabel()
        size_label_container = QWidget()
        size_label_layout = QHBoxLayout(size_label_container)
        size_label_layout.setContentsMargins(0, 0, 0, 0)
        size_label_layout.setSpacing(5)
        size_label_layout.addWidget(self.size_help)
        size_label_layout.addWidget(self.size_label)
        
        size_layout = QHBoxLayout()
        self.width_spinbox = QSpinBox()
        self.width_spinbox.setRange(1, 4096)
        self.height_spinbox = QSpinBox()
        self.height_spinbox.setRange(1, 4096)
        size_layout.addWidget(self.width_spinbox)
        size_layout.addWidget(QLabel("x"))
        size_layout.addWidget(self.height_spinbox)
        layout.addRow(size_label_container, size_layout)

        # NoLogo
        self.nologo_help = HelpLabel("nologo_label")
        self.nologo_checkbox = QCheckBox()
        nologo_container = QWidget()
        nologo_layout = QHBoxLayout(nologo_container)
        nologo_layout.setContentsMargins(0, 0, 0, 0)
        nologo_layout.setSpacing(5)
        nologo_layout.addWidget(self.nologo_help)
        nologo_layout.addWidget(self.nologo_checkbox)
        layout.addRow(nologo_container)

        # Enhance
        self.enhance_help = HelpLabel("enhance_prompt_label")
        self.enhance_checkbox = QCheckBox()
        enhance_container = QWidget()
        enhance_layout = QHBoxLayout(enhance_container)
        enhance_layout.setContentsMargins(0, 0, 0, 0)
        enhance_layout.setSpacing(5)
        enhance_layout.addWidget(self.enhance_help)
        enhance_layout.addWidget(self.enhance_checkbox)
        layout.addRow(enhance_container)

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
        
        # Update hints
        self.model_help.update_tooltip()
        self.token_help.update_tooltip()
        self.size_help.update_tooltip()
        self.nologo_help.update_tooltip()
        self.enhance_help.update_tooltip()

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