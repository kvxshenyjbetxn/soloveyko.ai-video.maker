from PySide6.QtWidgets import QWidget, QVBoxLayout, QComboBox, QTableWidget, QTableWidgetItem, QLabel, QHBoxLayout, QHeaderView, QListWidget, QListWidgetItem, QSplitter
from PySide6.QtCore import Qt
import matplotlib.pyplot as plt
from matplotlib.backends.backend_qtagg import FigureCanvasQTAgg as FigureCanvas
from matplotlib.figure import Figure
from matplotlib.ticker import MaxNLocator
from matplotlib.dates import DateFormatter
from datetime import datetime, timedelta
import pandas as pd

from utils.translator import translator
from core.statistics_manager import statistics_manager
from utils.settings import settings_manager

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
        
        self.on_theme_changed()
        self.init_ui()
        self.update_statistics()

    def init_ui(self):
        main_layout = QVBoxLayout(self)
        
        # Period selection
        period_layout = QHBoxLayout()
        self.period_label = QLabel()
        self.period_combo = QComboBox()
        self.period_combo.addItems(list(self.period_map.values()))
        self.period_combo.currentIndexChanged.connect(self.update_statistics)
        period_layout.addWidget(self.period_label)
        period_layout.addWidget(self.period_combo)
        period_layout.addStretch()
        main_layout.addLayout(period_layout)

        splitter = QSplitter(Qt.Horizontal)

        # Left side (Chart and Table)
        left_widget = QWidget()
        left_layout = QVBoxLayout(left_widget)
        
        self.figure = Figure(figsize=(5, 3))
        self.canvas = FigureCanvas(self.figure)
        left_layout.addWidget(self.canvas)
        
        self.table = QTableWidget()
        self.table.setColumnCount(2)
        self.table.horizontalHeader().setSectionResizeMode(0, QHeaderView.ResizeToContents)
        self.table.horizontalHeader().setSectionResizeMode(1, QHeaderView.Stretch)
        self.table.setEditTriggers(QTableWidget.NoEditTriggers)
        left_layout.addWidget(self.table)
        
        splitter.addWidget(left_widget)

        # Right side (Stage selection)
        right_widget = QWidget()
        right_layout = QVBoxLayout(right_widget)
        right_layout.setContentsMargins(0, 0, 0, 0)
        
        self.stage_list = QListWidget()
        right_layout.addWidget(self.stage_list)
        self.populate_stage_list()
        self.stage_list.itemChanged.connect(self.update_statistics)

        splitter.addWidget(right_widget)
        
        splitter.setSizes([800, 200])

        main_layout.addWidget(splitter)
        self.setLayout(main_layout)
        
        self.retranslate_ui()

    def populate_stage_list(self):
        self.stage_list.blockSignals(True)
        self.stage_list.clear()
        # Sort event_map by the translated values for consistent order
        sorted_events = sorted(self.event_map.items(), key=lambda item: self.translate_event_type(item[0]))
        for event_key, trans_key in sorted_events:
            item = QListWidgetItem(self.translate_event_type(event_key))
            item.setData(Qt.UserRole, event_key)
            if event_key == 'image_generated':
                item.setCheckState(Qt.Unchecked)
            else:
                item.setCheckState(Qt.Checked)
            self.stage_list.addItem(item)
        self.stage_list.blockSignals(False)

    def get_selected_stages(self):
        selected = []
        for i in range(self.stage_list.count()):
            item = self.stage_list.item(i)
            if item.checkState() == Qt.Checked:
                selected.append(item.data(Qt.UserRole))
        return selected

    def on_theme_changed(self):
        theme = settings_manager.get('theme', 'light')
        if theme == 'light':
            plt.style.use('default')
        else: # dark or black
            plt.style.use('dark_background')
            if theme == 'black':
                plt.rcParams.update({
                    "figure.facecolor": "black",
                    "axes.facecolor": "black",
                    "savefig.facecolor": "black",
                })
        
        if hasattr(self, 'figure'):
            self.update_statistics()

    def update_chart(self, period_key):
        selected_stages = self.get_selected_stages()
        
        self.figure.clear()
        ax = self.figure.add_subplot(111)
        
        raw_data = statistics_manager.get_daily_statistics_by_event(period_key)

        if not raw_data or not any(event in selected_stages for event in raw_data):
            ax.text(0.5, 0.5, translator.translate('no_data_available'), ha='center', va='center', transform=ax.transAxes)
            self.canvas.draw()
            return

        flat_data = []
        for event, records in raw_data.items():
            if event in selected_stages:
                for date_str, count in records:
                    flat_data.append({'event': event, 'date': pd.to_datetime(date_str), 'count': count})
        
        if not flat_data:
            ax.text(0.5, 0.5, translator.translate('no_data_available'), ha='center', va='center', transform=ax.transAxes)
            self.canvas.draw()
            return

        df = pd.DataFrame(flat_data)
        
        pivot_df = df.pivot_table(index='date', columns='event', values='count', fill_value=0)

        # Create a full date range to fill gaps
        now = datetime.now()
        if period_key == 'current_month':
            start_date = now.replace(day=1)
            end_date = now
        elif period_key == 'current_year':
            start_date = now.replace(month=1, day=1)
            end_date = now
        else: # all_time
            start_date = pivot_df.index.min()
            end_date = pivot_df.index.max()
            if pd.isna(start_date): start_date = now
            if pd.isna(end_date): end_date = now
             
        full_date_range = pd.date_range(start=start_date.date(), end=end_date.date(), freq='D')
        pivot_df = pivot_df.reindex(full_date_range, fill_value=0)
        
        for column in pivot_df.columns:
            ax.plot(pivot_df.index, pivot_df[column], marker='o', linestyle='-', label=self.translate_event_type(column))

        ax.set_xlabel(translator.translate('date_axis_label'))
        ax.set_ylabel(translator.translate('count_axis_label'))
        ax.set_title(translator.translate('daily_usage_chart_title'))
        ax.legend()
        
        date_format = DateFormatter('%Y-%m-%d')
        ax.xaxis.set_major_formatter(date_format)
        self.figure.autofmt_xdate(rotation=45)
        ax.grid(True, linestyle='--', alpha=0.6)
        
        ax.yaxis.set_major_locator(MaxNLocator(integer=True))
        if pivot_df.empty or pivot_df.sum().sum() == 0: # Check if all data is zero
             ax.set_ylim(bottom=0, top=5) # Set a default y-axis limit if no data
        else:
            ax.set_ylim(bottom=0)

        # Set x-axis limits to give padding around single-day data
        if len(full_date_range) == 1:
            ax.set_xlim(full_date_range[0] - timedelta(days=1), full_date_range[0] + timedelta(days=1))


        self.canvas.draw()

    def update_statistics(self):
        if not self.isVisible(): return
        selected_period_text = self.period_combo.currentText()
        period_key = [key for key, value in self.period_map.items() if value == selected_period_text]
        if not period_key: return
        period_key = period_key[0]

        self.update_chart(period_key)
        
        stats = statistics_manager.get_statistics(period_key)
        sort_order = ['stage_translation', 'custom_stage_', 'stage_img_prompts', 'stage_images', 'stage_generated_images', 'stage_voiceover', 'stage_subtitles', 'stage_montage']

        def get_sort_key(item):
            event_type = item[0]
            if event_type.startswith('custom_stage_'):
                try: return sort_order.index('custom_stage_')
                except ValueError: return len(sort_order)
            translation_key = self.event_map.get(event_type, event_type)
            try: return sort_order.index(translation_key)
            except ValueError: return len(sort_order)

        sorted_stats = sorted(stats.items(), key=get_sort_key)
        self.table.setRowCount(len(sorted_stats))
        for i, (event_type, count) in enumerate(sorted_stats):
            stage_name = self.translate_event_type(event_type)
            self.table.setItem(i, 0, QTableWidgetItem(stage_name))
            count_item = QTableWidgetItem(str(count))
            count_item.setTextAlignment(Qt.AlignCenter)
            self.table.setItem(i, 1, count_item)

    def translate_event_type(self, event_type):
        if event_type.startswith('custom_stage_'):
            return event_type.replace('custom_stage_', '').replace('_', ' ').capitalize()
        translation_key = self.event_map.get(event_type, event_type)
        return translator.translate(translation_key)

    def retranslate_ui(self):
        self.period_label.setText(translator.translate('statistics_period_label'))
        
        current_text = self.period_combo.currentText()
        self.period_map = {
            'all_time': translator.translate('all_time'),
            'current_month': translator.translate('current_month'),
            'current_year': translator.translate('current_year')
        }
        self.period_combo.blockSignals(True)
        self.period_combo.clear()
        self.period_combo.addItems(list(self.period_map.values()))
        # Find by value, not key
        current_key = None
        for key, value in self.period_map.items():
            if value == current_text:
                current_key = key
                break
        if not current_key and self.period_combo.count() > 0:
             self.period_combo.setCurrentIndex(0)
        else:
             index = list(self.period_map.keys()).index(current_key) if current_key in self.period_map else -1
             if index != -1: self.period_combo.setCurrentIndex(index)
        self.period_combo.blockSignals(False)

        self.table.setHorizontalHeaderLabels([translator.translate('statistics_stage_header'), translator.translate('statistics_count_header')])
        self.populate_stage_list()
        self.update_statistics()

    def showEvent(self, event):
        super().showEvent(event)
        if event.spontaneous(): return
        self.update_statistics()
