import sys
from PySide6.QtCore import QPoint, QRect, QSize, Qt
from PySide6.QtWidgets import QApplication, QLayout, QLayoutItem, QPushButton, QWidget


class FlowLayout(QLayout):
    def __init__(self, parent=None, margin=-1, hSpacing=-1, vSpacing=-1, v_align='middle'):
        super().__init__(parent)

        self.itemList = []
        self.m_hSpace = hSpacing
        self.m_vSpace = vSpacing
        self.v_align = v_align
        self.setContentsMargins(margin, margin, margin, margin)

    def __del__(self):
        while self.itemList:
            item = self.itemList.pop()
            del item

    def addItem(self, item):
        self.itemList.append(item)

    def count(self):
        return len(self.itemList)

    def itemAt(self, index):
        if 0 <= index < len(self.itemList):
            return self.itemList[index]
        return None

    def takeAt(self, index):
        if 0 <= index < len(self.itemList):
            return self.itemList.pop(index)
        return None

    def expandingDirections(self):
        return Qt.Orientation(0)

    def hasHeightForWidth(self):
        return True

    def heightForWidth(self, width):
        return self._doLayout(QRect(0, 0, width, 0), True)

    def setGeometry(self, rect):
        super().setGeometry(rect)
        self._doLayout(rect, False)

    def sizeHint(self):
        return self.minimumSize()

    def minimumSize(self):
        size = QSize()
        for item in self.itemList:
            size = size.expandedTo(item.minimumSize())
        
        margin = self.contentsMargins()
        size += QSize(margin.left() + margin.right(), margin.top() + margin.bottom())
        return size

    def _doLayout(self, rect, testOnly):
        left, top, right, bottom = self.getContentsMargins()
        effectiveRect = rect.adjusted(left, top, -right, -bottom)
        x = effectiveRect.x()
        y = effectiveRect.y()
        lineHeight = 0
        
        line_items = [] # List of tuples: (item, x_pos)

        def align_line_v(items, current_y, current_line_h):
            if testOnly: return
            for item, item_x in items:
                if self.v_align == 'top':
                    offset_y = 0
                else: # middle
                    offset_y = (current_line_h - item.sizeHint().height()) // 2
                item.setGeometry(QRect(QPoint(item_x, current_y + offset_y), item.sizeHint()))

        for item in self.itemList:
            spaceX = self.m_hSpace if self.m_hSpace >= 0 else self.spacing()
            if spaceX < 0: spaceX = 10
            
            spaceY = self.m_vSpace if self.m_vSpace >= 0 else self.spacing()
            if spaceY < 0: spaceY = 10
 
            item_w = item.sizeHint().width()
            item_h = item.sizeHint().height()
 
            nextX = x + item_w + spaceX
            if nextX - spaceX > effectiveRect.right() and lineHeight > 0:
                # Process the completed line
                align_line_v(line_items, y, lineHeight)
                
                # Reset for next line
                x = effectiveRect.x()
                y = y + lineHeight + spaceY
                nextX = x + item_w + spaceX
                lineHeight = 0
                line_items = []
 
            line_items.append((item, x))
            lineHeight = max(lineHeight, item_h)
            x = nextX
 
        # Align the last line
        align_line_v(line_items, y, lineHeight)

        return y + lineHeight - effectiveRect.y() + bottom
