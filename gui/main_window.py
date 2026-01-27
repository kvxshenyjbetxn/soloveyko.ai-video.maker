import os
import sys
import requests
import collections
import platform
from datetime import datetime
from PySide6.QtWidgets import QMainWindow, QTabWidget, QWidget, QComboBox, QAbstractSpinBox, QAbstractScrollArea, QSlider, QVBoxLayout, QMessageBox, QDialog, QTextEdit, QPushButton, QDialogButtonBox, QLabel, QHBoxLayout, QMenu, QInputDialog
from PySide6.QtCore import QCoreApplication, QEvent, QObject, Signal, QRunnable, QThreadPool, Qt, QSize, QByteArray, QTimer, Slot
from PySide6.QtGui import QWheelEvent, QIcon, QAction, QPixmap
from gui.widgets.animated_tab_widget import AnimatedTabWidget
from gui.dialogs.prompt_settings_dialog import PromptSettingsDialog
from gui.dialogs.welcome_dialog import WelcomeDialog
from gui.qt_material import apply_stylesheet
from gui.api_workers import ApiKeyCheckWorker, ApiKeyCheckSignals, ElevenLabsUnlimBalanceWorker, VersionCheckWorker, VersionCheckSignals
from gui.widgets.help_label import HelpLabel

# Windows COM handling for threads
try:
    import pythoncom
except ImportError:
    pythoncom = None


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
from gui.rewrite_tab import RewriteTab
from gui.gallery_tab.image_viewer import ImageViewer
from gui.gallery_tab.video_viewer import VideoViewer
from gui.other_tab.other_tab import OtherTab
from core.queue_manager import QueueManager
from core.task_processor import TaskProcessor
from utils.logger import logger, LogLevel
from utils.hint_manager import hint_manager

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

class BalanceWorker(QRunnable):
    def __init__(self):
        super().__init__()
        self.signals = BalanceWorkerSignals()

    def run(self):
        if pythoncom:
             pythoncom.CoInitialize()
        try:
            api = OpenRouterAPI()
            balance = api.get_balance()
            self.signals.finished.emit(balance, balance is not None)
        except Exception as e:
            logger.log(f"BalanceWorker error: {e}", level=LogLevel.ERROR)
            self.signals.finished.emit(None, False)
        finally:
            if pythoncom:
                pythoncom.CoUninitialize()

class GooglerUsageWorker(QRunnable):
    def __init__(self):
        super().__init__()
        self.signals = GooglerUsageWorkerSignals()

    def run(self):
        if pythoncom:
             pythoncom.CoInitialize()
        try:
            api = GooglerAPI()
            usage = api.get_usage()
            self.signals.finished.emit(usage, usage is not None)
        except Exception as e:
            logger.log(f"GooglerUsageWorker error: {e}", level=LogLevel.ERROR)
            self.signals.finished.emit(None, False)
        finally:
            if pythoncom:
                pythoncom.CoUninitialize()

class ElevenLabsBalanceWorker(QRunnable):
    def __init__(self):
        super().__init__()
        self.signals = ElevenLabsBalanceWorkerSignals()

    def run(self):
        if pythoncom:
             pythoncom.CoInitialize()
        try:
            api = ElevenLabsAPI()
            balance, status = api.get_balance()
            self.signals.finished.emit(balance, status == 'connected')
        except Exception as e:
            logger.log(f"ElevenLabsBalanceWorker error: {e}", level=LogLevel.ERROR)
            self.signals.finished.emit(None, False)
        finally:
            if pythoncom:
                pythoncom.CoUninitialize()

class VoicemakerBalanceWorker(QRunnable):
    def __init__(self):
        super().__init__()
        self.signals = VoicemakerBalanceWorkerSignals()

    def run(self):
        if pythoncom:
             pythoncom.CoInitialize()
        try:
            api = VoicemakerAPI()
            balance, status = api.get_balance()
            if balance is not None:
                 self.signals.finished.emit(int(balance), status == 'connected')
            else:
                 self.signals.finished.emit(None, False)
        except Exception as e:
            logger.log(f"VoicemakerBalanceWorker error: {e}", level=LogLevel.ERROR)
            self.signals.finished.emit(None, False)
        finally:
            if pythoncom:
                pythoncom.CoUninitialize()

class GeminiTTSBalanceWorker(QRunnable):
    def __init__(self):
        super().__init__()
        self.signals = GeminiTTSBalanceWorkerSignals()

    def run(self):
        if pythoncom:
             pythoncom.CoInitialize()
        try:
            api = GeminiTTSAPI()
            balance, status = api.get_balance()
            if balance is not None:
                self.signals.finished.emit(float(balance), status == 'connected')
            else:
                self.signals.finished.emit(None, False)
        except Exception as e:
            logger.log(f"GeminiTTSBalanceWorker error: {e}", level=LogLevel.ERROR)
            self.signals.finished.emit(None, False)
        finally:
            if pythoncom:
                pythoncom.CoUninitialize()



