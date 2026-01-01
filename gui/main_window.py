import os
import sys
import requests
import collections
import platform
from datetime import datetime
from PySide6.QtWidgets import QMainWindow, QTabWidget, QWidget, QComboBox, QAbstractSpinBox, QAbstractScrollArea, QSlider, QVBoxLayout, QMessageBox, QDialog, QTextEdit, QPushButton, QDialogButtonBox, QLabel, QHBoxLayout, QMenu, QInputDialog
from PySide6.QtCore import QCoreApplication, QEvent, QObject, Signal, QRunnable, QThreadPool, Qt, QSize, QByteArray, QTimer
from PySide6.QtGui import QWheelEvent, QIcon, QAction, QPixmap
from gui.widgets.animated_tab_widget import AnimatedTabWidget
from gui.dialogs.prompt_settings_dialog import PromptSettingsDialog
from gui.qt_material import apply_stylesheet
from gui.api_workers import ApiKeyCheckWorker, ApiKeyCheckSignals

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
        self.init_ui()
        logger.log('Application started.', level=LogLevel.INFO)
        self.app.installEventFilter(self)

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
                if self.SHOW_REWRITE_TAB:
                    # Add tab
                    if not hasattr(self, 'rewrite_tab'):
                         self.rewrite_tab = RewriteTab(main_window=self)
                    
                    # Insert after Text Translation tab (index 1)
                    self.tabs.insertTab(1, self.rewrite_tab, self.translator.translate('rewrite_tab'))
                    
                else:
                    # Remove tab
                    # Check if rewrite_tab exists and is in tabs
                    if hasattr(self, 'rewrite_tab'):
                        idx = self.tabs.indexOf(self.rewrite_tab)
                        if idx != -1:
                            self.tabs.removeTab(idx)
            
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
            if hasattr(self, 'rewrite_tab'):
                idx = self.tabs.indexOf(self.rewrite_tab)
                if idx != -1:
                    self.tabs.removeTab(idx)

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

        self.active_template_label = QLabel()
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
        if self.SHOW_REWRITE_TAB:
            self.rewrite_tab = RewriteTab(main_window=self)

        self.settings_tab = SettingsTab(main_window=self)
        self.log_tab = LogTab()
        self.queue_tab = QueueTab(parent=self.tabs, main_window=self, log_tab=self.log_tab)
        self.gallery_tab = GalleryTab()
        logger.log_message_signal.connect(self.log_tab.add_log_message)
        self.review_notification_shown = False

        self.tabs.addTab(self.text_tab, self.translator.translate('text_processing_tab'))
        if self.SHOW_REWRITE_TAB:
            self.tabs.addTab(self.rewrite_tab, self.translator.translate('rewrite_tab'))
        self.tabs.addTab(self.queue_tab, self.translator.translate('queue_tab'))
        self.tabs.addTab(self.gallery_tab, self.translator.translate('gallery_tab_title'))
        self.tabs.addTab(self.settings_tab, self.translator.translate('settings_tab'))
        self.tabs.addTab(self.log_tab, self.translator.translate('log_tab'))

        # Connect signals
        self.queue_manager.task_added.connect(self.queue_tab.add_task)
        self.queue_tab.start_processing_button.clicked.connect(self._start_processing_checked)
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
        self.task_processor.rewrite_review_required.connect(self._on_rewrite_review_required)
        self.gallery_tab.continue_montage_requested.connect(self.task_processor.resume_all_montages)
        self.gallery_tab.image_deleted.connect(self.task_processor._on_image_deleted)
        self.gallery_tab.media_clicked.connect(self.show_media_viewer)
        self.settings_tab.templates_tab.template_applied.connect(self.on_template_applied)

        QTimer.singleShot(100, self.check_api_key_validity) # Initial check
        QTimer.singleShot(200, lambda: self.settings_tab.languages_tab.load_elevenlabs_templates()) # Load templates in background to avoid blocking startup
        
        self.update_active_template_display()

    def on_template_applied(self):
        QMessageBox.information(self, translator.translate('template_applied_title', "Template Applied"), translator.translate('template_applied_message', "Template settings have been applied and saved."))
        self.refresh_ui_from_settings()

    def refresh_ui_from_settings(self):
        logger.log("Refreshing UI from new settings...", level=LogLevel.INFO)
        
        # Update all settings tabs
        if hasattr(self.settings_tab, '_update_all_tabs'):
            self.settings_tab._update_all_tabs()

        # Update other parts of the UI
        self.text_tab.retranslate_ui()
        self.apply_current_theme()
        
        # Update the template label in the corner
        self.update_active_template_display()

    def _start_processing_checked(self):
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
        else:
            # The periodic validation will have already shown a dialog.
            # We show a specific one here for the action of starting processing.
            title = self.translator.translate('subscription_expired_title')
            message = self.translator.translate('subscription_expired_message_start_processing')
            QMessageBox.warning(self, title, message)

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
        app_name = "Soloveyko.AI-Video.Maker"
        self.setWindowTitle(f"{app_name} v{__version__}")

    def update_active_template_display(self):
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
                balance_text = f"{self.translator.translate('balance_label')} {balance:.4f}$"
            else:
                balance_text = f"{self.translator.translate('balance_label')} {self.translator.translate('error_label')}"
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
        self.update_title()
        self.tabs.setTabText(self.tabs.indexOf(self.text_tab), self.translator.translate('text_processing_tab'))
        if self.SHOW_REWRITE_TAB:
            self.tabs.setTabText(self.tabs.indexOf(self.rewrite_tab), self.translator.translate('rewrite_tab'))
            self.rewrite_tab.retranslate_ui()
        self.tabs.setTabText(self.tabs.indexOf(self.queue_tab), self.translator.translate('queue_tab'))
        self.tabs.setTabText(self.tabs.indexOf(self.gallery_tab), self.translator.translate('gallery_tab_title'))
        self.tabs.setTabText(self.tabs.indexOf(self.settings_tab), self.translator.translate('settings_tab'))
        self.tabs.setTabText(self.tabs.indexOf(self.log_tab), self.translator.translate('log_tab'))
        
        self.text_tab.retranslate_ui()
        self.settings_tab.retranslate_ui()
        self.queue_tab.retranslate_ui()
        self.gallery_tab.retranslate_ui()

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
