import os
from PySide6.QtWidgets import QMainWindow, QTabWidget, QWidget, QComboBox, QAbstractSpinBox, QAbstractScrollArea, QSlider, QVBoxLayout, QMessageBox, QDialog, QTextEdit, QPushButton, QDialogButtonBox
from PySide6.QtCore import QCoreApplication, QEvent, QObject, Signal, QRunnable, QThreadPool
from PySide6.QtGui import QWheelEvent
from gui.qt_material import apply_stylesheet

from utils.settings import settings_manager
from utils.translator import translator
from config.version import __version__
from api.openrouter import OpenRouterAPI
from api.googler import GooglerAPI
from api.elevenlabs import ElevenLabsAPI
from api.voicemaker import VoicemakerAPI
from api.gemini_tts import GeminiTTSAPI

from gui.text_tab import TextTab
from gui.settings_tab.settings_tab import SettingsTab
from gui.log_tab import LogTab
from gui.queue_tab import QueueTab
from gui.gallery_tab.gallery_tab import GalleryTab
from gui.gallery_tab.image_viewer import ImageViewer
from gui.gallery_tab.video_viewer import VideoViewer
from core.queue_manager import QueueManager
from core.task_processor import TaskProcessor
from utils.logger import logger, LogLevel

class BalanceWorkerSignals(QObject):
    finished = Signal(float)

class GooglerUsageWorkerSignals(QObject):
    finished = Signal(dict)

class ElevenLabsBalanceWorkerSignals(QObject):
    finished = Signal(int)

class VoicemakerBalanceWorkerSignals(QObject):
    finished = Signal(int)

class GeminiTTSBalanceWorkerSignals(QObject):
    finished = Signal(float)

class BalanceWorker(QRunnable):
    def __init__(self):
        super().__init__()
        self.signals = BalanceWorkerSignals()

    def run(self):
        api = OpenRouterAPI()
        balance = api.get_balance()
        if balance is not None:
            self.signals.finished.emit(balance)

class GooglerUsageWorker(QRunnable):
    def __init__(self):
        super().__init__()
        self.signals = GooglerUsageWorkerSignals()

    def run(self):
        api = GooglerAPI()
        usage = api.get_usage()
        if usage is not None:
            self.signals.finished.emit(usage)

class ElevenLabsBalanceWorker(QRunnable):
    def __init__(self):
        super().__init__()
        self.signals = ElevenLabsBalanceWorkerSignals()

    def run(self):
        api = ElevenLabsAPI()
        balance, status = api.get_balance()
        if balance is not None:
            self.signals.finished.emit(balance)

class VoicemakerBalanceWorker(QRunnable):
    def __init__(self):
        super().__init__()
        self.signals = VoicemakerBalanceWorkerSignals()

    def run(self):
        api = VoicemakerAPI()
        balance, status = api.get_balance()
        if balance is not None:
            self.signals.finished.emit(int(balance))

class GeminiTTSBalanceWorker(QRunnable):
    def __init__(self):
        super().__init__()
        self.signals = GeminiTTSBalanceWorkerSignals()

    def run(self):
        api = GeminiTTSAPI()
        balance, status = api.get_balance()
        if balance is not None:
            self.signals.finished.emit(float(balance))

