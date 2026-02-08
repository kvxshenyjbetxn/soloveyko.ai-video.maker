from PySide6.QtWidgets import QDialog, QVBoxLayout, QLabel, QHBoxLayout, QPushButton
from PySide6.QtCore import QTimer, Qt
from utils.translator import translator

class AutoShutdownDialog(QDialog):
    def __init__(self, action_name, timeout=10, parent=None):
        super().__init__(parent)
        self.action_name = action_name
        self.timeout = timeout
        self.setWindowTitle(translator.translate('auto_shutdown_title'))
        self.setModal(True)
        self.setWindowFlags(self.windowFlags() & ~Qt.WindowContextHelpButtonHint)
        self.setFixedSize(400, 180)
        
        self.init_ui()
        
        self.timer = QTimer(self)
        self.timer.timeout.connect(self.update_timer)
        self.timer.start(1000) # 1 sec

    def init_ui(self):
        layout = QVBoxLayout(self)
        layout.setSpacing(20)
        layout.setContentsMargins(20, 20, 20, 20)
        
        # Message
        self.lbl_message = QLabel()
        self.lbl_message.setAlignment(Qt.AlignCenter)
        self.lbl_message.setWordWrap(True)
        self.lbl_message.setStyleSheet("font-size: 14px; font-weight: bold;")
        layout.addWidget(self.lbl_message)
        
        # Buttons
        btn_layout = QHBoxLayout()
        
        self.btn_cancel = QPushButton(translator.translate('cancel_action_button'))
        self.btn_cancel.setMinimumHeight(40)
        self.btn_cancel.setCursor(Qt.PointingHandCursor)
        self.btn_cancel.clicked.connect(self.reject)
        btn_layout.addWidget(self.btn_cancel)
        
        self.btn_now = QPushButton(translator.translate('perform_now_button'))
        self.btn_now.setMinimumHeight(40)
        self.btn_now.setCursor(Qt.PointingHandCursor)
        # Red style for action
        self.btn_now.setStyleSheet("""
            QPushButton {
                background-color: #dc3545; 
                color: white; 
                font-weight: bold;
                border-radius: 4px;
            }
            QPushButton:hover {
                background-color: #c82333;
            }
        """)
        self.btn_now.clicked.connect(self.accept)
        btn_layout.addWidget(self.btn_now)
        
        layout.addLayout(btn_layout)
        
        self.update_message()

    def update_message(self):
        action_key = f"action_{self.action_name}"
        localized_action = translator.translate(action_key)
        
        text = translator.translate('auto_shutdown_info').format(
            action=localized_action,
            seconds=self.timeout
        )
        self.lbl_message.setText(text)

    def update_timer(self):
        self.timeout -= 1
        self.update_message()
        if self.timeout <= 0:
            self.timer.stop()
            self.accept()