class MainWindow(QMainWindow):
    SHOW_REWRITE_TAB = False # Default to False, enabled only if level >= 2
    
    def __init__(self, app, subscription_info=None, api_key=None, server_url=None):
        super().__init__()
        self.app = app
        self.subscription_info = subscription_info
        self.api_key = api_key
        self.server_url = server_url
        self.subscription_level = 1 # Default level
        self.settings_manager = settings_manager
        self.translator = translator
        
        # self._apply_startup_template()  # Disabled to prevent overwriting global settings on startup
        self.settings_manager.set("last_applied_template", None) # Reset to ensure we start in "Global Settings" mode UI-wise

        self.queue_manager = QueueManager()
        self.task_processor = TaskProcessor(self.queue_manager)
        self.threadpool = QThreadPool()
        self.text_review_queue = collections.deque()
        self.is_review_dialog_active = False
        self.active_workers = set() # Track workers to prevent garbage collection and Segfaults
        
        # Periodic Googler usage updates during processing
        self._googler_timer = QTimer(self)
        self._googler_timer.timeout.connect(self.update_googler_usage)
        
        self.init_ui()
        logger.log('Application started.', level=LogLevel.INFO)
        self.app.installEventFilter(self)
        
        # Start version check
        self.check_app_version()

    def _apply_startup_template(self):
        template_name = self.settings_manager.get("last_applied_template")
        if not template_name:
            return

        from utils.settings import template_manager
        logger.log(f"Applying startup template: {template_name}", level=LogLevel.INFO)
        template_data = template_manager.load_template(template_name)
        if not template_data:
            logger.log(f"Startup template '{template_name}' not found.", level=LogLevel.WARNING)
            return

        def deep_merge(source, destination):
            for key, value in source.items():
                if isinstance(value, dict) and key in destination and isinstance(destination[key], dict):
                    deep_merge(value, destination[key])
                else:
                    destination[key] = value
            return destination

        # Perform a deep merge and save
        deep_merge(template_data, self.settings_manager.settings)
        self.settings_manager.save_settings()

    def check_api_key_validity(self):
        worker = ApiKeyCheckWorker(self.api_key, self.server_url)
        worker.signals.finished.connect(self.on_api_key_checked)
        self.threadpool.start(worker)

    def check_app_version(self):
        worker = VersionCheckWorker(self.server_url)
        worker.signals.finished.connect(self.on_version_checked)
        self.threadpool.start(worker)

    def on_version_checked(self, success, remote_version):
        if not success:
            logger.log(f"Version check failed: {remote_version}", level=LogLevel.WARNING)
            # Ensure welcome dialog still shows even if version check fails
            QTimer.singleShot(500, self.show_welcome_dialog)
            return

        try:
            # Simple semver comparison
            current_v = __version__.strip()
            remote_v = remote_version.strip()
            
            logger.log(f"Version Check: Current={current_v}, Remote={remote_v}", level=LogLevel.INFO)

            def parse_version(v_str):
                return [int(x) for x in v_str.split('.') if x.isdigit()]

            if parse_version(remote_v) > parse_version(current_v):
                title = self.translator.translate("update_available_title")
                message = self.translator.translate("update_available_message").format(remote_version=remote_version)
                
                # Using QMessageBox with custom text
                msg_box = QMessageBox(self)
                msg_box.setWindowTitle(title)
                msg_box.setTextFormat(Qt.RichText) # Enable HTML/Links
                msg_box.setText(message)
                msg_box.setIcon(QMessageBox.Information)
                msg_box.setStandardButtons(QMessageBox.Ok)
                msg_box.exec()

        except Exception as e:
            logger.log(f"Error comparing versions: {e}", level=LogLevel.ERROR)
        finally:
             QTimer.singleShot(500, self.show_welcome_dialog)

    def show_welcome_dialog(self):
        if not self.settings_manager.get('show_welcome_dialog', True):
            return

        try:
            dialog = WelcomeDialog(self)
            dialog.exec()
        except Exception as e:
            logger.log(f"Error showing welcome dialog: {e}", level=LogLevel.ERROR)

    def on_api_key_checked(self, is_valid, expires_at, subscription_level, telegram_id):
        self.subscription_level = subscription_level
        if is_valid:
            # Auto-populate Telegram ID if not set
            if telegram_id:
                current_tid = self.settings_manager.get("telegram_user_id")
                if not current_tid:
                    self.settings_manager.set("telegram_user_id", str(telegram_id))
                    # Check if NotificationTab is initialized (it might be inside SettingsTab -> SettingsTab (widget) -> uses internal structure)
                    # Easier: if settings_tab exists, updating settings_manager is enough, 
                    # but if we want strictly immediate visual update without reopening tab:
                    if hasattr(self.settings_tab, 'notification_tab'):
                         # Assuming settings_tab has notification_tab as a direct attribute if it was refactored, 
                         # but checking the file structure earlier showed it's likely a sub-tab.
                         # Actually settings_tab.py was not fully read, but notification_tab.py had update_fields.
                         pass 
                         
            # Update subscription info logic
            days_left = 0
            if expires_at:
                try:
                     # Parse ISO format
                    dt = datetime.fromisoformat(expires_at.replace('Z', '+00:00'))
                    now = datetime.now(dt.tzinfo) # Use tzinfo from dt for comparison
                    if dt > now:
                        days_left = (dt - now).days
                        
                        # --- Level 2 (Unlimited) Check ---
                        if subscription_level >= 2:
                            self.days_left_label.setText(self.translator.translate('subscription_unlimited'))
                            self.days_left_label.setStyleSheet("color: gold; font-weight: bold;")
                        else:
                             # Basic Plan logic
                            if days_left > 3650: # Fallback check
                                days_left_str = self.translator.translate('subscription_unlimited')
                                self.days_left_label.setText(days_left_str)
                            else:
                                days_left_str = str(days_left)
                                prefix = self.translator.translate('subscription_days_left')
                                self.days_left_label.setText(f"{prefix}{days_left_str}")
                                
                            self.days_left_label.setStyleSheet("") # Reset style
                except Exception as e:
                    logger.log(f"Error parsing date: {e}", level=LogLevel.ERROR)
            
            # --- Update Rewrite Tab Visibility ---
            # Level 2 = Plus (With Rewrite)
            should_show_rewrite = (subscription_level >= 2)
            
            if hasattr(self.settings_tab, 'languages_tab'):
                self.settings_tab.languages_tab.set_rewrite_visible(should_show_rewrite)

            if should_show_rewrite != self.SHOW_REWRITE_TAB:
                self.SHOW_REWRITE_TAB = should_show_rewrite
                self._rebuild_text_tabs()
            
            # Refresh balances for all valid users to ensure UI is up to date
            self.start_background_updates()
        else:
            # Invalid key logic...
            # Clear the saved key as it's no longer valid
            settings_manager.set('api_key', None)
            settings_manager.save_settings()
            
            title = self.translator.translate('subscription_expired_title')
            message = self.translator.translate('subscription_expired_message')
            QMessageBox.warning(self, title, message)
            
            # Visually indicate expiry
            self.days_left_label.setText("!")
            self.user_icon_button.setToolTip(self.translator.translate('subscription_expired_message'))
            self.SHOW_REWRITE_TAB = False # Ensure rewrite tab is hidden if key is invalid
            self._rebuild_text_tabs()

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
        if getattr(sys, 'frozen', False):
            base_path = sys._MEIPASS
        else:
            base_path = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
        
        icon_path = os.path.join(base_path, "assets", "icon.ico")
        self.setWindowIcon(QIcon(icon_path))

        self.update_title()
        self.setGeometry(100, 100, 1600, 900)

        self.central_widget = QWidget()
        self.setCentralWidget(self.central_widget)
        
        layout = QVBoxLayout(self.central_widget)

        self.tabs = AnimatedTabWidget()

        # --- Subscription Status Corner Widget ---
        corner_widget = QWidget()
        corner_layout = QHBoxLayout(corner_widget)
        corner_layout.setContentsMargins(0, 0, 10, 0) # Add some margin to the right
        corner_layout.setSpacing(10)

        self.help_icon = HelpLabel("info_icon_hint")
        self.help_legend_label = QLabel()
        self.active_template_label = QLabel()
        
        corner_layout.addWidget(self.help_icon)
        corner_layout.addWidget(self.help_legend_label)
        corner_layout.addSpacing(15)
        corner_layout.addWidget(self.active_template_label)

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
        self.rewrite_tab = RewriteTab(main_window=self)
        self.settings_tab = SettingsTab(main_window=self)
        self.other_tab = OtherTab(main_window=self)
        self.log_tab = LogTab()
        self.queue_tab = QueueTab(parent=self.tabs, main_window=self, log_tab=self.log_tab)
        self.gallery_tab = GalleryTab()
        
        logger.log_message_signal.connect(self.log_tab.add_log_message)
        self.review_notification_shown = False

        # Custom container for Text+Rewrite if both available
        self.text_group_tab = QWidget()
        self.text_group_layout = QVBoxLayout(self.text_group_tab)
        self.text_group_layout.setContentsMargins(0, 10, 0, 0)
        self.text_group_tabs = AnimatedTabWidget()
        self.text_group_layout.addWidget(self.text_group_tabs)
        
        # Initial building of Text/Rewrite tabs
        self._rebuild_text_tabs()

        self.tabs.addTab(self.queue_tab, self.translator.translate('queue_tab'))
        self.tabs.addTab(self.gallery_tab, self.translator.translate('gallery_tab_title'))
        self.tabs.addTab(self.settings_tab, self.translator.translate('settings_tab'))
        self.tabs.addTab(self.other_tab, self.translator.translate('other_tab_title', 'Інше'))
        self.tabs.addTab(self.log_tab, self.translator.translate('log_tab'))

        # Hide Queue and Gallery initially if empty
        self._update_tab_visibility()

        # Connect signals for dynamic visibility
        self.queue_manager.queue_updated.connect(self._update_tab_visibility)
        self.task_processor.image_generated.connect(self._update_tab_visibility)
        self.gallery_tab.image_deleted.connect(lambda: QTimer.singleShot(100, self._update_tab_visibility)) # Slight delay to let gallery update its state
        self.gallery_tab.image_regenerated.connect(self._update_tab_visibility)

        # Connect signals
        self.queue_manager.task_added.connect(self.queue_tab.add_task)
        self.queue_tab.start_processing_button.clicked.connect(self._start_processing_checked)
        self.task_processor.processing_started.connect(lambda: self._update_queue_tab_state('processing'))
        self.task_processor.processing_finished.connect(self._handle_processing_finished)
        self.task_processor.processing_finished.connect(self.show_processing_finished_dialog)
        self.task_processor.processing_finished.connect(self._googler_timer.stop)
        self.task_processor.processing_finished.connect(self.update_balance)
        self.task_processor.processing_finished.connect(self.update_googler_usage)
        self.task_processor.processing_finished.connect(self.update_elevenlabs_balance)
        self.task_processor.processing_finished.connect(self.update_elevenlabs_unlim_balance)
        self.task_processor.processing_finished.connect(self.update_voicemaker_balance)
        self.task_processor.processing_finished.connect(self.update_gemini_tts_balance)
        self.task_processor.stage_status_changed.connect(self._on_stage_status_changed)
        self.task_processor.stage_status_changed.connect(self.queue_tab.update_stage_status)
        self.task_processor.stage_metadata_updated.connect(self.queue_tab.update_stage_metadata)
        self.task_processor.image_generated.connect(self.gallery_tab.add_media)
        self.task_processor.video_generated.connect(self.gallery_tab.update_thumbnail)
        self.task_processor.task_progress_log.connect(self.queue_tab.on_task_progress_log)
        self.task_processor.balance_updated.connect(self._on_direct_balance_update)
        self.task_processor.image_review_required.connect(self._on_image_review_required)
        self.task_processor.translation_review_required.connect(self._on_translation_review_required)
        self.task_processor.rewrite_review_required.connect(self._on_rewrite_review_required)
        self.gallery_tab.continue_montage_requested.connect(self.task_processor.resume_all_montages)
        self.gallery_tab.continue_montage_requested.connect(lambda: self.tabs.setCurrentIndex(self.tabs.indexOf(self.queue_tab))) # Switch to Queue tab reliably
        self.gallery_tab.image_deleted.connect(self.task_processor._on_image_deleted)
        self.gallery_tab.image_regenerated.connect(self.task_processor._on_image_regenerated)
        self.gallery_tab.media_clicked.connect(self.show_media_viewer)
        self.settings_tab.templates_tab.template_applied.connect(self.on_template_applied)
        self.tabs.currentChanged.connect(self.on_tab_changed)

        QTimer.singleShot(100, self.check_api_key_validity) # Initial check
        QTimer.singleShot(200, lambda: self.settings_tab.languages_tab.load_elevenlabs_templates()) # Load templates in background to avoid blocking startup
        
        self.update_active_template_display()

    def on_tab_changed(self, index):
        self.refresh_quick_settings_panels()

    def on_template_applied(self):
        QMessageBox.information(self, translator.translate('template_applied_title', "Template Applied"), translator.translate('template_applied_message', "Template settings have been applied and saved."))
        self.refresh_ui_from_settings()

    def refresh_ui_from_settings(self):
        logger.log("Refreshing UI from new settings...", level=LogLevel.INFO)
        hint_manager.load_hints()
        
        # Update all settings tabs
        if hasattr(self.settings_tab, '_update_all_tabs'):
            self.settings_tab._update_all_tabs()

        # Update other parts of the UI
        self.text_tab.retranslate_ui()
        self.apply_current_theme()
        
        # Update Quick Settings Panels
        self.refresh_quick_settings_panels()
        
        # Update the template label in the corner
        self.update_active_template_display()

    def refresh_quick_settings_panels(self):
        """Refreshes the QuickSettingsPanel in TextTab and RewriteTab."""
        if hasattr(self, 'text_tab') and hasattr(self.text_tab, 'quick_settings_panel'):
            self.text_tab.quick_settings_panel.refresh()
        
        if hasattr(self, 'rewrite_tab') and hasattr(self.rewrite_tab, 'quick_settings_panel'):
            self.rewrite_tab.quick_settings_panel.refresh()

    def on_quick_setting_changed(self, key):
        """Called when a setting is changed from the QuickSettingsPanel."""
        # Update Settings Tab UI without full app visual refresh if possible,
        # but for things like Theme or Language we definitely need full refresh.
        
        # Always trigger settings tab update so values sync
        if hasattr(self.settings_tab, '_update_all_tabs'):
            self.settings_tab._update_all_tabs()
            
        settings_needing_full_refresh = ['language', 'theme', 'accent_color']
        if key in settings_needing_full_refresh:
            self.refresh_ui_from_settings()
        else:
             # Just update global UI elements that depend on settings if any (like tooltips?)
             # Most real-time things are read from settings_manager directly when used.
             pass

    def _start_processing_checked(self):
        """Reset error tracking and validate API key before starting processing."""
        self._has_processing_errors = False
        worker = ApiKeyCheckWorker(api_key=self.api_key, server_url=self.server_url)
        self.active_workers.add(worker)
        # Fix: the signal emits 3 arguments (bool, str, int)
        worker.signals.finished.connect(
            lambda is_valid, expires_at, sub_level, tid: (
                self.on_pre_processing_validation_finished(is_valid),
                self.active_workers.discard(worker)
            )
        )
        self.threadpool.start(worker)

    def on_pre_processing_validation_finished(self, is_valid):
        if is_valid:
            if not self.task_processor.is_finished and self.task_processor.task_states:
                 logger.log("Processing is already active. Checking for new tasks...", level=LogLevel.INFO)
            self.task_processor.start_processing()
            self._googler_timer.start(30000) # Update Googler usage every 30s during execution
        else:
            # The periodic validation will have already shown a dialog.
            # We show a specific one here for the action of starting processing.
            title = self.translator.translate('subscription_expired_title')
            message = self.translator.translate('subscription_expired_message_start_processing')
            QMessageBox.warning(self, title, message)

    def _on_stage_status_changed(self, job_id, lang_id, stage_key, status):
        """Monitor errors during processing to update the Queue tab indicator."""
        if status in ('failed', 'error'):
            # Switch to red immediately if error occurs
            self._update_queue_tab_state('error')
            self._has_processing_errors = True

    def _handle_processing_finished(self):
        """Decide whether to show green or red indicator when all tasks are done."""
        if getattr(self, '_has_processing_errors', False):
            self._update_queue_tab_state('error')
        else:
            self._update_queue_tab_state('finished')

    def update_subscription_status(self):
        days_left_text = ""
        tooltip_text = self.translator.translate('no_subscription_info')

        if hasattr(self, 'subscription_level') and self.subscription_level >= 2:
            self.days_left_label.setText(self.translator.translate('subscription_unlimited'))
            self.days_left_label.setStyleSheet("color: gold; font-weight: bold;")
            self.user_icon_button.setToolTip(self.translator.translate('subscription_unlimited'))
            return

        if self.subscription_info:
            try:
                # Handle ISO format string, potentially with 'Z' at the end
                expires_at_str = self.subscription_info.replace('Z', '+00:00')
                expires_at = datetime.fromisoformat(expires_at_str)
                
                # Make sure the datetime is timezone-aware for correct comparison
                now = datetime.now(expires_at.tzinfo)
                
                days_left = (expires_at - now).days

                if days_left >= 0:
                    days_left_text = f"{days_left} д"
                    tooltip_text = (
                        f"{self.translator.translate('subscription_active_until')}: {expires_at.strftime('%Y-%m-%d %H:%M')}\n"
                        f"{self.translator.translate('days_left')}: {days_left}"
                    )
                else:
                    days_left_text = "!"
                    tooltip_text = self.translator.translate('subscription_expired')
            except (ValueError, TypeError) as e:
                logger.log(f"Error parsing subscription date: {e}", level=LogLevel.ERROR)
                days_left_text = "?"
                tooltip_text = self.translator.translate('subscription_date_format_error')
        
        self.days_left_label.setText(days_left_text)
        self.user_icon_button.setToolTip(tooltip_text)

    def show_subscription_menu(self):
        menu = QMenu(self)
        
        # Logout Action
        logout_action = QAction(self.translator.translate('logout'), self)
        logout_action.triggered.connect(self.logout)
        
        if self.subscription_info:
            try:
                expires_at_str = self.subscription_info.replace('Z', '+00:00')
                expires_at = datetime.fromisoformat(expires_at_str)
                expires_at_formatted = expires_at.strftime('%Y-%m-%d %H:%M')
                
                now = datetime.now(expires_at.tzinfo)
                days_left = (expires_at - now).days

                # Hide expiration info for Level 2 (Unlimited)
                if hasattr(self, 'subscription_level') and self.subscription_level >= 2:
                    menu.addAction(self.translator.translate('subscription_unlimited'))
                elif days_left >= 0:
                    menu.addAction(f"{self.translator.translate('subscription_active_until')}: {expires_at_formatted}")
                    menu.addAction(f"{self.translator.translate('days_left')}: {days_left}")
                else:
                    menu.addAction(self.translator.translate('subscription_expired'))

            except (ValueError, TypeError):
                menu.addAction(self.translator.translate('subscription_date_format_error'))
        else:
            menu.addAction(self.translator.translate('no_subscription_info'))
            
        menu.addSeparator()
        menu.addAction(logout_action)
            
        menu.exec(self.user_icon_button.mapToGlobal(self.user_icon_button.rect().bottomLeft()))

    def logout(self):
        # Clear saved API key
        self.settings_manager.set('api_key', None)
        self.settings_manager.save_settings()
        
        # Inform the user and close the application
        QMessageBox.information(self, self.translator.translate('logout_success_title'), self.translator.translate('logout_success_message_manual_restart', "You have successfully logged out. Please manually restart the application to sign in with a different account."))
        
        # Close the application
        QCoreApplication.quit()

    def _on_image_review_required(self):
        # ... (rest of the file is the same)
        title = self.translator.translate('image_review_title')
        message = self.translator.translate('image_review_message')
        QMessageBox.information(self, title, message)
        self.tabs.setCurrentWidget(self.gallery_tab)
        self.gallery_tab.show_continue_button()

    def _on_translation_review_required(self, task_id, translated_text):
        self.text_review_queue.append((task_id, translated_text, 'stage_translation'))
        if not self.is_review_dialog_active:
            QTimer.singleShot(0, self._show_next_review_dialog)

    def _on_rewrite_review_required(self, task_id, rewritten_text):
        self.text_review_queue.append((task_id, rewritten_text, 'stage_rewrite'))
        if not self.is_review_dialog_active:
            QTimer.singleShot(0, self._show_next_review_dialog)

    def _show_next_review_dialog(self):
        if self.is_review_dialog_active or not self.text_review_queue:
            return

        self.is_review_dialog_active = True
        task_id, text, stage = self.text_review_queue.popleft()
        
        state = self.task_processor.task_states[task_id]
        dialog = TextReviewDialog(self, state, text, self.translator, stage)

        # Use open() instead of exec() to avoid blocking the main loop and causing 0x8001010d errors
        # Connect signals for result handling
        dialog.finished.connect(lambda result: self._on_review_dialog_finished(result, dialog, task_id, state, stage))
        
        def on_regenerate(extra_options=None):
            if stage == 'stage_translation':
                self.task_processor.regenerate_translation(task_id, extra_options)
            else:
                self.task_processor.regenerate_rewrite(task_id, extra_options)

        dialog.regenerate_requested.connect(on_regenerate)
        
        if stage == 'stage_translation':
            self.task_processor.translation_regenerated.connect(dialog.update_text)
        else:
            self.task_processor.rewrite_regenerated.connect(dialog.update_text)

        dialog.open() 

    def _on_review_dialog_finished(self, result, dialog, task_id, state, stage):
        try:
            # If closed due to regeneration request, we do nothing.
            # The regeneration was triggered in the background, and when it finishes,
            # a new dialog will appear because we reset the 'shown' flag in task_processor.
            if getattr(dialog, 'is_regenerating', False):
                return

            if result == QDialog.DialogCode.Accepted:
                new_text = dialog.get_text()
                self.task_processor.task_states[task_id].text_for_processing = new_text
                if state.dir_path:
                    try:
                        # We only update translation.txt which is the working copy.
                        # Origin is saved as translation_orig.txt in task_processor.py
                        with open(os.path.join(state.dir_path, "translation.txt"), 'w', encoding='utf-8') as f:
                            f.write(new_text)
                    except Exception as e:
                        logger.log(f"Failed to save reviewed text: {e}", level=LogLevel.ERROR)
                
                # Update status to success to turn green
                self.task_processor._set_stage_status(task_id, stage, 'success')
                self.task_processor._on_text_ready(task_id)
            else:
                self.task_processor._set_stage_status(task_id, stage, 'error', 'User cancelled review.')
            
            # Clean up connections
            try:
                if stage == 'stage_translation':
                    self.task_processor.translation_regenerated.disconnect(dialog.update_text)
                else:
                    self.task_processor.rewrite_regenerated.disconnect(dialog.update_text)
            except (RuntimeError, TypeError):
                pass
                
        finally:
            self.is_review_dialog_active = False
            # Process the next item in the queue asynchronously
            QTimer.singleShot(0, self._show_next_review_dialog)

    def show_media_viewer(self, media_path):
        all_media_data = self.gallery_tab.get_all_media_data()
        all_paths = [item['path'] for item in all_media_data]
        
        try:
            current_index = all_paths.index(media_path)
        except ValueError:
            current_index = 0
            # Fallback if path not found in current tab data
            all_media_data = [{
                'path': media_path,
                'prompt': '',
                'is_video': media_path.lower().endswith(('.mp4', '.avi', '.mov', '.webm'))
            }]

        if media_path.lower().endswith(('.mp4', '.avi', '.mov', '.webm')):
            self.viewer = VideoViewer(all_media_data, current_index, parent=self.central_widget)
        else:
            self.viewer = ImageViewer(all_media_data, current_index, self.central_widget)
        
        self.viewer.delete_requested.connect(self.gallery_tab.delete_media_by_path)
        self.viewer.regenerate_requested.connect(self.gallery_tab._on_regenerate_requested)
        
        self.viewer.setGeometry(self.central_widget.rect())
        self.viewer.show()
        self.viewer.setFocus()

    def show_processing_finished_dialog(self, elapsed_time):
        title = self.translator.translate('processing_complete_title')
        message = self.translator.translate('processing_complete_message').format(elapsed_time)
        QMessageBox.information(self, title, message)

    def update_title(self):
        app_name = "Soloveyko.AI-Video.Maker"
        self.setWindowTitle(f"{app_name} v{__version__}")

    def update_active_template_display(self):
        self.help_icon.update_tooltip()
        self.help_legend_label.setText(translator.translate('help_legend'))
        
        template_name = self.settings_manager.get("last_applied_template")
        if template_name:
            self.active_template_label.setText(f"{translator.translate('active_template_label', 'Template')}: {template_name}")
            self.active_template_label.setToolTip(translator.translate('active_template_tooltip', 'This template is applied to the global settings.'))
        else:
            self.active_template_label.setText(translator.translate('global_settings_label', 'Global Settings'))
            self.active_template_label.setToolTip(translator.translate('global_settings_tooltip', 'Using standard global settings.'))

    def start_background_updates(self):
        self.update_balance()
        self.update_googler_usage()
        self.update_elevenlabs_balance()
        self.update_elevenlabs_unlim_balance()
        self.update_voicemaker_balance()
        self.update_gemini_tts_balance()

    def update_balance(self, *args):
        worker = BalanceWorker()
        self.active_workers.add(worker)
        worker.signals.finished.connect(lambda b, s: (self._on_balance_updated(b, s), self.active_workers.discard(worker)))
        self.threadpool.start(worker)

    def update_googler_usage(self, *args):
        worker = GooglerUsageWorker()
        self.active_workers.add(worker)
        worker.signals.finished.connect(lambda u, s: (self._on_googler_usage_updated(u, s), self.active_workers.discard(worker)))
        self.threadpool.start(worker)

    def update_elevenlabs_balance(self, *args):
        worker = ElevenLabsBalanceWorker()
        self.active_workers.add(worker)
        worker.signals.finished.connect(lambda b, s: (self._on_elevenlabs_balance_updated(b, s), self.active_workers.discard(worker)))
        self.threadpool.start(worker)

    def update_elevenlabs_unlim_balance(self, *args):
        api_key = self.settings_manager.get("elevenlabs_unlim_api_key")
        if api_key:
            worker = ElevenLabsUnlimBalanceWorker(api_key)
            self.active_workers.add(worker)
            worker.signals.finished.connect(lambda b, s: (self._on_elevenlabs_unlim_balance_updated(b, s), self.active_workers.discard(worker)))
            self.threadpool.start(worker)
    def update_voicemaker_balance(self, *args):
        worker = VoicemakerBalanceWorker()
        self.active_workers.add(worker)
        worker.signals.finished.connect(lambda b, s: (self._on_voicemaker_balance_updated(b, s), self.active_workers.discard(worker)))
        self.threadpool.start(worker)

    def update_gemini_tts_balance(self, *args):
        worker = GeminiTTSBalanceWorker()
        self.active_workers.add(worker)
        worker.signals.finished.connect(lambda b, s: (self._on_gemini_tts_balance_updated(b, s), self.active_workers.discard(worker)))
        self.threadpool.start(worker)



    def _on_balance_updated(self, balance, success):
        api_key = self.settings_manager.get("openrouter_api_key")
        if api_key:
            if success:
                balance_text = f"{self.translator.translate('openrouter_balance_label')} {balance:.4f}$"
            else:
                balance_text = f"{self.translator.translate('openrouter_balance_label')} {self.translator.translate('error_label')}"
        else:
            balance_text = ""

        self.text_tab.update_balance(balance_text)
        if hasattr(self, 'rewrite_tab'):
            self.rewrite_tab.update_balance(balance_text)
        self.queue_tab.update_balance(balance_text)
        self.settings_tab.api_tab.openrouter_tab.update_balance_label(balance_text)

    def _on_googler_usage_updated(self, usage_data, success):
        googler_settings = self.settings_manager.get("googler", {})
        api_key = googler_settings.get("api_key")

        usage_text = ""
        display_text = "N/A"

        if api_key:
            if success and usage_data:
                try:
                    # Safely get data even if keys are missing or null
                    img_limits = usage_data.get("account_limits") or {}
                    img_limit = img_limits.get("img_gen_per_hour_limit", 0)
                    
                    cur_usage = usage_data.get("current_usage") or {}
                    hourly = cur_usage.get("hourly_usage") or {}
                    
                    img_stats = hourly.get("image_generation") or {}
                    img_cur = img_stats.get("current_usage", 0)
                    
                    vid_limit = img_limits.get("video_gen_per_hour_limit", 0)
                    vid_stats = hourly.get("video_generation") or {}
                    vid_cur = vid_stats.get("current_usage", 0)

                    usage_text = f"Googler: Img {img_cur}/{img_limit} | Vid {vid_cur}/{vid_limit}"
                    display_text = f"Img: {img_cur}/{img_limit} | Vid: {vid_cur}/{vid_limit}"
                    
                    # Update tooltips with detailed info
                    if hasattr(self.text_tab, 'update_googler_usage_detailed'):
                        self.text_tab.update_googler_usage_detailed(usage_text, usage_data)
                    if hasattr(self, 'rewrite_tab') and hasattr(self.rewrite_tab, 'update_googler_usage_detailed'):
                        self.rewrite_tab.update_googler_usage_detailed(usage_text, usage_data)
                    if hasattr(self, 'queue_tab') and hasattr(self.queue_tab, 'update_googler_usage_detailed'):
                        self.queue_tab.update_googler_usage_detailed(usage_text, usage_data)
                        
                except Exception as e:
                    logger.log(f"Error processing Googler usage data: {e}", level=LogLevel.ERROR)
                    usage_text = "Googler: Parse Error"
                    display_text = "Parse Error"
            else:
                usage_text = f"Googler: {self.translator.translate('error_label')}"
                display_text = self.translator.translate('error_label')
        
        self.text_tab.update_googler_usage(usage_text)
        if hasattr(self, 'rewrite_tab'):
            self.rewrite_tab.update_googler_usage(usage_text)
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
        if hasattr(self, 'rewrite_tab'):
            self.rewrite_tab.update_elevenlabs_balance(balance_text)
        self.queue_tab.update_elevenlabs_balance(balance_text)
        self.settings_tab.api_tab.audio_tab.elevenlabs_tab.update_balance_label(balance_to_display_on_settings_tab)

    def _on_elevenlabs_unlim_balance_updated(self, balance, success):
        api_key = self.settings_manager.get("elevenlabs_unlim_api_key")
        balance_text = ""
        balance_to_display_on_settings_tab = None

        if api_key:
            if success:
                balance_text = f"ElevenLabsUnlim: {balance}"
                balance_to_display_on_settings_tab = balance
            else:
                balance_text = f"ElevenLabsUnlim: {self.translator.translate('error_label')}"
                balance_to_display_on_settings_tab = self.translator.translate('error_label')

        self.text_tab.update_elevenlabs_unlim_balance(balance_text)
        if hasattr(self, 'rewrite_tab'):
            if hasattr(self.rewrite_tab, 'update_elevenlabs_unlim_balance'):
                self.rewrite_tab.update_elevenlabs_unlim_balance(balance_text)
        self.queue_tab.update_elevenlabs_unlim_balance(balance_text)
        # Check if tab exists before updating
        if hasattr(self.settings_tab.api_tab.audio_tab, 'elevenlabs_unlim_tab'):
             self.settings_tab.api_tab.audio_tab.elevenlabs_unlim_tab.update_balance_label(balance_to_display_on_settings_tab)

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
        if hasattr(self, 'rewrite_tab'):
            self.rewrite_tab.update_voicemaker_balance(balance_text)
        self.queue_tab.update_voicemaker_balance(balance_text)
    
    @Slot(str, object)
    def _on_direct_balance_update(self, provider, data):
        """Handle direct balance updates from workers."""
        if provider == 'voicemaker':
            # Data is the balance value itself
            self._on_voicemaker_balance_updated(data, True)
        elif provider == 'openrouter':
            # Data is None, trigger refresh
            self.update_balance()
        elif provider == 'googler':
            # Periodic updates are handled by _googler_timer during processing
            pass
        elif provider == 'elevenlabs':
            self.update_elevenlabs_balance()
        elif provider == 'elevenlabs_unlim':
            self.update_elevenlabs_unlim_balance()
        elif provider == 'gemini_tts':
            self.update_gemini_tts_balance()

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
        if hasattr(self, 'rewrite_tab'):
            self.rewrite_tab.update_gemini_tts_balance(balance_text)
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
        hint_manager.load_hints()
        self.update_title()
        
        # Text/Translation Tab
        if self.SHOW_REWRITE_TAB:
            # Translation is inside a group
            idx = self.tabs.indexOf(self.text_group_tab)
            if idx != -1:
                self.tabs.setTabText(idx, self.translator.translate('text_group_title', 'Текст'))
            self.text_group_tabs.setTabText(0, self.translator.translate('text_processing_tab'))
            self.text_group_tabs.setTabText(1, self.translator.translate('rewrite_tab'))
        else:
            # Translation is top-level
            idx = self.tabs.indexOf(self.text_tab)
            if idx != -1:
                self.tabs.setTabText(idx, self.translator.translate('text_processing_tab'))

        self.tabs.setTabText(self.tabs.indexOf(self.queue_tab), self.translator.translate('queue_tab'))
        self.tabs.setTabText(self.tabs.indexOf(self.gallery_tab), self.translator.translate('gallery_tab_title'))
        self.tabs.setTabText(self.tabs.indexOf(self.settings_tab), self.translator.translate('settings_tab'))
        self.tabs.setTabText(self.tabs.indexOf(self.other_tab), self.translator.translate('other_tab_title', 'Інше'))
        self.tabs.setTabText(self.tabs.indexOf(self.log_tab), self.translator.translate('log_tab'))
        
        self.text_tab.retranslate_ui()
        self.rewrite_tab.retranslate_ui()
        self.settings_tab.retranslate_ui()
        self.other_tab.retranslate_ui()
        self.queue_tab.retranslate_ui()
        self.gallery_tab.retranslate_ui()

    def _rebuild_text_tabs(self):
        """Rebuilds the Text/Rewrite tab structure based on self.SHOW_REWRITE_TAB."""
        # Save current active tab if it's one of the text tabs
        current_widget = self.tabs.currentWidget()
        
        # Remove both from everywhere first
        idx_main_text = self.tabs.indexOf(self.text_tab)
        if idx_main_text != -1: self.tabs.removeTab(idx_main_text)
        
        idx_main_group = self.tabs.indexOf(self.text_group_tab)
        if idx_main_group != -1: self.tabs.removeTab(idx_main_group)
        
        while self.text_group_tabs.count():
            self.text_group_tabs.removeTab(0)
            
        if self.SHOW_REWRITE_TAB:
            # Group them
            self.text_group_tabs.addTab(self.text_tab, self.translator.translate('text_processing_tab'))
            self.text_group_tabs.addTab(self.rewrite_tab, self.translator.translate('rewrite_tab'))
            self.tabs.insertTab(0, self.text_group_tab, self.translator.translate('text_group_title', 'Текст'))
            if current_widget in [self.text_tab, self.rewrite_tab, self.text_group_tab]:
                self.tabs.setCurrentWidget(self.text_group_tab)
        else:
            # Just Translation
            self.tabs.insertTab(0, self.text_tab, self.translator.translate('text_processing_tab'))
            if current_widget in [self.text_tab, self.rewrite_tab, self.text_group_tab]:
                self.tabs.setCurrentWidget(self.text_tab)

    def _update_tab_visibility(self):
        """Shows or hides Queue and Gallery tabs based on content."""
        # Target order: Text/Translation (0), Queue (1), Gallery (2), Settings (3), Other (4), Log (5)
        
        # 1. Queue Tab
        queue_count = self.queue_manager.get_task_count()
        queue_idx = self.tabs.indexOf(self.queue_tab)
        current_idx = self.tabs.currentIndex()
        
        if queue_count > 0:
            if queue_idx == -1:
                # Insert at position 1 (after Text tabs)
                title = self.translator.translate('queue_tab')
                self.tabs.insertTab(min(1, self.tabs.count()), self.queue_tab, title)
                # Reset state for freshly added tab
                self._update_queue_tab_state('default')
        else:
            if queue_idx != -1:
                # If Queue tab was active, redirect to Text
                was_active = (current_idx == queue_idx)
                # Reset text to original before removing (just in case)
                self.tabs.setTabText(queue_idx, self.translator.translate('queue_tab'))
                self.tabs.removeTab(queue_idx)
                self._update_queue_tab_state('default')
                if was_active:
                    self.activate_tab(self.text_tab)

        # 2. Gallery Tab
        # Custom logic: hide gallery if queue is empty as per request
        gallery_has_items = len(self.gallery_tab.task_groups) > 0 and queue_count > 0
        gallery_idx = self.tabs.indexOf(self.gallery_tab)
        if gallery_has_items:
            if gallery_idx == -1:
                title = self.translator.translate('gallery_tab_title')
                # Insert after Queue (if exists) or Text
                pos = 1
                if self.tabs.indexOf(self.queue_tab) != -1: pos = 2
                self.tabs.insertTab(min(pos, self.tabs.count()), self.gallery_tab, title)
        else:
            if gallery_idx != -1:
                # If Gallery tab was active and is being removed, redirect to Text
                was_active = (self.tabs.currentIndex() == gallery_idx)
                self.tabs.removeTab(gallery_idx)
                if was_active:
                    self.activate_tab(self.text_tab)
        
        # Ensure Settings, Other, Log are at the end if they moved
        for tab in [self.settings_tab, self.other_tab, self.log_tab]:
            idx = self.tabs.indexOf(tab)
            if idx != -1:
                pass

    def _update_queue_tab_state(self, state):
        """Updates the Queue tab text color and button color based on processing state."""
        idx = self.tabs.indexOf(self.queue_tab)
        if idx == -1:
            return

        base_title = self.translator.translate('queue_tab')
        
        # Define colors for robust visibility
        if state == 'processing':
            # Yellow
            color_hex = "#FFD700" 
            hover_color = "#FFEA00"
            btn_text_color = "black"
        elif state == 'finished':
            # Green (Success)
            color_hex = "#28a745"
            hover_color = "#218838"
            btn_text_color = "white"
        elif state == 'error':
            # Red (Failure)
            color_hex = "#dc3545"
            hover_color = "#c82333"
            btn_text_color = "white"
        else:
            # Default/Pending (Gray)
            color_hex = "#808080"
            hover_color = None
            btn_text_color = "white"

        # 1. Update Tab Bar with Icon instead of HTML (to avoid raw text error)
        self.tabs.setTabText(idx, base_title)
        
        # Create a small circular icon
        from PySide6.QtGui import QPixmap, QPainter, QColor, QBrush, QIcon
        from PySide6.QtCore import Qt, QSize
        
        # Using a narrower pixmap to bring the dot closer to the text
        pixmap = QPixmap(14, 16)
        pixmap.fill(Qt.transparent)
        painter = QPainter(pixmap)
        painter.setRenderHint(QPainter.Antialiasing)
        painter.setBrush(QBrush(QColor(color_hex)))
        painter.setPen(Qt.NoPen)
        # Draw 12x12 circle touching the right edge (x=2, width=12 -> right edge at 14)
        painter.drawEllipse(2, 2, 12, 12)
        painter.end()
        
        self.tabs.setIconSize(QSize(14, 16))
        self.tabs.setTabIcon(idx, QIcon(pixmap))
        
        if state != 'default':
            self.tabs.tabBar().setTabTextColor(idx, QColor(color_hex))
        else:
            self.tabs.tabBar().setTabTextColor(idx, QColor())

        # 2. Update Button in Queue Tab
        if hasattr(self.queue_tab, 'start_processing_button'):
            if state != 'default':
                self.queue_tab.start_processing_button.setStyleSheet(f"""
                    QPushButton {{
                        background-color: {color_hex}; 
                        color: {btn_text_color}; 
                        font-weight: bold; 
                        padding: 5px 15px;
                        border-radius: 4px;
                        border: none;
                    }}
                    QPushButton:hover {{
                        background-color: {hover_color};
                    }}
                """)
            else:
                # Revert to original green style if default
                self.queue_tab.start_processing_button.setStyleSheet("""
                    QPushButton {
                        background-color: #28a745; 
                        color: white; 
                        font-weight: bold; 
                        padding: 5px 15px;
                        border-radius: 4px;
                        border: none;
                    }
                    QPushButton:hover {
                        background-color: #218838;
                    }
                """)

    def activate_tab(self, widget):
        """Switches to the specified tab, even if it's nested (e.g., Translation/Rewrite)."""
        # Check if it's in the main tab widget
        idx = self.tabs.indexOf(widget)
        if idx != -1:
            self.tabs.setCurrentIndex(idx)
            return

        # Check if it's in the text group
        nested_idx = self.text_group_tabs.indexOf(widget)
        if nested_idx != -1:
            group_idx = self.tabs.indexOf(self.text_group_tab)
            if group_idx != -1:
                self.tabs.setCurrentIndex(group_idx)
                self.text_group_tabs.setCurrentIndex(nested_idx)
                return

    def closeEvent(self, event):
        # Save all settings globally
        self.settings_manager.save_settings()
        
        # Trigger save on sub-tabs if they have specific logic not covered by the global manager
        if hasattr(self.settings_tab.api_tab.image_tab.pollinations_tab, 'save_settings'):
            self.settings_tab.api_tab.image_tab.pollinations_tab.save_settings()
        if hasattr(self.settings_tab.api_tab.image_tab.googler_tab, 'save_settings'):
            self.settings_tab.api_tab.image_tab.googler_tab.save_settings()
        
        # Cleanup task processor resources to prevent 0x8001010d error
        if hasattr(self, 'task_processor'):
            self.task_processor.cleanup()
            
        logger.log('Application closing.', level=LogLevel.INFO)
        super().closeEvent(event)

