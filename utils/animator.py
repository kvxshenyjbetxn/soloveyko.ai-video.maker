from PySide6.QtCore import QPropertyAnimation, QEasingCurve, QAbstractAnimation, QSize
from PySide6.QtWidgets import QWidget, QGraphicsOpacityEffect

class Animator:
    @staticmethod
    def _stop_animations(widget: QWidget):
        """Stops any existing animations on the widget to prevent conflicts."""
        for attr in ["_slide_animation", "_slide_out_animation", "_fade_animation"]:
            if hasattr(widget, attr):
                anim = getattr(widget, attr)
                if anim:
                    try:
                        if anim.state() == QAbstractAnimation.State.Running:
                            anim.stop()
                    except RuntimeError:
                        # Internal C++ object already deleted
                        pass
                setattr(widget, attr, None)

    @staticmethod
    def fade_in(widget: QWidget, duration: int = 300):
        """
        Fades in a widget by animating its opacity from 0 to 1.
        """
        if not widget:
            return

        Animator._stop_animations(widget)

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
        animation.setEasingCurve(QEasingCurve.Type.OutCubic)
        
        def on_finished():
            widget.setGraphicsEffect(None)
            setattr(widget, "_fade_animation", None)
            
        animation.finished.connect(on_finished)
        animation.start(QAbstractAnimation.DeletionPolicy.DeleteWhenStopped)
        
        # Keep reference to avoid garbage collection
        setattr(widget, "_fade_animation", animation)

    @staticmethod
    def slide_in_down(widget: QWidget, duration: int = 400):
        """
        Combined slide and fade-in animation.
        """
        if not widget:
            return

        Animator._stop_animations(widget)
        
        widget.setVisible(True)
        widget.updateGeometry()
        
        # Force a measure of the target height
        widget.setMaximumHeight(16777215)
        widget.adjustSize()
        target_height = widget.sizeHint().height()
        
        # If still 0, the layout might need more info, but we proceed
        if target_height <= 0:
            target_height = 40 # Typical minimal height for a row

        # Set up opacity effect
        effect = QGraphicsOpacityEffect(widget)
        widget.setGraphicsEffect(effect)
        
        # Opacity Animation
        fade_anim = QPropertyAnimation(effect, b"opacity")
        fade_anim.setDuration(duration)
        fade_anim.setStartValue(0.0)
        fade_anim.setEndValue(1.0)
        fade_anim.setEasingCurve(QEasingCurve.Type.OutCubic)

        # Height Animation
        widget.setMaximumHeight(0)
        height_anim = QPropertyAnimation(widget, b"maximumHeight")
        height_anim.setDuration(duration)
        height_anim.setStartValue(0)
        height_anim.setEndValue(target_height)
        height_anim.setEasingCurve(QEasingCurve.Type.OutCubic)

        def on_finished():
            widget.setMaximumHeight(16777215)
            widget.setGraphicsEffect(None)
            setattr(widget, "_fade_animation", None)
            setattr(widget, "_slide_animation", None)

        height_anim.finished.connect(on_finished)
        
        fade_anim.start(QAbstractAnimation.DeletionPolicy.DeleteWhenStopped)
        height_anim.start(QAbstractAnimation.DeletionPolicy.DeleteWhenStopped)

        setattr(widget, "_fade_animation", fade_anim)
        setattr(widget, "_slide_animation", height_anim)

    @staticmethod
    def slide_out_up(widget: QWidget, duration: int = 400, on_complete=None):
        """
        Combined slide and fade-out animation.
        """
        if not widget or not widget.isVisible():
            if on_complete: on_complete()
            return

        Animator._stop_animations(widget)

        current_height = widget.height()
        
        # Set up opacity effect
        effect = QGraphicsOpacityEffect(widget)
        widget.setGraphicsEffect(effect)
        
        # Opacity Animation
        fade_anim = QPropertyAnimation(effect, b"opacity")
        fade_anim.setDuration(duration)
        fade_anim.setStartValue(1.0)
        fade_anim.setEndValue(0.0)
        fade_anim.setEasingCurve(QEasingCurve.Type.OutCubic)

        # Height Animation
        widget.setMaximumHeight(current_height)
        height_anim = QPropertyAnimation(widget, b"maximumHeight")
        height_anim.setDuration(duration)
        height_anim.setStartValue(current_height)
        height_anim.setEndValue(0)
        height_anim.setEasingCurve(QEasingCurve.Type.OutCubic)

        def on_finished():
            widget.setVisible(False)
            widget.setMaximumHeight(16777215) 
            widget.setGraphicsEffect(None)
            setattr(widget, "_fade_animation", None)
            setattr(widget, "_slide_out_animation", None)
            if on_complete:
                on_complete()

        height_anim.finished.connect(on_finished)
        
        fade_anim.start(QAbstractAnimation.DeletionPolicy.DeleteWhenStopped)
        height_anim.start(QAbstractAnimation.DeletionPolicy.DeleteWhenStopped)

        setattr(widget, "_fade_animation", fade_anim)
        setattr(widget, "_slide_out_animation", height_anim)
