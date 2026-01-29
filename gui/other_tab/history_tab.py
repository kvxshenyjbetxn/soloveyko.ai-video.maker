from PySide6.QtWidgets import (QWidget, QVBoxLayout, QLabel, QHBoxLayout, 
                             QPushButton, QScrollArea, QFrame, QTextEdit, QDialog, QMessageBox)
from PySide6.QtCore import Qt
from datetime import datetime

from utils.translator import translator
from core.history_manager import history_manager
from utils.settings import settings_manager

class TextPreviewDialog(QDialog):
    def __init__(self, original, translated=None, parent=None):
        super().__init__(parent)
        self.setWindowTitle(translator.translate('history_text_preview_title', 'Перегляд тексту'))
        self.resize(700, 500)
        layout = QVBoxLayout(self)
        
        if translated:
            layout.addWidget(QLabel(f"<b>{translator.translate('original_text', 'Оригінальний текст')}:</b>"))
            orig_edit = QTextEdit()
            orig_edit.setPlainText(original)
            orig_edit.setReadOnly(True)
            layout.addWidget(orig_edit)
            
            layout.addWidget(QLabel(f"<b>{translator.translate('translated_text', 'Переклад')}:</b>"))
            trans_edit = QTextEdit()
            trans_edit.setPlainText(translated)
            trans_edit.setReadOnly(True)
            layout.addWidget(trans_edit)
        else:
            layout.addWidget(QLabel(f"<b>{translator.translate('text', 'Текст')}:</b>"))
            text_edit = QTextEdit()
            text_edit.setPlainText(original)
            text_edit.setReadOnly(True)
            layout.addWidget(text_edit)
            
        close_btn = QPushButton(translator.translate('close', 'Закрити'))
        close_btn.clicked.connect(self.accept)
        layout.addWidget(close_btn)

class HistoryCard(QFrame):
    def __init__(self, entry, parent=None):
        super().__init__(parent)
        self.entry = entry
        self.setFrameShape(QFrame.StyledPanel)
        self.setObjectName("historyCard")
        
        # Apply subtle styling for the card
        theme = settings_manager.get('theme', 'light')
        if theme == 'light':
            bg_color = "#ffffff"
            hover_color = "#f9f9f9"
            border_color = "#dddddd"
        elif theme == 'black':
            bg_color = "#000000"
            hover_color = "#0a0a0a"
            border_color = "#333333"
        else: # dark
            bg_color = "#2b2b2b"
            hover_color = "#323232"
            border_color = "#444444"
            
        self.setStyleSheet(f"""
            #historyCard {{ 
                border: 1px solid {border_color}; 
                border-radius: 8px; 
                background-color: {bg_color}; 
                padding: 10px; 
            }}
            #historyCard:hover {{
                border-color: #555555;
            }}
            QPushButton {{
                background-color: transparent;
                border: 1px solid rgba(255, 255, 255, 0.1);
                border-radius: 4px;
                padding: 4px 8px;
                font-size: 11px;
            }}
            QPushButton:hover {{
                background-color: rgba(255, 255, 255, 0.05);
            }}
        """)
        
        layout = QVBoxLayout(self)
        layout.setContentsMargins(15, 12, 15, 12)
        layout.setSpacing(10)
        
        # Header: Job Name and Time
        header_layout = QHBoxLayout()
        job_name = entry.get('job_name', 'Unknown')
        self.title_label = QLabel(f"<span style='font-size: 14px;'><b>{job_name}</b></span>")
        header_layout.addWidget(self.title_label)
        header_layout.addStretch()
        
        end_time_str = entry.get('end_time', '')
        if end_time_str:
            try:
                dt = datetime.fromisoformat(end_time_str)
                time_display = dt.strftime('%d-%m-%Y %H:%M:%S')
            except:
                time_display = end_time_str
        else:
            time_display = '?'
            
        self.time_label = QLabel(f"<span style='color: gray;'>{time_display}</span>")
        header_layout.addWidget(self.time_label)
        layout.addLayout(header_layout)
        
        # Languages section
        langs_container = QWidget()
        langs_layout = QVBoxLayout(langs_container)
        langs_layout.setContentsMargins(0, 0, 0, 0)
        langs_layout.setSpacing(5)
        
        languages = entry.get('languages', [])
        # If it's an old entry format, wrap the entry itself as a language
        if not languages and 'lang_id' in entry:
            languages = [entry]
            
        for lang in languages:
            # Create a clickable widget for each language row
            lang_widget = QFrame()
            lang_widget.setObjectName("langRow")
            lang_widget.setCursor(Qt.PointingHandCursor)
            lang_widget.setStyleSheet(f"""
                #langRow {{
                    background-color: transparent;
                    border: 1px solid rgba(255, 255, 255, 0.05);
                    border-radius: 6px;
                    padding: 5px;
                }}
                #langRow:hover {{
                    background-color: rgba(255, 255, 255, 0.08);
                    border-color: rgba(255, 255, 255, 0.2);
                }}
            """)
            
            lang_row = QHBoxLayout(lang_widget)
            lang_row.setContentsMargins(10, 2, 10, 2)
            
            lang_name = lang.get('lang_name', 'Unknown')
            template = lang.get('template', 'Default')
            
            # Lang and Template label
            lang_label = QLabel(f"<b>{lang_name}</b> <span style='color: gray; font-size: 11px;'>({template})</span>")
            lang_row.addWidget(lang_label)
            
            # Stages and Duration
            stages = lang.get('stages', [])
            translated_stage_names = []
            for s in stages:
                translated_name = translator.translate(s, s.replace('stage_', '').replace('_', ' ').capitalize())
                translated_stage_names.append(translated_name)
            
            stages_text = ", ".join(translated_stage_names)
            
            # Duration calculation
            start_iso = lang.get('start_time')
            end_iso = lang.get('end_time')
            duration_text = ""
            if start_iso and end_iso:
                try:
                    start_dt = datetime.fromisoformat(start_iso)
                    end_dt = datetime.fromisoformat(end_iso)
                    diff = end_dt - start_dt
                    minutes, seconds = divmod(int(diff.total_seconds()), 60)
                    if minutes > 0:
                        duration_text = f" [{minutes}m {seconds}s]"
                    else:
                        duration_text = f" [{seconds}s]"
                except:
                    pass

            info_label = QLabel(f"<span style='color: gray; font-size: 10px;'>| {stages_text} <b style='color: rgba(255,255,255,0.4);'>{duration_text}</b></span>")
            lang_row.addWidget(info_label)
            
            lang_row.addStretch()
            
            # Connect click event
            lang_widget.mousePressEvent = lambda event, l=lang: self.show_texts(l)
            
            langs_layout.addWidget(lang_widget)
            
        layout.addWidget(langs_container)

    def show_texts(self, lang_entry):
        original = lang_entry.get('original_text', '')
        translated = lang_entry.get('translated_text')
        dialog = TextPreviewDialog(original, translated, self)
        dialog.exec()

