from PySide6.QtWidgets import (QWidget, QVBoxLayout, QLabel, QScrollArea, 
                               QFrame, QPushButton, QHBoxLayout, QStyle, QSizePolicy)
from PySide6.QtCore import Qt, Signal, QSize
from PySide6.QtGui import QColor, QIcon, QFont
from core.history_manager import history_manager
from utils.translator import translator
from datetime import datetime
import json

class RecentTaskCard(QFrame):
    clicked = Signal(dict)

    def __init__(self, job, parent=None):
        super().__init__(parent)
        self.job = job
        self.init_ui()

    def init_ui(self):
        self.setFrameShape(QFrame.Shape.StyledPanel)
        self.setObjectName("RecentTaskCard")
        
        layout = QVBoxLayout(self)
        layout.setContentsMargins(10, 10, 10, 10)
        layout.setSpacing(5)

        # Header: Name and Type Badge
        header_layout = QHBoxLayout()
        
        name_label = QLabel(self.job.get('name', 'Untitled'))
        name_label.setStyleSheet("font-weight: bold; font-size: 13px; color: #ffffff;")
        name_label.setWordWrap(True)
        header_layout.addWidget(name_label, 1)
        
        job_type = self.job.get('type', 'text')
        type_label = QLabel(translator.translate(f"type_{job_type}", job_type.capitalize()))
        type_label.setStyleSheet(f"""
            background-color: {'#4CAF50' if job_type == 'text' else '#2196F3'};
            color: white;
            border-radius: 4px;
            padding: 2px 6px;
            font-size: 10px;
            font-weight: bold;
        """)
        type_label.setFixedSize(type_label.sizeHint())
        header_layout.addWidget(type_label)
        
        layout.addLayout(header_layout)

        # Content: Languages and Templates
        langs = self.job.get('languages', {})
        lang_text = ", ".join(langs.keys())
        
        info_label = QLabel(f"{translator.translate('languages_label', 'Languages')}: {lang_text}")
        info_label.setStyleSheet("color: #bbbbbb; font-size: 11px;")
        layout.addWidget(info_label)

        # Time
        try:
            created_at = datetime.fromisoformat(self.job.get('created_at', ''))
            time_str = created_at.strftime('%H:%M, %d.%m')
        except:
            time_str = "Recently"
            
        time_label = QLabel(time_str)
        time_label.setStyleSheet("color: #888888; font-size: 10px;")
        time_label.setAlignment(Qt.AlignmentFlag.AlignRight)
        layout.addWidget(time_label)

        # Styling
        self.setStyleSheet("""
            QFrame#RecentTaskCard {
                background-color: rgba(255, 255, 255, 0.05);
                border: 1px solid rgba(255, 255, 255, 0.1);
                border-radius: 12px;
            }
            QFrame#RecentTaskCard:hover {
                background-color: rgba(255, 255, 255, 0.1);
                border: 1px solid rgba(255, 255, 255, 0.2);
            }
        """)
        
        self.setCursor(Qt.CursorShape.PointingHandCursor)

    def mousePressEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton:
            self.clicked.emit(self.job)
        super().mousePressEvent(event)

class RecentTasksPanel(QWidget):
    task_selected = Signal(dict)

    def __init__(self, main_window=None, parent=None):
        super().__init__(parent)
        self.main_window = main_window
        self.init_ui()
        self.refresh()

    def init_ui(self):
        self.layout = QVBoxLayout(self)
        self.layout.setContentsMargins(0, 5, 0, 0)
        self.layout.setSpacing(0)
        
        # Header
        header_frame = QFrame()
        header_layout = QHBoxLayout(header_frame)
        header_layout.setContentsMargins(10, 5, 10, 5)
        
        title_label = QLabel(translator.translate('recent_tasks_title', "Recent Tasks"))
        title_label.setStyleSheet("font-weight: bold; color: gray;")
        header_layout.addWidget(title_label)
        
        header_layout.addStretch()
        self.layout.addWidget(header_frame)

        # Scroll Area
        self.scroll_area = QScrollArea()
        self.scroll_area.setWidgetResizable(True)
        self.scroll_area.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        self.scroll_area.setFrameShape(QFrame.Shape.NoFrame)
        
        self.container_widget = QWidget()
        self.container_layout = QVBoxLayout(self.container_widget)
        self.container_layout.setContentsMargins(10, 5, 10, 5)
        self.container_layout.setSpacing(10)
        self.container_layout.addStretch()
        
        self.scroll_area.setWidget(self.container_widget)
        self.layout.addWidget(self.scroll_area)
        
        # Default appearance
        self.setMinimumWidth(250)

        # Footer
        footer_frame = QFrame()
        footer_layout = QVBoxLayout(footer_frame)
        footer_layout.setContentsMargins(10, 10, 10, 10)
        footer_layout.setSpacing(5)
        
        # Main Footer Text
        footer_text = translator.translate('recent_tasks_footer', "Click on a task to quickly fill data for re-run.")
        self.footer_label = QLabel(footer_text)
        self.footer_label.setWordWrap(True)
        self.footer_label.setStyleSheet("color: gray; font-size: 10px; font-style: italic;")
        self.footer_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        footer_layout.addWidget(self.footer_label)

        # Close Hint Text
        close_hint_text = translator.translate('recent_tasks_close_hint', "To close, drag the splitter to the left.")
        self.close_hint_label = QLabel(close_hint_text)
        self.close_hint_label.setWordWrap(True)
        self.close_hint_label.setStyleSheet("color: #666; font-size: 9px;")
        self.close_hint_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        footer_layout.addWidget(self.close_hint_label)

        self.layout.addWidget(footer_frame)

    def refresh(self):
        # Clear existing
        while self.container_layout.count() > 1: # Keep the stretch
            item = self.container_layout.takeAt(0)
            if item.widget():
                item.widget().deleteLater()
        
        recent_jobs = history_manager.get_recent_jobs(days=2)
        
        if not recent_jobs:
            no_tasks_label = QLabel(translator.translate('no_recent_tasks', "No tasks in last 2 days"))
            no_tasks_label.setStyleSheet("color: gray; font-style: italic;")
            no_tasks_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
            self.container_layout.insertWidget(0, no_tasks_label)
        else:
            for job in recent_jobs:
                card = RecentTaskCard(job)
                card.clicked.connect(self.task_selected.emit)
                self.container_layout.insertWidget(self.container_layout.count() - 1, card)

    def set_active_job_filter(self, job_type):
        """Optionally filter recent jobs by type (text/rewrite)"""
        # ... logic to filter ...
        pass
