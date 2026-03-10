import React, { useState, useEffect, useRef, useMemo, useCallback } from 'react';
import { useI18n } from '../contexts/I18nContext';
import './MontageEditor.css';
import { QueueTask } from '../contexts/QueueContext';

interface MontageClip {
    path: string;
    duration: number;
    isVideo: boolean;
}

interface MontagePlan {
    audioDuration: number;
    transDuration: number;
    isFadeFast: boolean;
    clips: MontageClip[];
}

interface MontageEditorProps {
    task: QueueTask;
    onConfirm: (taskId: string, resultData: string) => void;
    onCancel: (taskId: string) => void;
}

export const MontageEditor: React.FC<MontageEditorProps> = ({ task, onConfirm, onCancel }) => {
    const { t } = useI18n();
    const [plan, setPlan] = useState<MontagePlan | null>(null);
    const [clips, setClips] = useState<MontageClip[]>([]);
    const [zoom, setZoom] = useState<number>(50); // pixels per second
    const timelineRef = useRef<HTMLDivElement>(null);
    const [draggingIdx, setDraggingIdx] = useState<number | null>(null);
    const [startX, setStartX] = useState<number>(0);
    const [startDurations, setStartDurations] = useState<{current: number, next: number}>({current: 0, next: 0});

    // Track scroll position for custom scroll sync if needed
    const containerRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (task.montagePlanData) {
            try {
                const parsed = JSON.parse(task.montagePlanData);
                setPlan(parsed);
                // Create a working copy of clips
                setClips(parsed.clips.map((c: MontageClip) => ({ ...c })));
            } catch (e) {
                console.error("Failed to parse montage plan:", e);
            }
        }
    }, [task.montagePlanData]);

    const handleZoomIn = () => setZoom(prev => Math.min(prev + 10, 200));
    const handleZoomOut = () => setZoom(prev => Math.max(prev - 10, 10));

    // Drag to resize logic
    const handleMouseDown = (e: React.MouseEvent, idx: number) => {
        e.stopPropagation();
        if (idx >= clips.length - 1) return; // Cannot drag the last clip's right edge
        setDraggingIdx(idx);
        setStartX(e.clientX);
        setStartDurations({
            current: clips[idx].duration,
            next: clips[idx + 1].duration
        });
    };

    const handleMouseMove = useCallback((e: MouseEvent) => {
        if (draggingIdx === null || draggingIdx >= clips.length - 1 || !plan) return;
        
        const deltaX = e.clientX - startX;
        const deltaDuration = deltaX / zoom;
        
        let newCurrent = startDurations.current + deltaDuration;
        let newNext = startDurations.next - deltaDuration;
        
        const MIN_DURATION = 0.5;
        
        if (newCurrent < MIN_DURATION) {
            newCurrent = MIN_DURATION;
            newNext = startDurations.current + startDurations.next - MIN_DURATION;
        } else if (newNext < MIN_DURATION) {
            newNext = MIN_DURATION;
            newCurrent = startDurations.current + startDurations.next - MIN_DURATION;
        }
        
        setClips(prev => {
            const newClips = [...prev];
            newClips[draggingIdx] = { ...newClips[draggingIdx], duration: newCurrent };
            newClips[draggingIdx + 1] = { ...newClips[draggingIdx + 1], duration: newNext };
            return newClips;
        });
    }, [draggingIdx, startX, startDurations, zoom, plan, clips.length]);

    const handleMouseUp = useCallback(() => {
        if (draggingIdx !== null) {
            setDraggingIdx(null);
        }
    }, [draggingIdx]);

    useEffect(() => {
        if (draggingIdx !== null) {
            document.addEventListener('mousemove', handleMouseMove);
            document.addEventListener('mouseup', handleMouseUp);
        } else {
            document.removeEventListener('mousemove', handleMouseMove);
            document.removeEventListener('mouseup', handleMouseUp);
        }
        return () => {
            document.removeEventListener('mousemove', handleMouseMove);
            document.removeEventListener('mouseup', handleMouseUp);
        };
    }, [draggingIdx, handleMouseMove, handleMouseUp]);

    const handleConfirm = () => {
        // Send format: "confirm:duration1,duration2,..."
        const durs = clips.map(c => c.duration.toFixed(3)).join(',');
        onConfirm(task.id, `confirm:${durs}`);
    };

    const handleCancel = () => {
        onCancel(task.id);
    };

    if (!plan) return null;

    // Calculate exact positions considering overlap
    let currentStart = 0;
    const clipLayouts = clips.map((clip, idx) => {
        const width = clip.duration * zoom;
        const x = currentStart * zoom;
        
        // Next clip starts before this one ends if xfade is used
        if (!plan.isFadeFast) {
            currentStart += (clip.duration - plan.transDuration);
        } else {
            currentStart += clip.duration;
        }

        // For the last clip, we don't subtract transition duration for the next one
        if (!plan.isFadeFast && idx === clips.length - 1) {
            currentStart += plan.transDuration; // Add it back since there's no next clip
        }

        return { clip, idx, width, x };
    });

    // Actual visual end time of the video
    const actualVideoDuration = clipLayouts.length > 0 
        ? (clipLayouts[clipLayouts.length - 1].x + clipLayouts[clipLayouts.length - 1].width) / zoom 
        : 0;

    const clipElements = clipLayouts.map(({ clip, idx, width, x }) => {
        const isVideo = clip.isVideo;
        const name = clip.path.split('\\').pop()?.split('/').pop() || '';
        
        return (
            <div 
                key={idx} 
                className={`montage-clip-block ${isVideo ? 'video' : 'image'} ${draggingIdx === idx ? 'dragging' : ''}`}
                style={{ 
                    left: `${x}px`, 
                    width: `${width}px`,
                    zIndex: draggingIdx === idx ? 10 : 1
                }}
            >
                <div className="montage-clip-content">
                    <div className="montage-clip-thumbnail-placeholder">
                        {isVideo ? '🎬' : '🖼️'}
                    </div>
                    <span className="montage-clip-name" title={name}>{name}</span>
                    <span className="montage-clip-duration">{clip.duration.toFixed(1)}s</span>
                </div>
                
                {/* Right Edge Resizer */}
                {idx < clips.length - 1 && (
                    <div 
                        className="montage-clip-resizer right" 
                        onMouseDown={(e) => handleMouseDown(e, idx)}
                    />
                )}
            </div>
        );
    });

    // Time markers
    const numMarkers = Math.ceil(Math.max(actualVideoDuration, plan.audioDuration));
    const markers = [];
    for (let i = 0; i <= numMarkers; i++) {
        // Show marker every 5 seconds or 1 second depending on zoom
        if (zoom > 50 || i % 5 === 0) {
            markers.push(
                <div key={i} className="timeline-marker" style={{ left: `${i * zoom}px` }}>
                    <span>{i}s</span>
                </div>
            );
        } else {
            markers.push(
                <div key={i} className="timeline-marker minor" style={{ left: `${i * zoom}px` }} />
            );
        }
    }

    return (
        <div className="montage-editor-overlay animate-fade">
            <div className="montage-editor-window">
                <div className="montage-editor-header">
                    <div className="montage-editor-title">
                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><line x1="9" y1="3" x2="9" y2="21"></line></svg>
                        {t('pipeline.montage_control') || 'Montage Editor'} - {task.name}
                    </div>
                    <div className="montage-editor-controls">
                        <button className="montage-btn icon" onClick={handleZoomOut} title="Zoom Out">-</button>
                        <span className="montage-zoom-label">{zoom}%</span>
                        <button className="montage-btn icon" onClick={handleZoomIn} title="Zoom In">+</button>
                    </div>
                </div>

                <div className="montage-editor-body">
                    <div className="montage-info-panel">
                        <div className="info-item">
                            <span className="info-label">Audio:</span>
                            <span className="info-value">{plan.audioDuration.toFixed(1)}s</span>
                        </div>
                        <div className="info-item">
                            <span className="info-label">Video Track:</span>
                            <span className="info-value">{actualVideoDuration.toFixed(1)}s</span>
                        </div>
                        <div className="info-item">
                            <span className="info-label">Transition:</span>
                            <span className="info-value">{plan.transDuration}s ({plan.isFadeFast ? 'Fade' : 'XFade'})</span>
                        </div>
                        <div className="info-message">
                            * {t('montage_editor.drag_hint') || 'Drag the right edge of a block to change its duration.'}
                        </div>
                    </div>

                    <div className="montage-timeline-container" ref={containerRef}>
                        <div className="montage-timeline-wrapper" style={{ width: `${Math.max(actualVideoDuration * zoom + 100, containerRef.current?.clientWidth || 0)}px` }}>
                            <div className="montage-timeline-ruler">
                                {markers}
                                {/* Total Track Reference */}
                                <div className="audio-track-reference" style={{ width: `${actualVideoDuration * zoom}px`, background: '#3498db', opacity: 0.6 }}>
                                    <span style={{ color: '#3498db' }}>Total Duration ({actualVideoDuration.toFixed(1)}s)</span>
                                </div>
                            </div>
                            <div className="montage-timeline-tracks" ref={timelineRef}>
                                <div className="montage-track">
                                    {clipElements}
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <div className="montage-editor-footer">
                    <button className="montage-btn secondary" onClick={handleCancel}>{t('common.cancel')}</button>
                    <button className="montage-btn primary" onClick={handleConfirm}>{t('common.save')} & {t('queue.start')}</button>
                </div>
            </div>
        </div>
    );
};
