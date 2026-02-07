from PySide6.QtWidgets import QWidget, QVBoxLayout, QLabel, QLineEdit, QHBoxLayout, QComboBox, QWidget
from gui.widgets.help_label import HelpLabel
from utils.translator import translator
from utils.settings import settings_manager
from api.assemblyai import assembly_ai_api
from gui.widgets.setting_row import add_setting_row

class AssemblyAITab(QWidget):
    def __init__(self, main_window=None, settings_mgr=None, is_template_mode=False):
        super().__init__()
        self.settings = settings_mgr or settings_manager
        self.is_template_mode = is_template_mode
        self.init_ui()
        self.update_fields()
        self.retranslate_ui()

    def init_ui(self):
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)


        # API Key
        self.api_key_label = QLabel()
        self.api_key_input = QLineEdit()
        self.api_key_input.setPlaceholderText(translator.translate("enter_api_key"))
        self.api_key_input.textChanged.connect(self.save_api_key)
        
        def refresh_quick_panel():
            if self.window():
                 if hasattr(self.window(), 'refresh_quick_settings_panels'):
                      self.window().refresh_quick_settings_panels()

        add_setting_row(layout, self.api_key_label, self.api_key_input, "assemblyai_api_key", refresh_quick_panel, show_star=not self.is_template_mode)

        # Max Threads
        if not self.is_template_mode:
            self.max_threads_help = HelpLabel("assemblyai_max_threads")
            max_threads_layout = QHBoxLayout()
            self.max_threads_label = QLabel()
            max_label_container = QWidget()
            max_label_layout = QHBoxLayout(max_label_container)
            max_label_layout.setContentsMargins(0, 0, 0, 0)
            max_label_layout.setSpacing(5)
            max_label_layout.addWidget(self.max_threads_help)
            max_label_layout.addWidget(self.max_threads_label)

            self.max_threads_input = QComboBox()
            self.max_threads_input.addItems(["5", "100"])
            self.max_threads_input.currentIndexChanged.connect(self.save_max_threads)
            max_threads_layout.addWidget(max_label_container)
            max_threads_layout.addWidget(self.max_threads_input)
            layout.addLayout(max_threads_layout)

        # Info Link
        self.info_layout = QHBoxLayout()
        self.info_label = QLabel()
        self.link_label = QLabel('<a href="https://www.assemblyai.com/" style="color: #0078d4;">https://www.assemblyai.com/</a>')
        self.link_label.setOpenExternalLinks(True)
        self.info_layout.addWidget(self.info_label)
        self.info_layout.addWidget(self.link_label)
        self.info_layout.addStretch()
        layout.addLayout(self.info_layout)

        layout.addStretch()

    def retranslate_ui(self):
        self.api_key_label.setText(translator.translate("assemblyai_api_key"))
        if hasattr(self, 'max_threads_label'):
             self.max_threads_label.setText(translator.translate("assemblyai_max_threads"))
        self.api_key_input.setPlaceholderText(translator.translate("enter_api_key"))
        self.info_label.setText(translator.translate("assemblyai_info"))
        
        # Update hints
        if hasattr(self, 'max_threads_help'): self.max_threads_help.update_tooltip()

    def update_fields(self):
        self.api_key_input.blockSignals(True)
        api_key = self.settings.get("assemblyai_api_key", "")
        self.api_key_input.setText(api_key)
        self.api_key_input.blockSignals(False)

        if hasattr(self, 'max_threads_input'):
            self.max_threads_input.blockSignals(True)
            max_threads = self.settings.get("assemblyai_max_threads", "5")
            self.max_threads_input.setCurrentText(max_threads)
            self.max_threads_input.blockSignals(False)
        
    def save_api_key(self, key):
        self.settings.set("assemblyai_api_key", key)

    def save_max_threads(self, index):
        max_threads = self.max_threads_input.itemText(index)
        self.settings.set("assemblyai_max_threads", max_threads)
        assembly_ai_api.update_max_threads()
