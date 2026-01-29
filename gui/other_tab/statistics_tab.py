from PySide6.QtWidgets import QWidget, QVBoxLayout, QLabel, QHBoxLayout, QPushButton, QMessageBox
from PySide6.QtCore import Qt
import matplotlib.pyplot as plt
from matplotlib.backends.backend_qtagg import FigureCanvasQTAgg as FigureCanvas
from matplotlib.figure import Figure
from matplotlib.ticker import MaxNLocator
from matplotlib.dates import DateFormatter
from datetime import datetime

from utils.translator import translator
from core.statistics_manager import statistics_manager
from utils.settings import settings_manager

class StatisticsTab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        self.main_window = main_window
        
        self.on_theme_changed()
        self.init_ui()
        self.update_statistics()

    def init_ui(self):
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(20, 20, 20, 20)
        main_layout.setSpacing(15)
        
        header_layout = QHBoxLayout()
        header_layout.setContentsMargins(0, 0, 0, 0)
        
        # Left side with title and total count (nested)
        left_layout = QVBoxLayout()
        left_layout.setSpacing(0)
        self.title_label = QLabel()
        self.total_videos_label = QLabel()
        left_layout.addWidget(self.title_label)
        left_layout.addWidget(self.total_videos_label)
        header_layout.addLayout(left_layout)
        
        header_layout.addStretch()
        
        # Clear button
        self.clear_button = QPushButton()
        self.clear_button.clicked.connect(self.clear_statistics)
        header_layout.addWidget(self.clear_button)
        
        main_layout.addLayout(header_layout)

        # Chart
        self.figure = Figure()
        self.canvas = FigureCanvas(self.figure)
        from PySide6.QtWidgets import QSizePolicy
        self.canvas.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Expanding)
        main_layout.addWidget(self.canvas)
        
        self.setLayout(main_layout)
        
        self.retranslate_ui()

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

    def update_chart(self):
        self.figure.clear()
        ax = self.figure.add_subplot(111)
        
        daily_counts = statistics_manager.get_daily_video_counts()

        if not daily_counts:
            ax.text(0.5, 0.5, translator.translate('no_data_available'), ha='center', va='center', transform=ax.transAxes)
            self.canvas.draw()
            return

        dates = [datetime.strptime(d, '%d-%m-%Y') for d in daily_counts.keys()]
        counts = list(daily_counts.values())

        accent_color = settings_manager.get('accent_color', '#3f51b5')
        line, = ax.plot(dates, counts, marker='o', linestyle='-', label=translator.translate('videos_created_label'), color=accent_color)

        ax.set_xlabel(translator.translate('date_axis_label'))
        ax.set_ylabel(translator.translate('video_count_axis_label'))
        ax.set_title(translator.translate('daily_video_chart_title'))
        ax.legend()

        date_format = DateFormatter('%d-%m-%Y')
        ax.xaxis.set_major_formatter(date_format)
        ax.set_xticks(dates)
        self.figure.autofmt_xdate(rotation=45)
        ax.grid(True, linestyle='--', alpha=0.6)

        ax.yaxis.set_major_locator(MaxNLocator(integer=True))
        if not counts or sum(counts) == 0:
            ax.set_ylim(bottom=0, top=5)
        else:
            ax.set_ylim(bottom=0)

        # Create an annotation object for hover effect
        annot = ax.annotate("", xy=(0,0), xytext=(-20,20), textcoords="offset points",
                            bbox=dict(boxstyle="round", fc="w"),
                            arrowprops=dict(arrowstyle="->"))
        annot.set_visible(False)

        def update_annot(ind):
            pos = line.get_xydata()[ind["ind"][0]]
            annot.xy = pos
            date_str = dates[ind["ind"][0]].strftime('%d-%m-%Y')
            text = f"{date_str}: {counts[ind['ind'][0]]}"
            annot.set_text(text)
            annot.get_bbox_patch().set_alpha(0.4)

        def hover(event):
            vis = annot.get_visible()
            if event.inaxes == ax:
                cont, ind = line.contains(event)
                if cont:
                    update_annot(ind)
                    annot.set_visible(True)
                    self.figure.canvas.draw_idle()
                else:
                    if vis:
                        annot.set_visible(False)
                        self.figure.canvas.draw_idle()

        self.figure.canvas.mpl_connect("motion_notify_event", hover)

        # ТУТ РЕГУЛЮВАТИ ВНУТРІШНІ ВІДСТУПИ ГРАФІКА
        self.figure.subplots_adjust(left=0.02, right=0.98, top=0.96, bottom=0.14)
        self.canvas.draw()

    def update_statistics(self):
        if not self.isVisible(): return
        
        daily_counts = statistics_manager.get_daily_video_counts()
        total_videos = sum(daily_counts.values())
        
        text = f"<b>{translator.translate('total_videos_created')}: {total_videos}</b>"
        self.total_videos_label.setText(text)

        self.update_chart()

    def clear_statistics(self):
        reply = QMessageBox.question(self, 
                                     translator.translate('clear_statistics_title'),
                                     translator.translate('clear_statistics_confirm'),
                                     QMessageBox.Yes | QMessageBox.No, QMessageBox.No)

        if reply == QMessageBox.Yes:
            statistics_manager.clear_all_data()
            self.update_statistics()
            QMessageBox.information(self,
                                      translator.translate('statistics_cleared_title'),
                                      translator.translate('statistics_cleared_message'))


    def retranslate_ui(self):
        self.title_label.setText(f"<h1>{translator.translate('statistics_title')}</h1>")
        self.clear_button.setText(translator.translate('clear_statistics_button'))
        # The chart will be redrawn with new labels
        self.update_statistics()

    def showEvent(self, event):
        super().showEvent(event)
        if event.spontaneous(): return
        self.update_statistics()
