from PySide6.QtCore import QPropertyAnimation, QEasingCurve, QAbstractAnimation, QSize
from PySide6.QtWidgets import QWidget, QGraphicsOpacityEffect

class Animator:
    @staticmethod
    def fade_in(widget: QWidget, duration: int = 300):
        """
        Fades in a widget by animating its opacity from 0 to 1.
        """
        if not widget:
            return

        # Ensure widget is visible
        widget.setVisible(True)
        
        # Create opacity effect
        effect = QGraphicsOpacityEffect(widget)
        widget.setGraphicsEffect(effect)
        
        # Create animation
        animation = QPropertyAnimation(effect, b"opacity")
        animation.setDuration(duration)
        animation.setStartValue(0.0)
        animation.setEndValue(1.0)
        animation.setEasingCurve(QEasingCurve.Type.InOutQuad)
        
        # Cleanup effect after animation (optional, but good for performance)
        # Note: Keeping the effect might be needed if we want to fade out later. 
        # For simple fade-in, we typically leave it or remove it. 
        # Removing it resets opacity to 1.0 (widget default), which is what we want.
        def on_finished():
            widget.setGraphicsEffect(None)
            
        animation.finished.connect(on_finished)
        animation.start(QAbstractAnimation.DeletionPolicy.DeleteWhenStopped)
        
        # Keep reference to avoid garbage collection
        setattr(widget, "_fade_animation", animation)

    @staticmethod
    def slide_in_down(widget: QWidget, duration: int = 300):
        """
        Animates a widget sliding down (expanding height).
        Note: The widget must be in a layout that allows expansion.
        """
        if not widget:
            return

        widget.setVisible(True)
        # Force layout to calculate size hint
        widget.updateGeometry()
        target_height = widget.sizeHint().height()
        
        # If sizeHint is 0 (it happens), try to measure
        if target_height <= 0:
             # Fallback: let it be visible and measure, but that might flicker.
             # Better approach: Start with 0 height property, animate to detected height.
             # This assumes widget uses minimumHeight/maximumHeight or fixedHeight.
             pass

        # We will animate maximumHeight
        # First, set current max height to 0
        widget.setMaximumHeight(0)
        
        animation = QPropertyAnimation(widget, b"maximumHeight")
        animation.setDuration(duration)
        animation.setStartValue(0)
        # We want it to be unrestricted at the end, or a specific height.
        # Since we don't know the exact height it *should* be (dynamic content),
        # we can animate to a large enough number, or better:
        # 1. Animate to sizeHint().height()
        # 2. On finish, set maximumHeight to QWIDGETSIZE_MAX
        
        # Let's try to get a reasonable target height.
        # Temporarily set unrestricted to measure
        widget.setMaximumHeight(16777215) # QWIDGETSIZE_MAX
        widget.adjustSize()
        target_height = widget.sizeHint().height()
        # Reset to 0
        widget.setMaximumHeight(0)

        animation.setEndValue(target_height)
        animation.setEasingCurve(QEasingCurve.Type.OutQuad)

        def on_finished():
            # Release the constraint so it can resize if content changes
            widget.setMaximumHeight(16777215) 

        animation.finished.connect(on_finished)
        animation.start(QAbstractAnimation.DeletionPolicy.DeleteWhenStopped)

        setattr(widget, "_slide_animation", animation)

    @staticmethod
    def slide_out_up(widget: QWidget, duration: int = 300, on_complete=None):
        """
        Animates a widget sliding up (collapsing height) and then hides it.
        """
        if not widget or not widget.isVisible():
            if on_complete:
                on_complete()
            return

        # Start from current height
        current_height = widget.height()
        
        # We animate maximumHeight to 0
        widget.setMaximumHeight(current_height)
        
        animation = QPropertyAnimation(widget, b"maximumHeight")
        animation.setDuration(duration)
        animation.setStartValue(current_height)
        animation.setEndValue(0)
        animation.setEasingCurve(QEasingCurve.Type.OutQuad)

        def on_finished():
            widget.setVisible(False)
            # Reset maximumHeight so it can be shown again later
            widget.setMaximumHeight(16777215) # QWIDGETSIZE_MAX
            if on_complete:
                on_complete()

        animation.finished.connect(on_finished)
        animation.start(QAbstractAnimation.DeletionPolicy.DeleteWhenStopped)

        setattr(widget, "_slide_out_animation", animation)
