import React, { useState, useEffect } from 'react';
import { useI18n } from '../contexts/I18nContext';
import { GetGalleryImages } from '../../wailsjs/go/main/App';
import { EventsOn, EventsOff } from '../../wailsjs/runtime/runtime';
import { utils } from '../../wailsjs/go/models';
import './gallery.css';

interface SelectedMedia {
    name: string;
    url: string;
    path: string;
}

export const Gallery = () => {
    const { t } = useI18n();
    const [tasks, setTasks] = useState<utils.GalleryTask[]>([]);
    const [selectedMedia, setSelectedMedia] = useState<SelectedMedia | null>(null);
    const [selectedPaths, setSelectedPaths] = useState<Set<string>>(new Set());
    const [collapsedTasks, setCollapsedTasks] = useState<Set<string>>(new Set());
    const [collapsedTemplates, setCollapsedTemplates] = useState<Set<string>>(new Set());
    const [loading, setLoading] = useState(true);
    const [isSelectionMode, setIsSelectionMode] = useState(false);

    const loadGallery = async () => {
        try {
            setLoading(true);
            const data = await GetGalleryImages();
            setTasks(data || []);
            // Notify main app to update tab visibility
            // @ts-ignore
            if (window.runtime) window.runtime.EventsEmit('galleryUpdate');
        } catch (error) {
            console.error('Failed to load gallery:', error);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        loadGallery();

        // Listen for completed image stages to refresh gallery
        EventsOn('stageStatus', (id: string, stage: string, status: string, message: string) => {
            if (stage === 'image' && status === 'completed') {
                loadGallery();
            }
        });

        // Listen for internal gallery updates
        EventsOn('galleryUpdate', () => {
            loadGallery();
        });

        return () => {
            EventsOff('stageStatus');
            EventsOff('galleryUpdate');
        };
    }, []);

    const toggleTask = (taskName: string) => {
        setCollapsedTasks(prev => {
            const newSet = new Set(prev);
            if (newSet.has(taskName)) {
                newSet.delete(taskName);
            } else {
                newSet.add(taskName);
            }
            return newSet;
        });
    };

    const toggleTemplate = (taskName: string, templateName: string) => {
        const key = `${taskName}_${templateName}`;
        setCollapsedTemplates(prev => {
            const newSet = new Set(prev);
            if (newSet.has(key)) {
                newSet.delete(key);
            } else {
                newSet.add(key);
            }
            return newSet;
        });
    };

    const toggleImageSelection = (path: string, e?: React.MouseEvent) => {
        if (e) e.stopPropagation();
        setSelectedPaths(prev => {
            const newSet = new Set(prev);
            if (newSet.has(path)) {
                newSet.delete(path);
            } else {
                newSet.add(path);
            }
            // Update selection mode based on the new size
            if (newSet.size === 0) setIsSelectionMode(false);
            else setIsSelectionMode(true);
            return newSet;
        });
    };

    const handleDeleteImage = async (path: string, e?: React.MouseEvent) => {
        if (e) e.stopPropagation();

        // @ts-ignore
        const success = await window.go.main.App.DeleteGalleryImage(path);
        if (success) {
            loadGallery();
            if (selectedMedia?.path === path) setSelectedMedia(null);
        }
    };

    const handleBulkDelete = async () => {
        if (selectedPaths.size === 0) return;

        // @ts-ignore
        await window.go.main.App.DeleteGalleryImages(Array.from(selectedPaths));
        setSelectedPaths(new Set());
        setIsSelectionMode(false);
        loadGallery();
    };

    const clearSelection = () => {
        setSelectedPaths(new Set());
        setIsSelectionMode(false);
    };

    // List of all images for keyboard navigation
    const flatImages = tasks.reduce((acc: any[], task) => {
        task.templates?.forEach(tpl => {
            tpl.images?.forEach(img => {
                acc.push({ ...img, taskName: task.name, templateName: tpl.name });
            });
        });
        return acc;
    }, []);

    const handleKeyDown = (e: KeyboardEvent) => {
        if (!selectedMedia) return;

        if (e.key === 'Escape') {
            setSelectedMedia(null);
            return;
        }

        const currentIndex = flatImages.findIndex(img => img.url === selectedMedia.url);
        if (currentIndex === -1) return;

        if (e.key === 'ArrowRight') {
            if (currentIndex < flatImages.length - 1) {
                const nextIndex = currentIndex + 1;
                setSelectedMedia({
                    name: flatImages[nextIndex].name,
                    url: flatImages[nextIndex].url,
                    path: flatImages[nextIndex].path
                });
            }
        } else if (e.key === 'ArrowLeft') {
            if (currentIndex > 0) {
                const prevIndex = currentIndex - 1;
                setSelectedMedia({
                    name: flatImages[prevIndex].name,
                    url: flatImages[prevIndex].url,
                    path: flatImages[prevIndex].path
                });
            }
        } else if (e.key === 'Delete' || e.key === 'Backspace') {
            handleDeleteImage(selectedMedia.path);
        }
    };

    useEffect(() => {
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [selectedMedia, flatImages]);

    return (
        <div className="content-wrapper animate-fade gallery-page">
            <div className="gallery-header-top">
                <h2>{t('gallery.title') || 'Gallery'}</h2>
                <div className="gallery-stats-main">
                    <span className="total-count-badge">
                        {flatImages.length} {t('gallery.imagesCount') || 'images'}
                    </span>
                    <button className="btn-refresh-icon" onClick={loadGallery} title={t('gallery.refresh')}>
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M23 4v6h-6M1 20v-6h6M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" /></svg>
                    </button>
                </div>
            </div>

            {isSelectionMode && selectedPaths.size > 0 && (
                <div className="gallery-bulk-actions animate-slide-down">
                    <div className="bulk-info">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: '8px' }}><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" /><polyline points="22 4 12 14.01 9 11.01" /></svg>
                        <span>{t('gallery.selected') || 'Selected'}: <strong>{selectedPaths.size}</strong></span>
                    </div>
                    <div className="bulk-buttons">
                        <button className="bulk-btn-cancel" onClick={clearSelection}>
                            {t('common.cancel') || 'Cancel'}
                        </button>
                        <button className="bulk-btn-delete" onClick={handleBulkDelete}>
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: '6px' }}><path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2M10 11v6M14 11v6" /></svg>
                            {t('common.delete') || 'Delete'}
                        </button>
                    </div>
                </div>
            )}
            <div className={`gallery-scroll-container premium-scrollbar ${isSelectionMode ? 'with-bulk' : ''}`}>
                {loading && tasks.length === 0 ? (
                    <div className="gallery-empty">
                        <p>{t('common.loading') || 'Loading...'}</p>
                    </div>
                ) : tasks.length === 0 ? (
                    <div className="gallery-empty">
                        <p>{t('gallery.empty')}</p>
                    </div>
                ) : (
                    <div className="gallery-tasks">
                        {tasks.map((task, tIndex) => {
                            const isTaskCollapsed = collapsedTasks.has(task.name);
                            const taskImagesCount = task.templates?.reduce((sum, tpl) => sum + (tpl.images?.length || 0), 0) || 0;

                            return (
                                <div key={tIndex} className={`task-section ${isTaskCollapsed ? 'is-collapsed' : ''}`}>
                                    <div className="task-header" onClick={() => toggleTask(task.name)}>
                                        <div className="task-name">
                                            <svg className="section-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" /></svg>
                                            {task.name}
                                            <span className="section-count">{taskImagesCount}</span>
                                        </div>
                                        <svg className={`collapse-chevron ${isTaskCollapsed ? 'collapsed' : ''}`} width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M6 9l6 6 6-6" /></svg>
                                    </div>

                                    <div className="task-collapsible-wrapper">
                                        <div className="task-content">
                                            {task.templates.map((template, tmpIndex) => {
                                                const isTemplateCollapsed = collapsedTemplates.has(`${task.name}_${template.name}`);
                                                const templateImagesCount = template.images?.length || 0;

                                                return (
                                                    <div key={tmpIndex} className={`template-section ${isTemplateCollapsed ? 'is-collapsed' : ''}`}>
                                                        <div className="template-header" onClick={() => toggleTemplate(task.name, template.name)}>
                                                            <div className="template-name">
                                                                <svg className="section-icon-minor" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="3" y="3" width="7" height="7" /><rect x="14" y="3" width="7" height="7" /><rect x="14" y="14" width="7" height="7" /><rect x="3" y="14" width="7" height="7" /></svg>
                                                                {template.name}
                                                                <span className="section-count">{templateImagesCount}</span>
                                                            </div>
                                                            <svg className={`collapse-chevron-minor ${isTemplateCollapsed ? 'collapsed' : ''}`} width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M6 9l6 6 6-6" /></svg>
                                                        </div>

                                                        <div className="template-collapsible-wrapper">
                                                            <div className="template-content gallery-grid">
                                                                {template.images.map((image, imgIndex) => (
                                                                    <div
                                                                        key={imgIndex}
                                                                        className={`gallery-card ${selectedPaths.has(image.path) ? 'selected' : ''}`}
                                                                        onClick={() => isSelectionMode ? toggleImageSelection(image.path) : setSelectedMedia({
                                                                            name: image.name,
                                                                            url: image.url,
                                                                            path: image.path
                                                                        })}
                                                                    >
                                                                        <div className="media-container">
                                                                            <img src={image.url} alt={image.name} loading="lazy" />
                                                                            <div className="media-overlay">
                                                                                <button
                                                                                    className="card-delete-btn"
                                                                                    onClick={(e) => handleDeleteImage(image.path, e)}
                                                                                    title={t('common.delete')}
                                                                                >
                                                                                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2M10 11v6M14 11v6" /></svg>
                                                                                </button>
                                                                                <div
                                                                                    className={`card-checkbox ${selectedPaths.has(image.path) ? 'checked' : ''}`}
                                                                                    onClick={(e) => toggleImageSelection(image.path, e)}
                                                                                >
                                                                                    {selectedPaths.has(image.path) && (
                                                                                        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3"><polyline points="20 6 9 17 4 12" /></svg>
                                                                                    )}
                                                                                </div>
                                                                            </div>
                                                                        </div>
                                                                        <div className="media-info">
                                                                            <span className="media-name">{image.name}</span>
                                                                        </div>
                                                                    </div>
                                                                ))}
                                                            </div>
                                                        </div>
                                                    </div>
                                                );
                                            })}
                                        </div>
                                    </div>
                                </div>
                            );
                        })}
                    </div>
                )}
            </div>

            {selectedMedia && (
                <div className="media-modal open" onClick={() => setSelectedMedia(null)}>
                    <div className="modal-content" onClick={e => e.stopPropagation()}>
                        <div className="modal-header">
                            <span className="modal-title">{selectedMedia.name}</span>
                            <div className="modal-actions">
                                <button className="modal-delete-btn" onClick={() => handleDeleteImage(selectedMedia.path)}>
                                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: '6px' }}><path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2M10 11v6M14 11v6" /></svg>
                                    {t('common.delete')}
                                </button>
                                <button className="modal-close-static" onClick={() => setSelectedMedia(null)}>&times;</button>
                            </div>
                        </div>
                        <div className="modal-image-wrapper" onClick={() => setSelectedMedia(null)}>
                            <img src={selectedMedia.url} alt={selectedMedia.name} className="animate-fade" />
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
};
