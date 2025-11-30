from PySide6.QtWidgets import QWidget
from PySide6.QtGui import QPainter, QPen, QColor
from PySide6.QtCore import Qt, QTimer, QRectF

class LoadingSpinner(QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowFlags(Qt.WindowType.FramelessWindowHint)
        self.setAttribute(Qt.WidgetAttribute.WA_TranslucentBackground)
        
        self.timer = QTimer(self)
        self.timer.timeout.connect(self.rotate)
        self.timer.setInterval(15) # Update interval for smooth animation

        self._rotation = 0

    def rotate(self):
        self._rotation = (self._rotation + 6) % 360
        self.update()

    def paintEvent(self, event):
        painter = QPainter(self)
        painter.setRenderHint(QPainter.RenderHint.Antialiasing)
        
        # Define the pen for drawing the arc
        pen = QPen(QColor("#ffffff")) # Solid white color
        pen.setWidth(8) # Make it thick
        pen.setCapStyle(Qt.PenCapStyle.RoundCap)
        painter.setPen(pen)

        # Create a centered rectangle to draw the arc in
        side = min(self.width(), self.height())
        padding = 15
        rect = QRectF(
            (self.width() - side) / 2 + padding,
            (self.height() - side) / 2 + padding,
            side - 2 * padding,
            side - 2 * padding
        )

        # Draw the arc. The start angle is animated.
        # Angles are in 1/16th of a degree.
        start_angle = self._rotation * 16
        span_angle = 270 * 16  # Draw a 270-degree arc (a "C" shape)
        
        painter.drawArc(rect, start_angle, span_angle)

    def start(self):
        self.timer.start()
        self.show()

    def stop(self):
        self.timer.stop()
        self.hide()