class TextReviewDialog(QDialog):
    regenerate_requested = Signal(object) # Can optionally carry a custom prompt (str) or None

    def __init__(self, parent, state, text, translator, stage='stage_translation'):
        super().__init__(parent)
        self.state = state
        self.translator = translator
        self.stage = stage
        self.is_regenerating = False # Flag to track if regeneration was requested

        job_name = self.state.job_name
        lang_name = self.state.lang_name
        
        title_key = 'translation_review_label' if stage == 'stage_translation' else 'rewrite_review_label'
        title = self.translator.translate(title_key).replace(':', '').strip()
        self.setWindowTitle(f"{title}: {job_name} ({lang_name})")
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
        self.edit_prompt_button = self.button_box.addButton(self.translator.translate("edit_prompt_button"), QDialogButtonBox.ButtonRole.ActionRole)
        self.regenerate_button = self.button_box.addButton(self.translator.translate("thumbnail_regen_button"), QDialogButtonBox.ButtonRole.ActionRole)
        self.ok_button = self.button_box.addButton(QDialogButtonBox.StandardButton.Ok)
        self.cancel_button = self.button_box.addButton(QDialogButtonBox.StandardButton.Cancel)
        bottom_layout.addWidget(self.button_box)
        
        main_layout.addLayout(bottom_layout)

        self.edit_prompt_button.clicked.connect(self.on_edit_prompt_clicked)
        self.regenerate_button.clicked.connect(self.on_regenerate_clicked)
        self.ok_button.clicked.connect(self.accept)
        self.cancel_button.clicked.connect(self.reject)
        self.text_edit.textChanged.connect(self.update_char_count)

        self.update_char_count()

    def on_regenerate_clicked(self):
        self.is_regenerating = True
        self.regenerate_requested.emit(None)
        self.close() # Close immediately to allow other tasks to proceed

    def on_edit_prompt_clicked(self):
        # Get current settings from state
        current_prompt = ""
        current_model = None
        current_temp = None
        current_tokens = None
        
        lang_config = self.state.lang_data
        
        if self.stage == 'stage_translation':
            current_prompt = lang_config.get('prompt', '')
            current_model = lang_config.get('model')
            current_temp = lang_config.get('temperature')
            current_tokens = lang_config.get('max_tokens')
        else:
            current_prompt = lang_config.get('rewrite_prompt') or 'Rewrite this text:'
            current_model = lang_config.get('rewrite_model')
            current_temp = lang_config.get('rewrite_temperature')
            current_tokens = lang_config.get('rewrite_max_tokens')

        # Fallback to global settings if not found in language config
        if not current_model:
            current_model = self.state.settings.get('model')
        
        available_models = self.state.settings.get('openrouter_models', [])

        dialog = PromptSettingsDialog(
            self, 
            current_prompt, 
            current_model, 
            current_temp, 
            current_tokens, 
            available_models
        )
        
        if dialog.exec() == QDialog.DialogCode.Accepted:
            data = dialog.get_data()
            self.is_regenerating = True
            self.regenerate_requested.emit(data)
            self.close()

    def get_text(self):
        return self.text_edit.toPlainText()

    def update_char_count(self):
        original_len = len(self.state.original_text)
        translated_len = len(self.get_text())
        
        original_str = self.translator.translate('original_chars').format(count=original_len)
        
        res_key = 'translated_chars' if self.stage == 'stage_translation' else 'characters_count'
        if res_key == 'characters_count':
            translated_str = f"{self.translator.translate('stage_rewrite')}: {translated_len} {self.translator.translate('characters_count')}"
        else:
            translated_str = self.translator.translate(res_key).format(count=translated_len)
        
        self.char_count_label.setText(f"{original_str} | {translated_str}")

    def update_text(self, task_id, new_text):
        if self.state.task_id == task_id:
             self.text_edit.setPlainText(new_text)
