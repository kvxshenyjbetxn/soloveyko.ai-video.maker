import os
import requests
from datetime import datetime
from PySide6.QtWidgets import QMainWindow, QTabWidget, QWidget, QComboBox, QAbstractSpinBox, QAbstractScrollArea, QSlider, QVBoxLayout, QMessageBox, QDialog, QTextEdit, QPushButton, QDialogButtonBox, QLabel, QHBoxLayout, QMenu
from PySide6.QtCore import QCoreApplication, QEvent, QObject, Signal, QRunnable, QThreadPool, Qt, QSize, QByteArray, QTimer
from PySide6.QtGui import QWheelEvent, QIcon, QAction, QPixmap
from gui.widgets.animated_tab_widget import AnimatedTabWidget
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
    finished = Signal(object, bool)

class GooglerUsageWorkerSignals(QObject):
    finished = Signal(object, bool)

class ElevenLabsBalanceWorkerSignals(QObject):
    finished = Signal(object, bool)

class VoicemakerBalanceWorkerSignals(QObject):
    finished = Signal(object, bool)

class GeminiTTSBalanceWorkerSignals(QObject):
    finished = Signal(object, bool)

class ValidationWorkerSignals(QObject):
    finished = Signal(bool, str) # is_valid, expires_at

class BalanceWorker(QRunnable):
    def __init__(self):
        super().__init__()
        self.signals = BalanceWorkerSignals()

    def run(self):
        api = OpenRouterAPI()
        balance = api.get_balance()
        self.signals.finished.emit(balance, balance is not None)

class GooglerUsageWorker(QRunnable):
    def __init__(self):
        super().__init__()
        self.signals = GooglerUsageWorkerSignals()

    def run(self):
        api = GooglerAPI()
        usage = api.get_usage()
        self.signals.finished.emit(usage, usage is not None)

class ElevenLabsBalanceWorker(QRunnable):
    def __init__(self):
        super().__init__()
        self.signals = ElevenLabsBalanceWorkerSignals()

    def run(self):
        api = ElevenLabsAPI()
        balance, status = api.get_balance()
        self.signals.finished.emit(balance, status == 'connected')

class VoicemakerBalanceWorker(QRunnable):
    def __init__(self):
        super().__init__()
        self.signals = VoicemakerBalanceWorkerSignals()

    def run(self):
        api = VoicemakerAPI()
        balance, status = api.get_balance()
        if balance is not None:
             self.signals.finished.emit(int(balance), status == 'connected')
        else:
             self.signals.finished.emit(None, False)

class GeminiTTSBalanceWorker(QRunnable):
    def __init__(self):
        super().__init__()
        self.signals = GeminiTTSBalanceWorkerSignals()

    def run(self):
        api = GeminiTTSAPI()
        balance, status = api.get_balance()
        if balance is not None:
            self.signals.finished.emit(float(balance), status == 'connected')
        else:
            self.signals.finished.emit(None, False)

class ValidationWorker(QRunnable):
    def __init__(self, api_key, server_url):
        super().__init__()
        self.signals = ValidationWorkerSignals()
        self.api_key = api_key
        self.server_url = server_url

    def run(self):
        is_valid = False
        expires_at = None
        if not self.api_key or not self.server_url:
            self.signals.finished.emit(is_valid, expires_at)
            return
        try:
            response = requests.post(
                f"{self.server_url}/validate_key/",
                json={"key": self.api_key},
                timeout=5
            )
            if response.status_code == 200:
                data = response.json()
                if data.get("valid"):
                    is_valid = True
                    expires_at = data.get("expires_at")
        except requests.RequestException:
            # Network error, etc.
            pass
        self.signals.finished.emit(is_valid, expires_at)

