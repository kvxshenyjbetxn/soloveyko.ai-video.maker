import React, { useState, useEffect, useMemo, useCallback } from 'react';
import { useI18n } from '../contexts/I18nContext';
import { GetGalleryImages } from '../../wailsjs/go/main/App';
import { EventsOn } from '../../wailsjs/runtime/runtime';
import { utils } from '../../wailsjs/go/models';
import { useQueue } from '../contexts/QueueContext';
import './gallery.css';

interface SelectedMedia {
    name: string;
    url: string;
    path: string;
    prompt?: string;
}

import { RegenerateModal } from '../components/RegenerateModal';


// Memoized Card Component
const GalleryCard = React.memo(({ img, isSelected, isSelectionMode, onCardClick, onSelectionToggle, onDelete, onRegenerate, isRegenerating }: any) => {
    const isVideo = img.url.toLowerCase().endsWith('.mp4');
    const [showPrompt, setShowPrompt] = useState(false);

    return (
        <div className={`gallery-card ${isSelected ? 'selected' : ''}`}
            onClick={() => onCardClick(img)}>
            <div className="media-container">
                {isVideo ? (
                    <video
                        src={`${img.url}?v=${Date.now()}`}
                        muted
                        loop
                        playsInline
                        onMouseEnter={e => e.currentTarget.play()}
                        onMouseLeave={e => {
                            e.currentTarget.pause();
                            e.currentTarget.currentTime = 0;
                        }}
                    />
                ) : (
                    <img src={`${img.url}?thumb=1&v=${Date.now()}`} alt={img.name} loading="lazy" />
                )}

                {isRegenerating && (
                    <div className="card-loading-spinner">
                        <div className="spinner-medium"></div>
                    </div>
                )}

                {showPrompt && img.prompt && (
                    <div
                        className="card-prompt-overlay animate-fade"
                        onClick={e => e.stopPropagation()}
                        onMouseDown={e => e.stopPropagation()}
                        onMouseUp={e => e.stopPropagation()}
                    >
                        <div className="prompt-header">
                            <span className="prompt-label">Prompt</span>
                            <div className="prompt-header-actions">
                                <button
                                    className="prompt-copy-btn"
                                    onClick={(e) => {
                                        e.stopPropagation();
                                        navigator.clipboard.writeText(img.prompt);
                                    }}
                                    title="Copy Prompt"
                                >
                                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5"><rect x="9" y="9" width="13" height="13" rx="2" ry="2" /><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" /></svg>
                                </button>
                                <button className="prompt-close-btn-mini" onClick={() => setShowPrompt(false)}>&times;</button>
                            </div>
                        </div>
                        <div className="prompt-content premium-scrollbar">
                            {img.prompt}
                        </div>
                    </div>
                )}

                <div className="media-overlay">
                    <div className="overlay-top-actions">
                        <button className="card-delete-btn" onClick={e => onDelete(img.path, e)}>
                            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2M10 11v6M14 11v6" /></svg>
                        </button>
                        {img.prompt && (
                            <button
                                className={`card-prompt-btn ${showPrompt ? 'active' : ''}`}
                                onClick={e => { e.stopPropagation(); setShowPrompt(!showPrompt); }}
                                title="Show Generation Prompt"
                            >
                                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10" /><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3" /><line x1="12" y1="17" x2="12.01" y2="17" /></svg>
                            </button>
                        )}
                        <button className="card-regenerate-btn" onClick={e => onRegenerate(img, e)} title="Regenerate">
                            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M21 2v6h-6" /><path d="M3 12a9 9 0 0 1 15-6.7L21 8" /><path d="M3 22v-6h6" /><path d="M21 12a9 9 0 0 1-15 6.7L3 16" /></svg>
                        </button>
                    </div>
                    <div className={`card-checkbox ${isSelected ? 'checked' : ''}`} onClick={e => onSelectionToggle(img.path, e)}>
                        {isSelected && <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3"><polyline points="20 6 9 17 4 12" /></svg>}
                    </div>
                </div>
            </div>
            <div className="media-info"><span className="media-name">{img.name}</span></div>
        </div>
    );
});

// Memoized Template Section
const TemplateSection = React.memo(({ tpl, taskName, isCollapsed, onToggle, isSelectionMode, selectedPaths, onCardClick, onSelectionToggle, onDelete, onRegenerate, regeneratingPaths }: any) => {
    return (
        <div className={`template-section ${isCollapsed ? 'is-collapsed' : ''}`}>
            <div className="template-header" onClick={onToggle}>
                <div className="template-name">
                    <svg className="section-icon-minor" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="3" y="3" width="7" height="7" /><rect x="14" y="3" width="7" height="7" /><rect x="14" y="14" width="7" height="7" /><rect x="3" y="14" width="7" height="7" /></svg>
                    {tpl.name}
                    <span className="section-count">{tpl.images?.length || 0}</span>
                </div>
                <svg className={`collapse-chevron-minor ${isCollapsed ? 'collapsed' : ''}`} width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M6 9l6 6 6-6" /></svg>
            </div>
            <div className="template-collapsible-wrapper">
                <div className="template-content gallery-grid">
                    {tpl.images.map((img: any, idx: number) => (
                        <GalleryCard
                            key={img.path || idx}
                            img={img}
                            isSelected={selectedPaths.has(img.path)}
                            isSelectionMode={isSelectionMode}
                            onCardClick={onCardClick}
                            onSelectionToggle={onSelectionToggle}
                            onDelete={onDelete}
                            onRegenerate={onRegenerate}
                            isRegenerating={regeneratingPaths?.has(img.path)}
                        />
                    ))}
                </div>
            </div>
        </div>
    );
});

export const Gallery = ({ setCurrentPath }: { setCurrentPath?: (path: any) => void }) => {
    const { t } = useI18n();
    const [tasks, setTasks] = useState<utils.GalleryTask[]>([]);
    const [selectedMedia, setSelectedMedia] = useState<SelectedMedia | null>(null);
    const [selectedPaths, setSelectedPaths] = useState<Set<string>>(new Set());
    const [collapsedTasks, setCollapsedTasks] = useState<Set<string>>(new Set());
    const [collapsedTemplates, setCollapsedTemplates] = useState<Set<string>>(new Set());
    const [loading, setLoading] = useState(true);
    const [isSelectionMode, setIsSelectionMode] = useState(false);

    // Regeneration modal state
    const [isRegModalOpen, setIsRegModalOpen] = useState(false);
    const [regMedia, setRegMedia] = useState<any>(null);
    const [isBulkReg, setIsBulkReg] = useState(false);

    const { tasks: queueTasks, resumeImageControl, regeneratingPaths, addRegeneratingPath, removeRegeneratingPath } = useQueue();
    const isAwaitingControl = useMemo(() => queueTasks.some(t => t.isAwaitingImageControl), [queueTasks]);

    const handleContinueProcessing = async () => {
        // Force hide gallery tab via event before switching
        // @ts-ignore
        if (window.runtime) {
            // @ts-ignore
            window.runtime.EventsEmit("hideGallery");
        }

        await resumeImageControl();
        if (setCurrentPath) {
            setCurrentPath('queue');
        }
    };

    const loadGallery = useCallback(async () => {
        try {
            setLoading(true);
            const data = await GetGalleryImages();
            setTasks(data || []);
        } catch (error) {
            console.error('Failed to load gallery:', error);
        } finally {
            setLoading(false);
        }
    }, []);

    useEffect(() => {
        loadGallery();

        const uStage = EventsOn('stageStatus', (id: string, stage: string, status: string) => {
            if (stage === 'image' && status === 'completed') {
                loadGallery();
            }
        });

        const uUpdate = EventsOn('galleryUpdate', () => {
            loadGallery();
        });

        return () => {
            uStage();
            uUpdate();
        };
    }, [loadGallery]);

    const toggleTask = useCallback((taskName: string) => {
        setCollapsedTasks(prev => {
            const newSet = new Set(prev);
            if (newSet.has(taskName)) newSet.delete(taskName);
            else newSet.add(taskName);
            return newSet;
        });
    }, []);

    const toggleTemplate = useCallback((taskName: string, templateName: string) => {
        const key = `${taskName}_${templateName}`;
        setCollapsedTemplates(prev => {
            const newSet = new Set(prev);
            if (newSet.has(key)) newSet.delete(key);
            else newSet.add(key);
            return newSet;
        });
    }, []);

    const toggleImageSelection = useCallback((path: string, e?: React.MouseEvent) => {
        if (e) e.stopPropagation();
        setSelectedPaths(prev => {
            const newSet = new Set(prev);
            if (newSet.has(path)) newSet.delete(path);
            else newSet.add(path);
            if (newSet.size === 0) setIsSelectionMode(false);
            else setIsSelectionMode(true);
            return newSet;
        });
    }, []);

    const handleDeleteImage = useCallback(async (path: string, e?: React.MouseEvent) => {
        if (e) e.stopPropagation();
        const app = (window as any).go.main.App;
        const success = await app.DeleteGalleryImage(path);
        if (success) {
            loadGallery();
            setSelectedMedia(prev => (prev?.path === path ? null : prev));
        }
    }, [loadGallery]);

    const handleBulkDelete = async () => {
        if (selectedPaths.size === 0) return;
        const app = (window as any).go.main.App;
        await app.DeleteGalleryImages(Array.from(selectedPaths));
        setSelectedPaths(new Set());
        setIsSelectionMode(false);
        loadGallery();
    };

    const clearSelection = useCallback(() => {
        setSelectedPaths(new Set());
        setIsSelectionMode(false);
    }, []);

    const flatImages = useMemo(() => {
        return tasks.reduce((acc: any[], task) => {
            task.templates?.forEach(tpl => {
                tpl.images?.forEach(img => {
                    acc.push({ ...img, taskName: task.name, templateName: tpl.name });
                });
            });
            return acc;
        }, []);
    }, [tasks]);

    const onCardClick = useCallback((img: any) => {
        if (isSelectionMode) {
            toggleImageSelection(img.path);
        } else {
            setSelectedMedia({ name: img.name, url: img.url, path: img.path, prompt: img.prompt });
        }
    }, [isSelectionMode, toggleImageSelection]);

    const handleOpenRegenerate = useCallback((img: any, e?: React.MouseEvent) => {
        if (e) e.stopPropagation();
        setRegMedia(img);
        setIsBulkReg(false);
        setIsRegModalOpen(true);
    }, []);

    const handleOpenBulkRegenerate = useCallback(() => {
        if (selectedPaths.size === 0) return;
        const firstPath = Array.from(selectedPaths)[0];
        const firstMedia = flatImages.find(img => img.path === firstPath);
        setRegMedia(firstMedia || null);
        setIsBulkReg(true);
        setIsRegModalOpen(true);
    }, [selectedPaths, flatImages]);

    const handleRegenerateAction = useCallback(async (prompt: string, service: string, settings: any) => {
        const pathsToRegenerate = isBulkReg ? Array.from(selectedPaths) : [regMedia?.path].filter(Boolean);
        if (pathsToRegenerate.length === 0) return;

        const promises = pathsToRegenerate.map(async (path) => {
            addRegeneratingPath(path);
            try {
                const app = (window as any).go.main.App;
                let finalPrompt = prompt;
                if (isBulkReg) {
                    const imgObj = flatImages.find(i => i.path === path);
                    finalPrompt = imgObj?.prompt || "";
                }
                await app.RegenerateGalleryImage(path, finalPrompt, service, settings);
            } catch (e) {
                console.error(`Regeneration failed for ${path}:`, e);
            } finally {
                removeRegeneratingPath(path);
            }
        });

        await Promise.all(promises);

        loadGallery();
        if (isBulkReg) {
            clearSelection();
        }
    }, [isBulkReg, regMedia, selectedPaths, flatImages, loadGallery, clearSelection, addRegeneratingPath, removeRegeneratingPath]);

    const handleKeyDown = useCallback((e: KeyboardEvent) => {
        if (!selectedMedia) return;
        if (e.key === 'Escape') { setSelectedMedia(null); return; }
        const currentIndex = flatImages.findIndex(img => img.url === selectedMedia.url);
        if (currentIndex === -1) return;

        if (e.key === 'ArrowRight' && currentIndex < flatImages.length - 1) {
            const next = flatImages[currentIndex + 1];
            setSelectedMedia({ name: next.name, url: next.url, path: next.path, prompt: next.prompt });
        } else if (e.key === 'ArrowLeft' && currentIndex > 0) {
            const prev = flatImages[currentIndex - 1];
            setSelectedMedia({ name: prev.name, url: prev.url, path: prev.path, prompt: prev.prompt });
        } else if (e.key === 'Delete' || e.key === 'Backspace') {
            handleDeleteImage(selectedMedia.path);
        }
    }, [selectedMedia, flatImages, handleDeleteImage]);

    useEffect(() => {
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [handleKeyDown]);

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

            {isAwaitingControl && (
                <div className="gallery-continue-bar">
                    <div className="continue-content">
                        <div className="continue-info">
                            <div className="pulse-icon"></div>
                            <span>{t('pipeline.image_control_notification.title')}</span>
                        </div>
                        <button className="btn-continue-processing" onClick={handleContinueProcessing}>
                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: '8px' }}>
                                <polyline points="9 10 4 15 9 20"></polyline>
                                <path d="M20 4v7a4 4 0 0 1-4 4H4"></path>
                            </svg>
                            {t('pipeline.continue_processing')}
                        </button>
                    </div>
                </div>
            )}

            {isSelectionMode && selectedPaths.size > 0 && (
                <div className="gallery-bulk-actions animate-slide-down">
                    <div className="bulk-info">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: '8px' }}><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" /><polyline points="22 4 12 14.01 9 11.01" /></svg>
                        <span>{t('gallery.selected') || 'Selected'}: <strong>{selectedPaths.size}</strong></span>
                    </div>
                    <div className="bulk-buttons">
                        <button className="bulk-btn-regenerate" onClick={handleOpenBulkRegenerate}>
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: '6px' }}><path d="M23 4v6h-6M1 20v-6h6M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" /></svg>
                            {t('gallery.regenerate_modal.generate')}
                        </button>
                        <button className="bulk-btn-cancel" onClick={clearSelection}>{t('common.cancel')}</button>
                        <button className="bulk-btn-delete" onClick={handleBulkDelete}>
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: '6px' }}><path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2M10 11v6M14 11v6" /></svg>
                            {t('common.delete')}
                        </button>
                    </div>
                </div>
            )}

            <div className={`gallery-scroll-container premium-scrollbar ${isSelectionMode ? 'with-bulk' : ''}`}>
                {loading && tasks.length === 0 ? (
                    <div className="gallery-empty"><p>{t('common.loading')}</p></div>
                ) : tasks.length === 0 ? (
                    <div className="gallery-empty"><p>{t('gallery.empty')}</p></div>
                ) : (
                    <div className="gallery-tasks">
                        {tasks.map((task, tIndex) => (
                            <div key={task.name || tIndex} className={`task-section ${collapsedTasks.has(task.name) ? 'is-collapsed' : ''}`}>
                                <div className="task-header" onClick={() => toggleTask(task.name)}>
                                    <div className="task-name">
                                        <svg className="section-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" /></svg>
                                        {task.name}
                                        <span className="section-count">{task.templates?.reduce((sum, tpl) => sum + (tpl.images?.length || 0), 0) || 0}</span>
                                    </div>
                                    <svg className={`collapse-chevron ${collapsedTasks.has(task.name) ? 'collapsed' : ''}`} width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M6 9l6 6 6-6" /></svg>
                                </div>
                                <div className="task-collapsible-wrapper">
                                    <div className="task-content">
                                        {task.templates.map((tpl, tmpIndex) => (
                                            <TemplateSection
                                                key={`${task.name}_${tpl.name}` || tmpIndex}
                                                tpl={tpl}
                                                taskName={task.name}
                                                isCollapsed={collapsedTemplates.has(`${task.name}_${tpl.name}`)}
                                                onToggle={() => toggleTemplate(task.name, tpl.name)}
                                                isSelectionMode={isSelectionMode}
                                                selectedPaths={selectedPaths}
                                                onCardClick={onCardClick}
                                                onSelectionToggle={toggleImageSelection}
                                                onDelete={handleDeleteImage}
                                                onRegenerate={handleOpenRegenerate}
                                                regeneratingPaths={regeneratingPaths}
                                            />
                                        ))}
                                    </div>
                                </div>
                            </div>
                        ))}
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
                        <div className="modal-image-wrapper">
                            {selectedMedia.url.toLowerCase().endsWith('.mp4') ? (
                                <video src={selectedMedia.url} className="animate-fade" controls autoPlay loop playsInline onClick={e => e.stopPropagation()} />
                            ) : (
                                <img src={selectedMedia.url} alt={selectedMedia.name} className="animate-fade" onClick={e => e.stopPropagation()} />
                            )}
                        </div>
                    </div>
                </div>
            )}

            <RegenerateModal
                isOpen={isRegModalOpen}
                onClose={() => setIsRegModalOpen(false)}
                onConfirm={handleRegenerateAction}
                imagePath={regMedia?.path}
                initialPrompt={regMedia?.prompt}
                isBulk={isBulkReg}
            />
        </div>
    );
};
