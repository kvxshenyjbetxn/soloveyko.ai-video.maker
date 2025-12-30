
from PySide6.QtWidgets import QWidget, QHBoxLayout, QSlider, QDoubleSpinBox
from PySide6.QtCore import Qt, Signal

class SliderWithSpinBox(QWidget):
    valueChanged = Signal(float)

    def __init__(self, parent=None):
        super().__init__(parent)
        self._multiplier = 100.0  # Precision for slider (2 decimal places)
        self.init_ui()

    def init_ui(self):
        layout = QHBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        
        self.slider = QSlider(Qt.Orientation.Horizontal)
        self.spinbox = QDoubleSpinBox()
        
        layout.addWidget(self.slider)
        layout.addWidget(self.spinbox)
        
        # Connect signals
        self.slider.valueChanged.connect(self._on_slider_changed)
        self.spinbox.valueChanged.connect(self._on_spinbox_changed)

    def setRange(self, min_val, max_val):
        self.spinbox.setRange(min_val, max_val)
        self.slider.setRange(int(min_val * self._multiplier), int(max_val * self._multiplier))

    def setSingleStep(self, step):
        self.spinbox.setSingleStep(step)
        # Update multiplier based on step if needed? 
        # For simplicity, keeping fixed multiplier 100 allows 0.01 precision.
        # If step is 0.1, it still works fine.
        
    def setSuffix(self, suffix):
        self.spinbox.setSuffix(suffix)

    def setValue(self, value):
        self.spinbox.setValue(value)
        # Update slider even if signals are blocked
        int_val = int(value * self._multiplier)
        if abs(self.slider.value() - int_val) > 1:
            self.slider.setValue(int_val)

    def value(self):
        return self.spinbox.value()

    def _on_slider_changed(self, value):
        float_val = value / self._multiplier
        if abs(self.spinbox.value() - float_val) > 0.001: # Avoid feedback loop
            self.spinbox.setValue(float_val)
            self.valueChanged.emit(float_val)

    def _on_spinbox_changed(self, value):
        int_val = int(value * self._multiplier)
        if abs(self.slider.value() - int_val) > 1: # Avoid feedback loop
            self.slider.setValue(int_val)
        self.valueChanged.emit(value)

    def blockSignals(self, b):
        self.slider.blockSignals(b)
        self.spinbox.blockSignals(b)
        return super().blockSignals(b)