class MainWindow(QMainWindow):
    def __init__(self, app):
        super().__init__()
        self.app = app
        self.settings_manager = settings_manager
        self.translator = translator
        self.queue_manager = QueueManager()
        self.task_processor = TaskProcessor(self.queue_manager)
        self.threadpool = QThreadPool()
        self.init_ui()
        logger.log(translator.translate('app_started'), level=LogLevel.INFO)
        self.app.installEventFilter(self)

    def resizeEvent(self, event):
        super().resizeEvent(event)
        if hasattr(self, 'viewer') and self.viewer:
            try:
                if self.viewer.isVisible():
                    self.viewer.setGeometry(self.central_widget.rect())
            except RuntimeError:
                # This can happen if the C++ part of the viewer object is deleted
                # before this event is processed. We can safely ignore it.
                self.viewer = None


    def eventFilter(self, obj, event):
        if event.type() == QEvent.Type.Wheel and isinstance(obj, (QComboBox, QAbstractSpinBox, QSlider)):
            
            # Search for a scrollable ancestor.
            parent = obj.parent()
            while parent:
                if isinstance(parent, QAbstractScrollArea):
                    parent_pos = parent.mapFromGlobal(event.globalPosition())
                    new_event = QWheelEvent(
                        parent_pos, event.globalPosition(), event.pixelDelta(), event.angleDelta(),
                        event.buttons(), event.modifiers(), event.phase(), event.inverted(), event.source()
                    )
                    QCoreApplication.postEvent(parent, new_event)
                    return True
                parent = parent.parent()
            return True
        return super().eventFilter(obj, event)

    def init_ui(self):
        self.update_balance()
        self.update_title()
        self.setGeometry(100, 100, 1280, 720)

        self.central_widget = QWidget()
        self.setCentralWidget(self.central_widget)
        
        layout = QVBoxLayout(self.central_widget)
        self.tabs = QTabWidget()
        layout.addWidget(self.tabs)

        self.text_tab = TextTab(main_window=self)
        self.settings_tab = SettingsTab(main_window=self)
        self.log_tab = LogTab()
        self.queue_tab = QueueTab(parent=self.tabs, main_window=self, log_tab=self.log_tab)
        self.gallery_tab = GalleryTab()
        logger.set_log_widget(self.log_tab)
        self.review_notification_shown = False

        self.tabs.addTab(self.text_tab, self.translator.translate('text_processing_tab'))
        self.tabs.addTab(self.queue_tab, self.translator.translate('queue_tab'))
        self.tabs.addTab(self.gallery_tab, self.translator.translate('gallery_tab_title'))
        self.tabs.addTab(self.settings_tab, self.translator.translate('settings_tab'))
        self.tabs.addTab(self.log_tab, self.translator.translate('log_tab'))

        # Connect signals
        self.queue_manager.task_added.connect(self.queue_tab.add_task)
        self.queue_tab.start_processing_button.clicked.connect(self.task_processor.start_processing)
        self.task_processor.processing_finished.connect(self.show_processing_finished_dialog)
        self.task_processor.processing_finished.connect(self.update_balance)
        self.task_processor.processing_finished.connect(self.update_googler_usage)
        self.task_processor.processing_finished.connect(self.update_elevenlabs_balance)
        self.task_processor.processing_finished.connect(self.update_voicemaker_balance)
        self.task_processor.processing_finished.connect(self.update_gemini_tts_balance)
        self.task_processor.stage_status_changed.connect(self.queue_tab.update_stage_status)
        self.task_processor.stage_metadata_updated.connect(self.queue_tab.update_stage_metadata)
        self.task_processor.image_generated.connect(self.gallery_tab.add_media)
        self.task_processor.video_generated.connect(self.gallery_tab.update_thumbnail)
        self.task_processor.task_progress_log.connect(self.queue_tab.on_task_progress_log)
        self.task_processor.image_review_required.connect(self._on_image_review_required)
        self.task_processor.translation_review_required.connect(self._on_translation_review_required)
        self.gallery_tab.continue_montage_requested.connect(self.task_processor.resume_all_montages)
        self.gallery_tab.image_deleted.connect(self.task_processor._on_image_deleted)
        self.gallery_tab.media_clicked.connect(self.show_media_viewer)
        self.settings_tab.templates_tab.template_applied.connect(self.update_template_label)

        self.update_googler_usage()
        self.update_elevenlabs_balance()
        self.update_voicemaker_balance()
        self.update_gemini_tts_balance()
        
        # Apply last used template on startup
        last_template = self.settings_manager.get('last_used_template_name')
        if last_template:
            # We need to manually load it here because we can't trigger the button click easily/safely without UI effects
            from utils.settings import template_manager
            template_data = template_manager.load_template(last_template)
            if template_data:
                for key, value in template_data.items():
                    # Special handling for dictionaries to merge them instead of overwriting
                    if isinstance(value, dict) and key in self.settings_manager.settings and isinstance(self.settings_manager.settings[key], dict):
                        self.settings_manager.settings[key].update(value)
                    else:
                        self.settings_manager.settings[key] = value
                self.settings_manager.save_settings()
                
                # Update UI
                self.settings_tab.templates_tab.populate_templates_combo() # ensure combo is updated
                self.settings_tab.templates_tab.templates_combo.setCurrentText(last_template)
                self.text_tab.update_template_name(last_template)
                self.settings_tab._update_all_tabs()
                self.retranslate_ui() # Ensure translations applied if language changed
                logger.log(f"Applied last used template: {last_template}", level=LogLevel.INFO)

    def _on_image_review_required(self):
        title = translator.translate('image_review_title')
        message = translator.translate('image_review_message')
        QMessageBox.information(self, title, message)
        self.tabs.setCurrentWidget(self.gallery_tab)
        self.gallery_tab.show_continue_button()

    def _on_translation_review_required(self, task_id, translated_text):
        state = self.task_processor.task_states[task_id]
        dialog = TranslationReviewDialog(self, state, translated_text)

        def on_regenerate():
            self.task_processor.regenerate_translation(task_id)

        dialog.regenerate_requested.connect(on_regenerate)
        
        # We need a way to update the dialog with the new text
        # Let's create a signal on the task processor for this
        self.task_processor.translation_regenerated.connect(dialog.update_text)

        if dialog.exec():
            new_text = dialog.get_text()
            self.task_processor.task_states[task_id].text_for_processing = new_text
            # Manually save the reviewed text
            if state.dir_path:
                with open(os.path.join(state.dir_path, "translation_reviewed.txt"), 'w', encoding='utf-8') as f:
                    f.write(new_text)
            self.task_processor._on_text_ready(task_id)
        else:
            # Handle cancellation if needed, e.g., mark stage as failed
            self.task_processor._set_stage_status(task_id, 'stage_translation', 'error', 'User cancelled review.')
        
        # Disconnect to avoid issues with subsequent dialogs
        self.task_processor.translation_regenerated.disconnect(dialog.update_text)

    def show_media_viewer(self, media_path):
        if media_path.lower().endswith(('.mp4', '.avi', '.mov', '.webm')):
            self.viewer = VideoViewer(media_path, parent=self.central_widget)
        else:
            self.viewer = ImageViewer(media_path, self.central_widget)
        
        self.viewer.setGeometry(self.central_widget.rect())
        self.viewer.show()
        self.viewer.setFocus()

    def show_processing_finished_dialog(self, elapsed_time):
        title = translator.translate('processing_complete_title')
        message = translator.translate('processing_complete_message').format(elapsed_time)
        QMessageBox.information(self, title, message)

    def update_title(self):
        app_name = self.translator.translate('app_title')
        self.setWindowTitle(f"{app_name} v{__version__}")

    def update_template_label(self):
        name = self.settings_manager.get('last_used_template_name')
        self.text_tab.update_template_name(name)

    def update_balance(self):
        worker = BalanceWorker()
        worker.signals.finished.connect(self._on_balance_updated)
        self.threadpool.start(worker)

    def update_googler_usage(self):
        worker = GooglerUsageWorker()
        worker.signals.finished.connect(self._on_googler_usage_updated)
        self.threadpool.start(worker)

    def update_elevenlabs_balance(self):
        worker = ElevenLabsBalanceWorker()
        worker.signals.finished.connect(self._on_elevenlabs_balance_updated)
        self.threadpool.start(worker)

    def update_voicemaker_balance(self):
        worker = VoicemakerBalanceWorker()
        worker.signals.finished.connect(self._on_voicemaker_balance_updated)
        self.threadpool.start(worker)

    def update_gemini_tts_balance(self):
        worker = GeminiTTSBalanceWorker()
        worker.signals.finished.connect(self._on_gemini_tts_balance_updated)
        self.threadpool.start(worker)

    def _on_balance_updated(self, balance):
        api_key = self.settings_manager.get("openrouter_api_key")
        if api_key:
            balance_text = f"{translator.translate('balance_label')} {balance:.4f}$"
        else:
            balance_text = ""

        self.text_tab.update_balance(balance_text)
        self.queue_tab.update_balance(balance_text)
        self.settings_tab.api_tab.openrouter_tab.update_balance_label(balance_text)

    def _on_googler_usage_updated(self, usage_data):
        googler_settings = self.settings_manager.get("googler", {})
        api_key = googler_settings.get("api_key")

        if api_key and usage_data:
            img_usage = usage_data.get("current_usage", {}).get("hourly_usage", {}).get("image_generation", {})
            current_usage = img_usage.get("current_usage", "N/A")
            
            img_limits = usage_data.get("account_limits", {})
            limit = img_limits.get("img_gen_per_hour_limit", "N/A")

            usage_text = f"Googler: {current_usage} / {limit}"
        else:
            usage_text = ""

        self.text_tab.update_googler_usage(usage_text)
        self.queue_tab.update_googler_usage(usage_text)
        self.settings_tab.api_tab.image_tab.googler_tab.usage_display_label.setText(f"{current_usage} / {limit}" if api_key and usage_data else "N/A")

    def _on_elevenlabs_balance_updated(self, balance):
        api_key = self.settings_manager.get("elevenlabs_api_key")
        if api_key:
            balance_text = f"ElevenLabs: {balance}"
            balance_to_display_on_settings_tab = balance
        else:
            balance_text = ""
            balance_to_display_on_settings_tab = None

        self.text_tab.update_elevenlabs_balance(balance_text)
        self.queue_tab.update_elevenlabs_balance(balance_text)
        self.settings_tab.api_tab.audio_tab.elevenlabs_tab.update_balance_label(balance_to_display_on_settings_tab)

    def _on_voicemaker_balance_updated(self, balance):
        api_key = self.settings_manager.get("voicemaker_api_key")
        if api_key:
            balance_text = f"Voicemaker: {balance}"
            balance_to_display_on_settings_tab = balance
        else:
            balance_text = ""
            balance_to_display_on_settings_tab = None

        self.text_tab.update_voicemaker_balance(balance_text)
        self.queue_tab.update_voicemaker_balance(balance_text)
        self.settings_tab.api_tab.audio_tab.voicemaker_tab.update_balance_label(balance_to_display_on_settings_tab)

    def _on_gemini_tts_balance_updated(self, balance):
        api_key = self.settings_manager.get("gemini_tts_api_key")
        if api_key:
            balance_text = f"GeminiTTS: {balance}"
            balance_to_display_on_settings_tab = balance
        else:
            balance_text = ""
            balance_to_display_on_settings_tab = None

        self.text_tab.update_gemini_tts_balance(balance_text)
        self.queue_tab.update_gemini_tts_balance(balance_text)
        self.settings_tab.api_tab.audio_tab.gemini_tts_tab.update_balance_label(balance_to_display_on_settings_tab)

    def change_theme(self, theme_name):
        self.settings_manager.set('theme', theme_name)
        
        if theme_name == 'light':
            apply_stylesheet(self.app, theme='light_blue.xml')
        elif theme_name == 'dark':
            apply_stylesheet(self.app, theme='dark_teal.xml')
        elif theme_name == 'black':
            apply_stylesheet(self.app, theme='amoled_black.xml')

    def change_language(self, lang_code):
        self.translator.set_language(lang_code)
        self.retranslate_ui()

    def retranslate_ui(self):
        self.update_title()
        self.tabs.setTabText(self.tabs.indexOf(self.text_tab), self.translator.translate('text_processing_tab'))
        self.tabs.setTabText(self.tabs.indexOf(self.queue_tab), self.translator.translate('queue_tab'))
        self.tabs.setTabText(self.tabs.indexOf(self.gallery_tab), self.translator.translate('gallery_tab_title'))
        self.tabs.setTabText(self.tabs.indexOf(self.settings_tab), self.translator.translate('settings_tab'))
        self.tabs.setTabText(self.tabs.indexOf(self.log_tab), self.translator.translate('log_tab'))
        
        self.text_tab.retranslate_ui()
        self.settings_tab.retranslate_ui()
        self.queue_tab.retranslate_ui()
        self.gallery_tab.retranslate_ui()

    def closeEvent(self, event):
        self.settings_tab.api_tab.image_tab.pollinations_tab.save_settings()
        self.settings_tab.api_tab.image_tab.googler_tab.save_settings()
        logger.log(translator.translate('app_closing'), level=LogLevel.INFO)
        super().closeEvent(event)