class HistoryTab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        self.main_window = main_window
        self.init_ui()

    def init_ui(self):
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(20, 20, 20, 20)
        main_layout.setSpacing(15)
        
        # Header with Clear Button
        header_layout = QHBoxLayout()
        self.title_label = QLabel(f"<h1>{translator.translate('history_tab_title', 'Історія')}</h1>")
        header_layout.addWidget(self.title_label)
        header_layout.addStretch()
        
        self.clear_btn = QPushButton(translator.translate('clear_history_button', 'Очистити історію'))
        self.clear_btn.setObjectName("clearHistoryBtn")
        self.clear_btn.clicked.connect(self.clear_history)
        header_layout.addWidget(self.clear_btn)
        
        main_layout.addLayout(header_layout)
        
        # Scroll Area for cards
        self.scroll = QScrollArea()
        self.scroll.setWidgetResizable(True)
        self.scroll.setFrameShape(QFrame.NoFrame)
        self.scroll_content = QWidget()
        self.scroll_layout = QVBoxLayout(self.scroll_content)
        self.scroll_layout.setAlignment(Qt.AlignTop)
        self.scroll_layout.setSpacing(10)
        self.scroll_layout.setContentsMargins(0, 0, 10, 0)
        self.scroll.setWidget(self.scroll_content)
        
        main_layout.addWidget(self.scroll)
        
        self.setLayout(main_layout)
        self.load_history()

    def load_history(self):
        # Clear existing
        while self.scroll_layout.count():
            child = self.scroll_layout.takeAt(0)
            if child.widget():
                child.widget().deleteLater()
            
        history = history_manager.get_history(30)
        
        if not history:
            no_data = QLabel(f"<div style='margin-top: 50px; color: gray;'>{translator.translate('no_history_yet', 'Історії поки немає')}</div>")
            no_data.setAlignment(Qt.AlignCenter)
            self.scroll_layout.addWidget(no_data)
            return

        for entry in history:
            card = HistoryCard(entry)
            self.scroll_layout.addWidget(card)

    def clear_history(self):
        reply = QMessageBox.question(self, 
                                   translator.translate('clear_history_title', 'Очистити історію?'),
                                   translator.translate('clear_history_confirm', 'Ви впевнені, що хочете видалити всю історію?'),
                                   QMessageBox.Yes | QMessageBox.No, QMessageBox.No)
        if reply == QMessageBox.Yes:
            history_manager.clear_history()
            self.load_history()

    def retranslate_ui(self):
        self.title_label.setText(f"<h1>{translator.translate('history_tab_title', 'Історія')}</h1>")
        self.clear_btn.setText(translator.translate('clear_history_button', 'Очистити історію'))
        # Retranslate items if needed, but easier to just reload
        self.load_history()

    def showEvent(self, event):
        super().showEvent(event)
        if not event.spontaneous():
            self.load_history()
