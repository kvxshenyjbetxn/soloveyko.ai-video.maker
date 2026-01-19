from gui.gallery_tab.image_viewer import ImageViewer

class VideoViewer(ImageViewer):
    """
    Alias for ImageViewer since it now handles both images and videos
    with correct sizing and navigation.
    """
    def __init__(self, media_paths, current_index, parent=None):
        super().__init__(media_paths, current_index, parent)