class TranslationReviewDialog(QDialog):
    regenerate_requested = Signal()

    def __init__(self, parent, state, text):
        super().__init__(parent)
        job_name = state.job_name
        lang_name = state.lang_name
        self.setWindowTitle(f"Перевірка перекладу: {job_name} ({lang_name})")
        self.setMinimumSize(700, 500)

        layout = QVBoxLayout(self)

        self.text_edit = QTextEdit()
        self.text_edit.setPlainText(text)
        layout.addWidget(self.text_edit)

        self.button_box = QDialogButtonBox()
        self.regenerate_button = self.button_box.addButton("Перегенерувати", QDialogButtonBox.ButtonRole.ActionRole)
        self.ok_button = self.button_box.addButton(QDialogButtonBox.StandardButton.Ok)
        self.cancel_button = self.button_box.addButton(QDialogButtonBox.StandardButton.Cancel)
        
        self.regenerate_button.clicked.connect(self.regenerate_requested.emit)
        self.ok_button.clicked.connect(self.accept)
        self.cancel_button.clicked.connect(self.reject)
        
        layout.addWidget(self.button_box)

    def get_text(self):
        return self.text_edit.toPlainText()

    def update_text(self, task_id, new_text):
        # We only update if the task_id matches
        state = self.parent().task_processor.task_states.get(task_id)
        if state and self.windowTitle() == f"Перевірка перекладу: {state.job_name} ({state.lang_name})":
            self.text_edit.setPlainText(new_text)

