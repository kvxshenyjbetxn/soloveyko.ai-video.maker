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
    audioPath: string | null;
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
    const [zoom, setZoom] = useState<number>(100); // pixels per second (default 100)
    const [currentTime, setCurrentTime] = useState<number>(0);
    const [isScrubbing, setIsScrubbing] = useState<boolean>(false);
    const [isPlaying, setIsPlaying] = useState<boolean>(false);
    const [volume, setVolume] = useState<number>(0.8);

    const timelineRef = useRef<HTMLDivElement>(null);
    const previewVideoRef = useRef<HTMLVideoElement>(null);
    const previewAudioRef = useRef<HTMLAudioElement>(null);
    const containerRef = useRef<HTMLDivElement>(null);
    const requestRef = useRef<number>();

    const [draggingIdx, setDraggingIdx] = useState<number | null>(null);
    const [startX, setStartX] = useState<number>(0);
    const [startDurations, setStartDurations] = useState<{ current: number, next: number }>({ current: 0, next: 0 });
    const [timelineHeight, setTimelineHeight] = useState<number>(300);
    const isResizingRef = useRef<boolean>(false);

    useEffect(() => {
        if (task.montagePlanData) {
            try {
                const parsed = JSON.parse(task.montagePlanData);
                setPlan(parsed);
                setClips(parsed.clips.map((c: MontageClip) => ({ ...c })));
            } catch (e) {
                console.error("Failed to parse montage plan:", e);
            }
        }
    }, [task.montagePlanData]);

    const handleZoomIn = () => setZoom(prev => Math.min(prev + 10, 200));
    const handleZoomOut = () => setZoom(prev => Math.max(prev - 10, 10));

    // Calculate layouts
    const clipLayouts = useMemo(() => {
        if (!plan) return [];
        let currentStart = 0;
        return clips.map((clip, idx) => {
            const width = clip.duration * zoom;
            const x = currentStart * zoom;

            if (!plan.isFadeFast) {
                currentStart += (clip.duration - plan.transDuration);
            } else {
                currentStart += clip.duration;
            }

            if (!plan.isFadeFast && idx === clips.length - 1) {
                currentStart += plan.transDuration;
            }

            return { clip, idx, width, x };
        });
    }, [clips, zoom, plan]);

    const actualVideoDuration = useMemo(() => {
        if (clipLayouts.length === 0) return 0;
        const last = clipLayouts[clipLayouts.length - 1];
        return (last.x + last.width) / zoom;
    }, [clipLayouts, zoom]);

    const totalTimelineDuration = useMemo(() => {
        return Math.max(actualVideoDuration, plan?.audioDuration || 0);
    }, [actualVideoDuration, plan?.audioDuration]);

    // Preview Logic
    const activeClipInfo = useMemo(() => {
        // Find the clip at currentTime. If multiple (overlap), pick the one that started most recently or first one.
        // Let's pick the one where currentTime is between its x and x+width.
        for (let i = clipLayouts.length - 1; i >= 0; i--) {
            const layout = clipLayouts[i];
            const startTime = layout.x / zoom;
            const endTime = (layout.x + layout.width) / zoom;
            if (currentTime >= startTime && currentTime <= endTime) {
                return { ...layout, timeInClip: currentTime - startTime };
            }
        }
        return null;
    }, [clipLayouts, currentTime, zoom]);

    // Sync video currentTime
    useEffect(() => {
        if (previewVideoRef.current && activeClipInfo?.clip.isVideo) {
            previewVideoRef.current.currentTime = activeClipInfo.timeInClip;
        }
    }, [activeClipInfo, currentTime]);

    // Drag to resize logic
    const handleMouseDown = (e: React.MouseEvent, idx: number) => {
        e.stopPropagation();
        if (idx >= clips.length - 1) return;
        setDraggingIdx(idx);
        setStartX(e.clientX);
        setStartDurations({
            current: clips[idx].duration,
            next: clips[idx + 1].duration
        });
    };

    const lastTimeRef = useRef<number>(0);

    // Playback loop
    const animate = useCallback((time: number) => {
        if (!isPlaying || !plan) {
            lastTimeRef.current = 0;
            return;
        }

        if (lastTimeRef.current === 0) {
            lastTimeRef.current = time;
            requestRef.current = requestAnimationFrame(animate);
            return;
        }

        const delta = (time - lastTimeRef.current) / 1000;
        lastTimeRef.current = time;

        setCurrentTime(prev => {
            // If audio is playing, use it as the source of truth
            if (previewAudioRef.current && !previewAudioRef.current.paused) {
                const audioTime = previewAudioRef.current.currentTime;
                if (audioTime >= totalTimelineDuration) {
                    setIsPlaying(false);
                    return totalTimelineDuration;
                }
                return audioTime;
            }

            // Fallback to manual delta if audio is missing or loading
            const next = prev + delta;
            if (next >= totalTimelineDuration) {
                setIsPlaying(false);
                return totalTimelineDuration;
            }
            return next;
        });

        requestRef.current = requestAnimationFrame(animate);
    }, [isPlaying, totalTimelineDuration, plan]);

    useEffect(() => {
        if (isPlaying) {
            lastTimeRef.current = 0;
            requestRef.current = requestAnimationFrame(animate);
            if (previewAudioRef.current) {
                previewAudioRef.current.currentTime = currentTime;
                previewAudioRef.current.play().catch(e => console.error("Audio play failed:", e));
            }
            if (previewVideoRef.current && activeClipInfo?.clip.isVideo) {
                previewVideoRef.current.play().catch(e => console.error("Video play failed:", e));
            }
        } else {
            if (requestRef.current) cancelAnimationFrame(requestRef.current);
            if (previewAudioRef.current) previewAudioRef.current.pause();
            if (previewVideoRef.current) previewVideoRef.current.pause();
        }
    }, [isPlaying, animate]);

    // Sync audio/video with currentTime when NOT playing (scrubbing/manual move)
    useEffect(() => {
        if (!isPlaying) {
            if (previewAudioRef.current) {
                previewAudioRef.current.currentTime = currentTime;
                previewAudioRef.current.muted = false;
                previewAudioRef.current.volume = volume;
            }
            if (previewVideoRef.current && activeClipInfo?.clip.isVideo) {
                previewVideoRef.current.currentTime = activeClipInfo.timeInClip;
                previewVideoRef.current.muted = false;
                previewVideoRef.current.volume = volume;
            }
        }
    }, [currentTime, isPlaying, activeClipInfo, volume]);

    // Apply volume globally
    useEffect(() => {
        if (previewAudioRef.current) previewAudioRef.current.volume = volume;
        if (previewVideoRef.current) previewVideoRef.current.volume = volume;
    }, [volume]);

    const handleTogglePlay = () => setIsPlaying(!isPlaying);

    const handleTimelineMouseDown = (e: React.MouseEvent) => {
        if (!containerRef.current) return;
        setIsScrubbing(true);
        handleTimelineMove(e);
    };

    const handleTimelineMove = useCallback((e: React.MouseEvent | MouseEvent) => {
        if (!containerRef.current) return;
        const rect = containerRef.current.getBoundingClientRect();
        const scrollLeft = containerRef.current.scrollLeft;
        const x = e.clientX - rect.left + scrollLeft - 20; // 20 is padding
        const time = Math.max(0, Math.min(totalTimelineDuration, x / zoom));
        setCurrentTime(time);
    }, [zoom, totalTimelineDuration]);

    const handleResizeMouseMove = useCallback((e: MouseEvent) => {
        if (!isResizingRef.current) return;
        const windowHeight = window.innerHeight;
        // Calculate bottom offset for the timeline
        const newHeight = Math.max(150, Math.min(600, windowHeight - e.clientY - 120));
        setTimelineHeight(newHeight);
    }, []);

    const handleResizeMouseUp = useCallback(() => {
        isResizingRef.current = false;
        document.body.style.cursor = 'default';
        document.removeEventListener('mousemove', handleResizeMouseMove);
        document.removeEventListener('mouseup', handleResizeMouseUp);
    }, [handleResizeMouseMove]);

    const handleResizeMouseDown = (e: React.MouseEvent) => {
        e.preventDefault();
        isResizingRef.current = true;
        document.body.style.cursor = 'row-resize';
        document.addEventListener('mousemove', handleResizeMouseMove);
        document.addEventListener('mouseup', handleResizeMouseUp);
    };

    const handleMouseMoveGlobal = useCallback((e: MouseEvent) => {
        if (draggingIdx !== null) {
            if (draggingIdx >= clips.length - 1 || !plan) return;
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
        }

        if (isScrubbing) {
            handleTimelineMove(e);
        }
    }, [draggingIdx, startX, startDurations, zoom, plan, clips.length, isScrubbing, handleTimelineMove]);

    const handleMouseUpGlobal = useCallback(() => {
        setDraggingIdx(null);
        setIsScrubbing(false);
    }, []);

    useEffect(() => {
        if (draggingIdx !== null || isScrubbing) {
            document.addEventListener('mousemove', handleMouseMoveGlobal);
            document.addEventListener('mouseup', handleMouseUpGlobal);
        } else {
            document.removeEventListener('mousemove', handleMouseMoveGlobal);
            document.removeEventListener('mouseup', handleMouseUpGlobal);
        }
        return () => {
            document.removeEventListener('mousemove', handleMouseMoveGlobal);
            document.removeEventListener('mouseup', handleMouseUpGlobal);
        };
    }, [draggingIdx, isScrubbing, handleMouseMoveGlobal, handleMouseUpGlobal]);

    const handleConfirm = () => {
        const durs = clips.map(c => c.duration.toFixed(3)).join(',');
        onConfirm(task.id, `confirm:${durs}`);
    };

    const handleCancel = () => {
        onCancel(task.id);
    };

    const getResourceUrl = (path: string) => {
        if (!path) return '';
        const cleanPath = path.replace(/\\/g, '/');
        // If path is already absolute (contains drive colon), 
        // prepend 'local/' as the fileloader expects
        return `local/${cleanPath}`;
    };

    if (!plan) return null;

    const clipElements = clipLayouts.map(({ clip, idx, width, x }) => {
        const isVideo = clip.isVideo;
        const name = clip.path.split('\\').pop()?.split('/').pop() || '';

        return (
            <div
                key={idx}
                className={`montage-clip-block ${isVideo ? 'video' : 'image'} ${draggingIdx === idx ? 'dragging' : ''} ${activeClipInfo?.idx === idx ? 'active-preview' : ''}`}
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
    const numMarkers = Math.ceil(totalTimelineDuration);
    const markers = [];
    for (let i = 0; i <= numMarkers; i++) {
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
                    <div className="playback-overlay-controls">
                        <div className="playback-controls">
                            <button className="play-btn" onClick={handleTogglePlay} title={isPlaying ? "Pause" : "Play"}>
                                {isPlaying ? (
                                    <svg viewBox="0 0 24 24" width="28" height="28" fill="currentColor"><path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" /></svg>
                                ) : (
                                    <svg viewBox="0 0 24 24" width="28" height="28" fill="currentColor"><path d="M8 5v14l11-7z" /></svg>
                                )}
                            </button>
                            <div className="volume-control">
                                <svg viewBox="0 0 24 24" width="24" height="24" fill="currentColor"><path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z" /></svg>
                                <input
                                    type="range"
                                    min="0" max="1" step="0.01"
                                    value={volume}
                                    onChange={(e) => setVolume(parseFloat(e.target.value))}
                                />
                                <span className="volume-label">{Math.round(volume * 100)}%</span>
                            </div>
                        </div>
                    </div>

                    <div className="montage-preview-and-info">
                        <div className="montage-preview-container">
                            {activeClipInfo ? (
                                <div className="montage-preview-wrap">
                                    {activeClipInfo.clip.isVideo ? (
                                        <video
                                            ref={previewVideoRef}
                                            src={getResourceUrl(activeClipInfo.clip.path)}
                                            playsInline
                                        />
                                    ) : (
                                        <img src={getResourceUrl(activeClipInfo.clip.path)} alt="preview" />
                                    )}
                                    <div className="preview-timestamp">{currentTime.toFixed(2)}s</div>
                                    {plan?.audioPath && (
                                        <audio
                                            ref={previewAudioRef}
                                            src={getResourceUrl(plan.audioPath)}
                                            style={{ display: 'none' }}
                                        />
                                    )}
                                </div>
                            ) : (
                                <div className="montage-preview-placeholder">
                                    {t('montage_editor.no_preview') || 'No preview at this time'}
                                </div>
                            )}
                        </div>

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
                        </div>
                    </div>

                    <div className="montage-timeline-resizer" onMouseDown={handleResizeMouseDown}>
                        <div className="resizer-handle-line"></div>
                    </div>

                    <div className="montage-timeline-container" ref={containerRef} onMouseDown={handleTimelineMouseDown} style={{ height: `${timelineHeight}px`, flex: 'none' }}>
                        <div className="montage-timeline-wrapper" style={{ width: `${Math.max(totalTimelineDuration * zoom + 100, containerRef.current?.clientWidth || 0)}px` }}>
                            <div className="montage-timeline-ruler">
                                {markers}
                                {/* Total Track Reference */}
                                <div className="audio-track-reference" style={{ width: `${plan.audioDuration * zoom}px` }}>
                                    <span>Audio Duration ({plan.audioDuration.toFixed(1)}s)</span>
                                </div>
                            </div>
                            <div className="montage-timeline-tracks" ref={timelineRef}>
                                <div className="montage-track">
                                    {clipElements}
                                </div>
                            </div>

                            {/* Playhead */}
                            <div
                                className="montage-playhead"
                                style={{ left: `${currentTime * zoom}px` }}
                            >
                                <div className="playhead-handle" />
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