class MainWindow(QMainWindow):
    def __init__(self, app, subscription_info=None, api_key=None, server_url=None):
        super().__init__()
        self.app = app
        self.subscription_info = subscription_info
        self.api_key = api_key
        self.server_url = server_url
        self.settings_manager = settings_manager
        self.translator = translator
        self.queue_manager = QueueManager()
        self.task_processor = TaskProcessor(self.queue_manager)
        self.threadpool = QThreadPool()
        self.init_ui()
        logger.log(translator.translate('app_started'), level=LogLevel.INFO)
        self.app.installEventFilter(self)

    def check_api_key_validity(self):
        worker = ValidationWorker(api_key=self.api_key, server_url=self.server_url)
        worker.signals.finished.connect(self.on_validation_finished)
        self.threadpool.start(worker)

    def on_validation_finished(self, is_valid, expires_at):
        if not is_valid:
            self.validation_timer.stop()
            # Clear the saved key as it's no longer valid
            settings_manager.set('api_key', None)
            settings_manager.save_settings()
            
            title = self.translator.translate('subscription_expired_title')
            message = self.translator.translate('subscription_expired_message')
            QMessageBox.warning(self, title, message)
            
            # Visually indicate expiry
            self.days_left_label.setText("!")
            self.user_icon_button.setToolTip(self.translator.translate('subscription_expired_message'))
        else:
            # Update subscription info and UI
            self.subscription_info = expires_at
            self.update_subscription_status()

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
        self.update_title()
        self.setGeometry(100, 100, 1366, 768)

        self.central_widget = QWidget()
        self.setCentralWidget(self.central_widget)
        
        layout = QVBoxLayout(self.central_widget)

        self.tabs = AnimatedTabWidget()

        # --- Subscription Status Corner Widget ---
        corner_widget = QWidget()
        corner_layout = QHBoxLayout(corner_widget)
        corner_layout.setContentsMargins(0, 0, 10, 0) # Add some margin to the right
        corner_layout.setSpacing(10)

        self.days_left_label = QLabel()
        corner_layout.addWidget(self.days_left_label)

        self.user_icon_button = QPushButton()
        self.update_user_icon() # Set theme-aware icon
        
        icon_size = int(self.fontMetrics().height() * 1.5)
        self.user_icon_button.setIconSize(QSize(icon_size, icon_size))
        self.user_icon_button.setFlat(True)
        self.user_icon_button.setFixedSize(QSize(icon_size, icon_size))
        
        self.user_icon_button.clicked.connect(self.show_subscription_menu)
        corner_layout.addWidget(self.user_icon_button)

        self.tabs.setCornerWidget(corner_widget, Qt.Corner.TopRightCorner)
        # --- End of Corner Widget ---

        layout.addWidget(self.tabs)

        self.update_subscription_status()

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

        QTimer.singleShot(0, self.start_background_updates)
        
        # Apply last used template on startup
        last_template = self.settings_manager.get('last_used_template_name')
        template_applied = False
        if last_template:
            # ... (rest of the logic remains the same)
            from utils.settings import template_manager
            template_data = template_manager.load_template(last_template)
            if template_data:
                # Ignore subtitle settings from templates, as they are user/environment-specific
                template_data.pop('subtitles', None)

                for key, value in template_data.items():
                    if isinstance(value, dict) and key in self.settings_manager.settings and isinstance(self.settings_manager.settings[key], dict):
                        self.settings_manager.settings[key].update(value)
                    else:
                        self.settings_manager.settings[key] = value
                self.settings_manager.save_settings()
                
                self.settings_tab.templates_tab.populate_templates_combo()
                self.settings_tab.templates_tab.templates_combo.setCurrentText(last_template)
                self.text_tab.update_template_name(last_template)
                self.settings_tab._update_all_tabs()
                self.retranslate_ui()
                logger.log(f"Applied last used template: {last_template}", level=LogLevel.INFO)
                template_applied = True

        if not template_applied:
            self.settings_tab.languages_tab.load_elevenlabs_templates()

        # Setup periodic API key validation
        self.validation_timer = QTimer(self)
        self.validation_timer.timeout.connect(self.check_api_key_validity)
        self.validation_timer.start(10 * 60 * 1000) # 10 minutes

    def update_subscription_status(self):
        days_left_text = ""
        tooltip_text = "Немає інформації про підписку."

        if self.subscription_info:
            try:
                expires_at_str = self.subscription_info.split('.')[0]
                expires_at = datetime.fromisoformat(expires_at_str)
                
                days_left = (expires_at.date() - datetime.utcnow().date()).days

                if days_left >= 0:
                    days_left_text = f"{days_left} д"
                    tooltip_text = (
                        f"Підписка активна до: {expires_at.strftime('%Y-%m-%d')}\n"
                        f"Залишилось: {days_left} днів"
                    )
                else:
                    days_left_text = "!"
                    tooltip_text = "Термін дії вашої підписки закінчився."
            except (ValueError, TypeError):
                days_left_text = "?"
                tooltip_text = "Не вдалося визначити термін дії підписки."
        
        self.days_left_label.setText(days_left_text)
        self.user_icon_button.setToolTip(tooltip_text)

    def show_subscription_menu(self):
        menu = QMenu(self)
        
        if self.subscription_info:
            try:
                expires_at_str = self.subscription_info.split('.')[0]
                expires_at = datetime.fromisoformat(expires_at_str)
                expires_at_formatted = expires_at.strftime('%Y-%m-%d')
                
                days_left = (expires_at.date() - datetime.utcnow().date()).days

                if days_left >= 0:
                    menu.addAction(f"Підписка до: {expires_at_formatted}")
                    menu.addAction(f"Залишилось: {days_left} днів")
                else:
                    menu.addAction("Термін дії підписки закінчився")

            except (ValueError, TypeError):
                menu.addAction("Помилка формату дати")
        else:
            menu.addAction("Немає інформації про підписку")
            
        menu.exec(self.user_icon_button.mapToGlobal(self.user_icon_button.rect().bottomLeft()))

    def _on_image_review_required(self):
        # ... (rest of the file is the same)
        title = self.translator.translate('image_review_title')
        message = self.translator.translate('image_review_message')
        QMessageBox.information(self, title, message)
        self.tabs.setCurrentWidget(self.gallery_tab)
        self.gallery_tab.show_continue_button()

    def _on_translation_review_required(self, task_id, translated_text):
        state = self.task_processor.task_states[task_id]
        dialog = TranslationReviewDialog(self, state, translated_text, self.translator)

        def on_regenerate():
            self.task_processor.regenerate_translation(task_id)

        dialog.regenerate_requested.connect(on_regenerate)
        
        self.task_processor.translation_regenerated.connect(dialog.update_text)

        if dialog.exec():
            new_text = dialog.get_text()
            self.task_processor.task_states[task_id].text_for_processing = new_text
            if state.dir_path:
                with open(os.path.join(state.dir_path, "translation_reviewed.txt"), 'w', encoding='utf-8') as f:
                    f.write(new_text)
            self.task_processor._on_text_ready(task_id)
        else:
            self.task_processor._set_stage_status(task_id, 'stage_translation', 'error', 'User cancelled review.')
        
        try:
            self.task_processor.translation_regenerated.disconnect(dialog.update_text)
        except RuntimeError:
            pass

    def show_media_viewer(self, media_path):
        if media_path.lower().endswith(('.mp4', '.avi', '.mov', '.webm')):
            self.viewer = VideoViewer(media_path, parent=self.central_widget)
        else:
            self.viewer = ImageViewer(media_path, self.central_widget)
        
        self.viewer.setGeometry(self.central_widget.rect())
        self.viewer.show()
        self.viewer.setFocus()

    def show_processing_finished_dialog(self, elapsed_time):
        title = self.translator.translate('processing_complete_title')
        message = self.translator.translate('processing_complete_message').format(elapsed_time)
        QMessageBox.information(self, title, message)

    def update_title(self):
        app_name = self.translator.translate('app_title')
        self.setWindowTitle(f"{app_name} v{__version__}")

    def update_template_label(self):
        name = self.settings_manager.get('last_used_template_name')
        self.text_tab.update_template_name(name)

    def start_background_updates(self):
        self.update_balance()
        self.update_googler_usage()
        self.update_elevenlabs_balance()
        self.update_voicemaker_balance()
        self.update_gemini_tts_balance()

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

    def _on_balance_updated(self, balance, success):
        api_key = self.settings_manager.get("openrouter_api_key")
        if api_key:
            if success:
                balance_text = f"{self.translator.translate('balance_label')} {balance:.4f}$"
            else:
                balance_text = f"{self.translator.translate('balance_label')} {self.translator.translate('error_label')}"
        else:
            balance_text = ""

        self.text_tab.update_balance(balance_text)
        self.queue_tab.update_balance(balance_text)
        self.settings_tab.api_tab.openrouter_tab.update_balance_label(balance_text)

    def _on_googler_usage_updated(self, usage_data, success):
        googler_settings = self.settings_manager.get("googler", {})
        api_key = googler_settings.get("api_key")

        usage_text = ""
        display_text = "N/A"

        if api_key:
            if success and usage_data:
                img_usage = usage_data.get("current_usage", {}).get("hourly_usage", {}).get("image_generation", {})
                current_usage = img_usage.get("current_usage", "N/A")
                
                img_limits = usage_data.get("account_limits", {})
                limit = img_limits.get("img_gen_per_hour_limit", "N/A")

                usage_text = f"Googler: {current_usage} / {limit}"
                display_text = f"{current_usage} / {limit}"
            else:
                usage_text = f"Googler: {self.translator.translate('error_label')}"
                display_text = self.translator.translate('error_label')
        
        self.text_tab.update_googler_usage(usage_text)
        self.queue_tab.update_googler_usage(usage_text)
        self.settings_tab.api_tab.image_tab.googler_tab.usage_display_label.setText(display_text)

    def _on_elevenlabs_balance_updated(self, balance, success):
        api_key = self.settings_manager.get("elevenlabs_api_key")
        balance_text = ""
        balance_to_display_on_settings_tab = None

        if api_key:
            if success:
                balance_text = f"ElevenLabs: {balance}"
                balance_to_display_on_settings_tab = balance
            else:
                balance_text = f"ElevenLabs: {self.translator.translate('error_label')}"
                balance_to_display_on_settings_tab = self.translator.translate('error_label')

        self.text_tab.update_elevenlabs_balance(balance_text)
        self.queue_tab.update_elevenlabs_balance(balance_text)
        self.settings_tab.api_tab.audio_tab.elevenlabs_tab.update_balance_label(balance_to_display_on_settings_tab)

    def _on_voicemaker_balance_updated(self, balance, success):
        api_key = self.settings_manager.get("voicemaker_api_key")
        balance_text = ""
        balance_to_display_on_settings_tab = None

        if api_key:
            if success:
                balance_text = f"Voicemaker: {balance}"
                balance_to_display_on_settings_tab = balance
            else:
                balance_text = f"Voicemaker: {self.translator.translate('error_label')}"
                balance_to_display_on_settings_tab = self.translator.translate('error_label')

        self.text_tab.update_voicemaker_balance(balance_text)
        self.queue_tab.update_voicemaker_balance(balance_text)
        self.settings_tab.api_tab.audio_tab.voicemaker_tab.update_balance_label(balance_to_display_on_settings_tab)

    def _on_gemini_tts_balance_updated(self, balance, success):
        api_key = self.settings_manager.get("gemini_tts_api_key")
        balance_text = ""
        balance_to_display_on_settings_tab = None
        
        if api_key:
            if success:
                balance_text = f"GeminiTTS: {balance}"
                balance_to_display_on_settings_tab = balance
            else:
                balance_text = f"GeminiTTS: {self.translator.translate('error_label')}"
                balance_to_display_on_settings_tab = self.translator.translate('error_label')

        self.text_tab.update_gemini_tts_balance(balance_text)
        self.queue_tab.update_gemini_tts_balance(balance_text)
        self.settings_tab.api_tab.audio_tab.gemini_tts_tab.update_balance_label(balance_to_display_on_settings_tab)

    def update_user_icon(self):
        theme_name = self.settings_manager.get('theme', 'light')
        
        user_icon_base64_black = b"PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIGhlaWdodD0iMjRweCIgdmlld0JveD0iMCAwIDI0IDI0IiB3aWR0aD0iMjRweCIgZmlsbD0iIzAwMDAwMCI+PHBhdGggZD0iTTAgMGgyNHYyNEgwVjB6IiBmaWxsPSJub25lIi8+PHBhdGggZD0iTTEyIDEyYzIuMjEgMCA0LTEuNzkgNC00cy0xLjc5LTQtNC00LTQgMS43OS00IDQgMS43OSA0IDQgNHptMCAyYy0yLjY3IDAtOCAxLjM0LTggNHYyaDE2di0yYzAtMi42Ni01LjMzLTQtOC00eiIvPjwvc3ZnPg=="
        user_icon_base64_white = b"PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIGhlaWdodD0iMjRweCIgdmlld0JveD0iMCAwIDI0IDI0IiB3aWR0aD0iMjRweCIgZmlsbD0iI0ZGRkZGRiI+PHBhdGggZD0iTTAgMGgyNHYyNEgwVjB6IiBmaWxsPSJub25lIi8+PHBhdGggZD0iTTEyIDEyYzIuMjEgMCA0LTEuNzkgNC00cy0xLjc5LTQtNC00LTQgMS43OS00IDQgMS43OSA0IDQgNHptMCAyYy0yLjY3IDAtOCAxLjM0LTggNHYyaDE2di0yYzAtMi42Ni01LjMzLTQtOC00eiIvPjwvc3ZnPg=="

        if theme_name in ['dark', 'black']:
            icon_data = user_icon_base64_white
        else:
            icon_data = user_icon_base64_black
        
        pixmap = QPixmap()
        pixmap.loadFromData(QByteArray.fromBase64(icon_data), "svg")
        
        icon = QIcon(pixmap)
        self.user_icon_button.setIcon(icon)

    def apply_current_theme(self):
        theme_name = self.settings_manager.get('theme', 'light')
        accent_color = self.settings_manager.get('accent_color', '#3f51b5')

        extra = {
            'primaryColor': accent_color,
            'font_family': 'RobotoCondensed'
            }

        if theme_name == 'light':
            apply_stylesheet(self.app, theme='light_blue.xml', extra=extra)
        elif theme_name == 'dark':
            apply_stylesheet(self.app, theme='dark_teal.xml', extra=extra)
        elif theme_name == 'black':
            apply_stylesheet(self.app, theme='amoled_black.xml', extra=extra)
            custom_style = "QTextEdit { background-color: #121212; }"
            self.app.setStyleSheet(self.app.styleSheet() + custom_style)

        self.update_user_icon()

        if hasattr(self, 'settings_tab') and hasattr(self.settings_tab, 'statistics_tab'):
            self.settings_tab.statistics_tab.on_theme_changed()
            self.settings_tab.general_tab.update_style()

        if hasattr(self, 'text_tab'):
            self.text_tab.update_styles()

    def change_accent_color(self, color_hex):
        self.settings_manager.set('accent_color', color_hex)
        self.apply_current_theme()

    def change_theme(self, theme_name):
        self.settings_manager.set('theme', theme_name)
        self.apply_current_theme()

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
    # ... (rest of the file is the same)
    regenerate_requested = Signal()

    def __init__(self, parent, state, text, translator):
        super().__init__(parent)
        self.state = state
        self.translator = translator
        job_name = self.state.job_name
        lang_name = self.state.lang_name
        self.setWindowTitle(f"Перевірка перекладу: {job_name} ({lang_name})")
        self.setMinimumSize(700, 500)

        main_layout = QVBoxLayout(self)

        self.text_edit = QTextEdit()
        self.text_edit.setPlainText(text)
        main_layout.addWidget(self.text_edit)

        bottom_layout = QHBoxLayout()

        self.char_count_label = QLabel()
        bottom_layout.addWidget(self.char_count_label)
        bottom_layout.addStretch()

        self.button_box = QDialogButtonBox()
        self.regenerate_button = self.button_box.addButton("Перегенерувати", QDialogButtonBox.ButtonRole.ActionRole)
        self.ok_button = self.button_box.addButton(QDialogButtonBox.StandardButton.Ok)
        self.cancel_button = self.button_box.addButton(QDialogButtonBox.StandardButton.Cancel)
        bottom_layout.addWidget(self.button_box)
        
        main_layout.addLayout(bottom_layout)

        self.regenerate_button.clicked.connect(self.regenerate_requested.emit)
        self.ok_button.clicked.connect(self.accept)
        self.cancel_button.clicked.connect(self.reject)
        self.text_edit.textChanged.connect(self.update_char_count)

        self.update_char_count()

    def get_text(self):
        return self.text_edit.toPlainText()

    def update_char_count(self):
        original_len = len(self.state.original_text)
        translated_len = len(self.get_text())
        
        original_str = self.translator.translate('original_chars').format(count=original_len)
        translated_str = self.translator.translate('translated_chars').format(count=translated_len)
        
        self.char_count_label.setText(f"{original_str} | {translated_str}")

    def update_text(self, task_id, new_text):
        if self.state.task_id == task_id:
             self.text_edit.setPlainText(new_text)
