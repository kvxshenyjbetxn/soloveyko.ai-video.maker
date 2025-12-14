from PySide6.QtWidgets import QTabWidget
from utils.animator import Animator

class AnimatedTabWidget(QTabWidget):
    """
    A QTabWidget that applies a fade-in animation to the new tab content
    when switching tabs.
    """
    def __init__(self, parent=None):
        super().__init__(parent)
        self.currentChanged.connect(self._animate_current_tab)

    def _animate_current_tab(self, index):
        """
        Called when the current tab changes. Animates the new widget.
        """
        current_widget = self.widget(index)
        if current_widget:
            # We use a short duration for snappy feel
            Animator.fade_in(current_widget, duration=250)
