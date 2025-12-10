from PySide6.QtWidgets import QWidget, QVBoxLayout, QComboBox, QTableWidget, QTableWidgetItem, QLabel, QHBoxLayout, QHeaderView
from PySide6.QtCore import Qt

from utils.translator import translator
from core.statistics_manager import statistics_manager

class StatisticsTab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        self.main_window = main_window
        self.period_map = {
            'all_time': translator.translate('all_time'),
            'current_month': translator.translate('current_month'),
            'current_year': translator.translate('current_year')
        }
        self.event_map = {
            'translation': 'stage_translation',
            'image_prompts': 'stage_img_prompts',
            'image_generation_stage': 'stage_images',
            'image_generated': 'stage_generated_images',
            'voiceover': 'stage_voiceover',
            'subtitles': 'stage_subtitles',
            'montage': 'stage_montage'
        }
        self.init_ui()
        self.update_statistics()

    def init_ui(self):
        layout = QVBoxLayout(self)

        # Period selection
        period_layout = QHBoxLayout()
        self.period_label = QLabel()
        self.period_combo = QComboBox()
        self.period_combo.addItems(list(self.period_map.values()))
        self.period_combo.currentIndexChanged.connect(self.update_statistics)
        period_layout.addWidget(self.period_label)
        period_layout.addWidget(self.period_combo)
        period_layout.addStretch()

        # Statistics table
        self.table = QTableWidget()
        self.table.setColumnCount(2)
        self.table.horizontalHeader().setSectionResizeMode(0, QHeaderView.ResizeToContents)
        self.table.horizontalHeader().setSectionResizeMode(1, QHeaderView.Stretch)
        self.table.setEditTriggers(QTableWidget.NoEditTriggers)

        layout.addLayout(period_layout)
        layout.addWidget(self.table)
        self.setLayout(layout)
        
        self.retranslate_ui()

    def update_statistics(self):
        selected_period_text = self.period_combo.currentText()
        period_key = [key for key, value in self.period_map.items() if value == selected_period_text][0]
        
        stats = statistics_manager.get_statistics(period_key)

        # Definethe sort order of translation keys
        sort_order = [
            'stage_translation',
            'custom_stage_',
            'stage_img_prompts',
            'stage_images',
            'stage_generated_images',
            'stage_voiceover',
            'stage_subtitles',
            'stage_montage'
        ]

        def get_sort_key(item):
            event_type = item[0]
            if event_type.startswith('custom_stage_'):
                try:
                    return sort_order.index('custom_stage_')
                except ValueError:
                    return len(sort_order)
            
            translation_key = self.event_map.get(event_type, event_type)
            try:
                return sort_order.index(translation_key)
            except ValueError:
                return len(sort_order)

        sorted_stats = sorted(stats.items(), key=get_sort_key)

        self.table.setRowCount(len(sorted_stats))
        
        for i, (event_type, count) in enumerate(sorted_stats):
            stage_name = self.translate_event_type(event_type)
            self.table.setItem(i, 0, QTableWidgetItem(stage_name))
            
            count_item = QTableWidgetItem(str(count))
            count_item.setTextAlignment(Qt.AlignCenter)
            self.table.setItem(i, 1, count_item)

    def translate_event_type(self, event_type):
        # This maps the internal event names to translation keys
        if event_type.startswith('custom_stage_'):
            # For custom stages, we display the name directly
            return event_type.replace('custom_stage_', '').replace('_', ' ').capitalize()
        
        translation_key = self.event_map.get(event_type, event_type)
        return translator.translate(translation_key)

    def retranslate_ui(self):
        self.period_label.setText(translator.translate('statistics_period_label'))
        
        # Update combo box items without changing index
        current_text = self.period_combo.currentText()
        self.period_map = {
            'all_time': translator.translate('all_time'),
            'current_month': translator.translate('current_month'),
            'current_year': translator.translate('current_year')
        }
        self.period_combo.blockSignals(True)
        self.period_combo.clear()
        self.period_combo.addItems(list(self.period_map.values()))
        # Restore selection
        index = self.period_combo.findText(current_text)
        if index != -1: self.period_combo.setCurrentIndex(index)
        self.period_combo.blockSignals(False)

        self.table.setHorizontalHeaderLabels([
            translator.translate('statistics_stage_header'),
            translator.translate('statistics_count_header')
        ])
        
        # Also need to re-translate the items in the table
        self.update_statistics()

    def showEvent(self, event):
        """Override showEvent to refresh data when tab becomes visible."""
        super().showEvent(event)
        if event.spontaneous():
            return
        # Refresh statistics when the tab is shown
        self.update_statistics()
