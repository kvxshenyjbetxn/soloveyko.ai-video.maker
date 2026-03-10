import React, { useState, useEffect, useRef, useMemo, useCallback } from 'react';
import { useI18n } from '../contexts/I18nContext';
import './MontageEditor.css';
import { QueueTask } from '../contexts/QueueContext';

interface MontageClip {
    path: string;
    duration: number;
    isVideo: boolean;
}

interface MontageSegment {
    start: number;
    end: number;
}

interface MontagePlan {
    audioDuration: number;
    audioPath: string | null;
    transDuration: number;
    isFadeFast: boolean;
    clips: MontageClip[];
    subtitlePath?: string;
    audioSegments?: MontageSegment[];
}

interface SubtitleEntry {
    start: number;
    end: number;
    text: string;
}

interface MontageEditorProps {
    task: QueueTask;
    onConfirm: (taskId: string, resultData: string) => void;
    onCancel: (taskId: string) => void;
}

interface AnimationState {
    currentTime: number;
    selection: { start: number | null, end: number | null };
    isPlaying: boolean;
    audioSegments: MontageSegment[];
    clips: MontageClip[];
    zoom: number;
    totalDuration: number;
}

export const MontageEditor: React.FC<MontageEditorProps> = ({ task, onConfirm, onCancel }) => {
    const { t } = useI18n();
    const [plan, setPlan] = useState<MontagePlan | null>(null);
    const [clips, setClips] = useState<MontageClip[]>([]);
    const [zoom, setZoom] = useState<number>(100); 
    const [currentTime, setCurrentTime] = useState<number>(0);
    const [isScrubbing, setIsScrubbing] = useState<boolean>(false);
    const [isPlaying, setIsPlaying] = useState<boolean>(false);
    const [volume, setVolume] = useState<number>(0.8);
    const [selection, setSelection] = useState<{ start: number | null, end: number | null }>({ start: null, end: null });
    const [audioSegments, setAudioSegments] = useState<MontageSegment[]>([]);
    const [isCuttingMode, setIsCuttingMode] = useState<boolean>(false);
    const [draggingSelectionSide, setDraggingSelectionSide] = useState<null | 'start' | 'end'>(null);
    const [cutJunctions, setCutJunctions] = useState<{ position: number, durationRemoved: number }[]>([]);
    const [subtitles, setSubtitles] = useState<SubtitleEntry[]>([]);

    const previewVideoRef = useRef<HTMLVideoElement>(null);
    const previewAudioRef = useRef<HTMLAudioElement>(null);
    const containerRef = useRef<HTMLDivElement>(null);
    const requestRef = useRef<number>();
    const lastTimeRef = useRef<number>(0);

    const [draggingIdx, setDraggingIdx] = useState<number | null>(null);
    const [startX, setStartX] = useState<number>(0);
    const [startDurations, setStartDurations] = useState<{ current: number, next: number }>({ current: 0, next: 0 });
    const [timelineHeight, setTimelineHeight] = useState<number>(300);
    const isResizingRef = useRef<boolean>(false);

    // Initial Load
    useEffect(() => {
        if (task.montagePlanData) {
            try {
                const parsed = JSON.parse(task.montagePlanData);
                setPlan(parsed);
                setClips(parsed.clips.map((c: MontageClip) => ({ ...c })));
                if (parsed.audioSegments && parsed.audioSegments.length > 0) {
                    setAudioSegments(parsed.audioSegments);
                } else {
                    setAudioSegments([{ start: 0, end: parsed.audioDuration }]);
                }
            } catch (e) {
                console.error("Failed to parse montage plan:", e);
            }
        }
    }, [task.montagePlanData]);

    // Subtitle Fetching & Parsing
    useEffect(() => {
        if (!plan?.subtitlePath) return;

        const parseTime = (timeStr: string) => {
            const parts = timeStr.trim().replace(',', '.').split(':');
            if (parts.length !== 3) return 0;
            const h = parseFloat(parts[0]);
            const m = parseFloat(parts[1]);
            const s = parseFloat(parts[2]);
            return h * 3600 + m * 60 + s;
        };

        const fetchSubtitles = async () => {
            try {
                const response = await fetch(getUrl(plan.subtitlePath!));
                const text = await response.text();
                const entries: SubtitleEntry[] = [];
                const blockRegex = /(\d+)\r?\n(\d{2}:\d{2}:\d{2}[,\.]\d{3}) --> (\d{2}:\d{2}:\d{2}[,\.]\d{3})\r?\n([\s\S]*?)(?=\r?\n\r?\n|\r?\n?$)/g;
                
                let match;
                while ((match = blockRegex.exec(text)) !== null) {
                    entries.push({
                        start: parseTime(match[2]),
                        end: parseTime(match[3]),
                        text: match[4].trim().replace(/\r?\n/g, ' ')
                    });
                }
                setSubtitles(entries);
            } catch (err) {
                console.error("Failed to load subtitles:", err);
            }
        };

        fetchSubtitles();
    }, [plan?.subtitlePath]);

    // Layout Calculations
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
        const audioTotal = audioSegments.reduce((sum, seg) => sum + (seg.end - seg.start), 0);
        return Math.max(actualVideoDuration, audioTotal);
    }, [actualVideoDuration, audioSegments]);

    const activeClipInfo = useMemo(() => {
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

    // Animation Ref for high-performance access
    const animStateRef = useRef<AnimationState>({
        currentTime, selection, isPlaying, audioSegments, clips, zoom, totalDuration: totalTimelineDuration
    });
    animStateRef.current = { currentTime, selection, isPlaying, audioSegments, clips, zoom, totalDuration: totalTimelineDuration };

    const getOriginalTime = useCallback((timelineTime: number) => {
        const segs = animStateRef.current.audioSegments;
        if (segs.length === 0) return timelineTime;
        let current = 0;
        for (const seg of segs) {
            const segDur = seg.end - seg.start;
            if (timelineTime <= current + segDur + 0.001) {
                return seg.start + (timelineTime - current);
            }
            current += segDur;
        }
        return segs[segs.length - 1]?.end || timelineTime;
    }, []);

    const currentSubtitle = useMemo(() => {
        const origTime = getOriginalTime(currentTime);
        return subtitles.find(s => origTime >= s.start && origTime <= s.end);
    }, [subtitles, currentTime, getOriginalTime]);

    const animate = useCallback((time: number) => {
        if (lastTimeRef.current === 0) {
            lastTimeRef.current = time;
            requestRef.current = requestAnimationFrame(animate);
            return;
        }
        const delta = (time - lastTimeRef.current) / 1000;
        lastTimeRef.current = time;

        setCurrentTime(prev => {
            const next = prev + delta;
            if (previewAudioRef.current) {
                const targetOrig = getOriginalTime(next);
                if (Math.abs(previewAudioRef.current.currentTime - targetOrig) > 0.15) {
                    previewAudioRef.current.currentTime = targetOrig;
                }
            }
            if (next >= animStateRef.current.totalDuration) {
                setIsPlaying(false);
                return animStateRef.current.totalDuration;
            }
            return next;
        });
        requestRef.current = requestAnimationFrame(animate);
    }, [getOriginalTime]);

    useEffect(() => {
        if (isPlaying) {
            lastTimeRef.current = 0;
            requestRef.current = requestAnimationFrame(animate);
            if (previewAudioRef.current) {
                previewAudioRef.current.currentTime = getOriginalTime(currentTime);
                previewAudioRef.current.play().catch(() => {});
            }
            if (previewVideoRef.current && activeClipInfo?.clip.isVideo) {
                previewVideoRef.current.play().catch(() => {});
            }
        } else {
            if (requestRef.current) cancelAnimationFrame(requestRef.current);
            if (previewAudioRef.current) previewAudioRef.current.pause();
            if (previewVideoRef.current) previewVideoRef.current.pause();
        }
        return () => { if (requestRef.current) cancelAnimationFrame(requestRef.current); };
    }, [isPlaying, animate]);

    useEffect(() => {
        if (!isPlaying) {
            if (previewAudioRef.current) previewAudioRef.current.currentTime = getOriginalTime(currentTime);
            if (previewVideoRef.current && activeClipInfo?.clip.isVideo) {
                previewVideoRef.current.currentTime = activeClipInfo.timeInClip;
            }
        }
    }, [currentTime, isPlaying, activeClipInfo?.timeInClip, getOriginalTime]);

    useEffect(() => {
        if (previewAudioRef.current) previewAudioRef.current.volume = volume;
        if (previewVideoRef.current) previewVideoRef.current.volume = volume;
    }, [volume]);

    // Handlers
    const handleTogglePlay = useCallback(() => setIsPlaying(p => !p), []);
    const handleMarkIn = useCallback(() => setSelection(p => ({ ...p, start: currentTime })), [currentTime]);
    const handleMarkOut = useCallback(() => setSelection(p => ({ ...p, end: currentTime })), [currentTime]);
    const handleClearSelection = useCallback(() => setSelection({ start: null, end: null }), []);

    const handleTimelineMove = useCallback((e: React.MouseEvent | MouseEvent) => {
        if (!containerRef.current) return;
        const rect = containerRef.current.getBoundingClientRect();
        const x = e.clientX - rect.left + containerRef.current.scrollLeft - 20;
        setCurrentTime(Math.max(0, Math.min(animStateRef.current.totalDuration, x / animStateRef.current.zoom)));
    }, []);

    const handleCutSelection = useCallback(() => {
        const { start: sBound, end: eBound } = animStateRef.current.selection;
        if (sBound === null || eBound === null) return;
        const start = Math.min(sBound, eBound);
        const end = Math.max(sBound, eBound);
        const durationDelta = end - start;
        
        setIsPlaying(false);
        const origStart = getOriginalTime(start);
        const origEnd = getOriginalTime(end);

        setAudioSegments(prev => {
            const next: MontageSegment[] = [];
            for (const seg of prev) {
                if (seg.end <= origStart + 0.005) next.push(seg);
                else if (seg.start >= origEnd - 0.005) next.push(seg);
                else {
                    if (seg.start < origStart - 0.005) next.push({ start: seg.start, end: origStart });
                    if (seg.end > origEnd + 0.005) next.push({ start: origEnd, end: seg.end });
                }
            }
            return next;
        });

        setClips(prev => prev.map((c, i) => {
            const l = clipLayouts[i];
            if (!l) return c;
            const overlap = Math.max(0, Math.min(end, (l.x + l.width) / zoom) - Math.max(start, l.x / zoom));
            return { ...c, duration: Math.max(0.1, c.duration - overlap) };
        }));

        // Track the junction and shift existing ones
        setCutJunctions(prev => {
            const filtered = (prev as any[]).filter(j => (j.position || j) < start || (j.position || j) > end);
            const shifted = filtered.map(j => {
                const pos = j.position || j;
                const dur = j.durationRemoved || 0;
                return {
                    position: pos > end ? pos - durationDelta : pos,
                    durationRemoved: dur
                };
            });
            return [...shifted, { position: start, durationRemoved: durationDelta }].sort((a, b) => a.position - b.position);
        });

        setSelection({ start: null, end: null });
        setCurrentTime(start);
    }, [getOriginalTime, clipLayouts, zoom]);

    // Keyboard shortcuts
    const actionRef = useRef({ handleCutSelection, handleMarkIn, handleMarkOut, handleTogglePlay });
    actionRef.current = { handleCutSelection, handleMarkIn, handleMarkOut, handleTogglePlay };

    useEffect(() => {
        const kd = (e: KeyboardEvent) => {
            if (e.target instanceof HTMLInputElement) return;
            if (e.key === '[') actionRef.current.handleMarkIn();
            if (e.key === ']') actionRef.current.handleMarkOut();
            if (e.key === 'Backspace' || e.key === 'Delete') {
                if (animStateRef.current.selection.start !== null) actionRef.current.handleCutSelection();
            }
            if (e.key === ' ') { e.preventDefault(); actionRef.current.handleTogglePlay(); }
        };
        window.addEventListener('keydown', kd);
        return () => window.removeEventListener('keydown', kd);
    }, []);

    const handleResizeMouseDown = (e: React.MouseEvent) => {
        e.preventDefault();
        const mm = (me: MouseEvent) => setTimelineHeight(Math.max(150, Math.min(600, window.innerHeight - me.clientY - 120)));
        const mu = () => { document.removeEventListener('mousemove', mm); document.removeEventListener('mouseup', mu); };
        document.addEventListener('mousemove', mm);
        document.addEventListener('mouseup', mu);
    };

    const handleMouseDownGlobal = (e: React.MouseEvent) => {
        setIsScrubbing(true);
        handleTimelineMove(e);
    };

    useEffect(() => {
        const mu = () => { 
            setDraggingIdx(null); 
            setIsScrubbing(false); 
            setDraggingSelectionSide(null);
        };
        const mm = (e: MouseEvent) => {
            if (draggingIdx !== null) {
                const deltaD = (e.clientX - startX) / zoom;
                let cD = startDurations.current + deltaD;
                let nD = startDurations.next - deltaD;
                if (cD < 0.5) { cD = 0.5; nD = startDurations.current + startDurations.next - 0.5; }
                else if (nD < 0.5) { nD = 0.5; cD = startDurations.current + startDurations.next - 0.5; }
                setClips(p => { const nc = [...p]; nc[draggingIdx] = { ...nc[draggingIdx], duration: cD }; nc[draggingIdx + 1] = { ...nc[draggingIdx + 1], duration: nD }; return nc; });
            } else if (draggingSelectionSide !== null && containerRef.current) {
                const rect = containerRef.current.getBoundingClientRect();
                const x = e.clientX - rect.left + containerRef.current.scrollLeft - 20;
                const newTime = Math.max(0, Math.min(animStateRef.current.totalDuration, x / zoom));
                setSelection(prev => ({ ...prev, [draggingSelectionSide]: newTime }));
            }
            if (isScrubbing) handleTimelineMove(e);
        };
        if (draggingIdx !== null || isScrubbing || draggingSelectionSide !== null) {
            document.addEventListener('mousemove', mm);
            document.addEventListener('mouseup', mu);
        }
        return () => { document.removeEventListener('mousemove', mm); document.removeEventListener('mouseup', mu); };
    }, [draggingIdx, isScrubbing, draggingSelectionSide, handleTimelineMove, zoom, startX, startDurations]);

    const markers = useMemo(() => {
        const res = [];
        const count = Math.ceil(totalTimelineDuration);
        for (let i = 0; i <= count; i++) {
            if (zoom > 50 || i % 5 === 0) res.push(<div key={i} className="timeline-marker" style={{ left: `${i * zoom}px` }}><span>{i}s</span></div>);
            else res.push(<div key={i} className="timeline-marker minor" style={{ left: `${i * zoom}px` }} />);
        }
        return res;
    }, [totalTimelineDuration, zoom]);

    const clipElements = useMemo(() => {
        return clipLayouts.map(({ clip, idx, width, x }) => (
            <div key={idx} className={`montage-clip-block ${clip.isVideo ? 'video' : 'image'} ${activeClipInfo?.idx === idx ? 'active-preview' : ''}`} style={{ left: `${x}px`, width: `${width}px` }}>
                <div className="montage-clip-content">
                    <div className="montage-clip-thumbnail-placeholder">{clip.isVideo ? '🎬' : '🖼️'}</div>
                    <span className="montage-clip-name">{clip.path.split(/[\\/]/).pop()}</span>
                    <span className="montage-clip-duration">{clip.duration.toFixed(1)}s</span>
                </div>
                {idx < clips.length - 1 && (
                    <div className="montage-clip-resizer right" onMouseDown={(e) => { e.stopPropagation(); setDraggingIdx(idx); setStartX(e.clientX); setStartDurations({ current: clips[idx].duration, next: clips[idx + 1].duration }); }} />
                )}
            </div>
        ));
    }, [clipLayouts, activeClipInfo?.idx, clips, zoom]);

    const getUrl = (p: string) => `local/${p.replace(/\\/g, '/')}`;

    if (!plan) return null;

    return (
        <div className="montage-editor-overlay animate-fade">
            <div className="montage-editor-window">
                <div className="montage-editor-header">
                    <div className="montage-editor-title">{t('pipeline.montage_control') || 'Montage Editor'} - {task.name}</div>
                    <div className="montage-editor-controls">
                        <button className="montage-btn icon" onClick={() => setZoom(p => Math.max(p - 10, 10))}>-</button>
                        <span className="montage-zoom-label">{zoom}%</span>
                        <button className="montage-btn icon" onClick={() => setZoom(p => Math.min(p + 10, 200))}>+</button>
                    </div>
                </div>
                <div className="montage-editor-body">
                    <div className="playback-overlay-controls">
                        <div className="playback-controls">
                            <button className="play-btn" onClick={handleTogglePlay}>
                                {isPlaying ? <svg viewBox="0 0 24 24" width="28" height="28" fill="currentColor"><path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" /></svg> : <svg viewBox="0 0 24 24" width="28" height="28" fill="currentColor"><path d="M8 5v14l11-7z" /></svg>}
                            </button>
                            <div className="volume-control">
                                <input type="range" min="0" max="1" step="0.01" value={volume} onChange={(e) => setVolume(parseFloat(e.target.value))} />
                                <span className="volume-label">{Math.round(volume * 100)}%</span>
                            </div>
                        </div>
                        <div className="selection-controls-container">
                            <button 
                                className={`scissors-toggle ${isCuttingMode ? 'active' : ''}`} 
                                onClick={() => setIsCuttingMode(!isCuttingMode)}
                                title="Toggle Trimming Tools"
                            >
                                <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                                    <circle cx="6" cy="6" r="3"></circle>
                                    <circle cx="6" cy="18" r="3"></circle>
                                    <line x1="20" y1="4" x2="8.12" y2="15.88"></line>
                                    <line x1="14.47" y1="14.48" x2="20" y2="20"></line>
                                    <line x1="8.12" y1="8.12" x2="12" y2="12"></line>
                                </svg>
                                <span>{isCuttingMode ? 'Close Tools' : 'Cut & Trim'}</span>
                            </button>

                            {isCuttingMode && (
                                <div className="selection-controls-expanded animate-slide-in">
                                    <button 
                                        className={`selection-btn ${selection.start !== null ? 'active' : ''}`} 
                                        onClick={handleMarkIn}
                                    >
                                        <span className="icon">①</span> {selection.start !== null ? 'Reposition Point 1' : 'Set Point 1'}
                                    </button>
                                    <button 
                                        className={`selection-btn ${selection.end !== null ? 'active' : ''}`} 
                                        onClick={handleMarkOut}
                                    >
                                        <span className="icon">②</span> {selection.end !== null ? 'Reposition Point 2' : 'Set Point 2'}
                                    </button>
                                    <button 
                                        className="selection-btn danger large" 
                                        disabled={selection.start === null || selection.end === null} 
                                        onClick={handleCutSelection}
                                    >
                                        <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
                                            <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
                                        </svg>
                                        <span>CONFIRM & CUT</span>
                                    </button>
                                    {(selection.start !== null || selection.end !== null) && (
                                        <button className="selection-btn secondary" onClick={handleClearSelection}>
                                            Reset Selection
                                        </button>
                                    )}
                                </div>
                            )}
                        </div>
                    </div>
                    <div className="montage-preview-and-info">
                        <div className="montage-preview-container">
                            {activeClipInfo ? (
                                <div className="montage-preview-wrap">
                                    {activeClipInfo.clip.isVideo ? <video ref={previewVideoRef} src={getUrl(activeClipInfo.clip.path)} playsInline /> : <img src={getUrl(activeClipInfo.clip.path)} alt="p" />}
                                    <div className="preview-timestamp">{currentTime.toFixed(2)}s</div>
                                    {plan.audioPath && <audio ref={previewAudioRef} src={getUrl(plan.audioPath)} style={{ display: 'none' }} />}
                                    
                                    {currentSubtitle && (
                                        <div className="preview-subtitle-overlay animate-fade">
                                            {currentSubtitle.text}
                                        </div>
                                    )}
                                </div>
                            ) : <div className="montage-preview-placeholder">No preview</div>}
                        </div>
                        <div className="montage-info-panel">
                            <div className="info-item"><span className="info-label">Clips Count:</span><span className="info-value">{clips.length} items</span></div>
                            <div className="info-item"><span className="info-label">Current Duration:</span><span className="info-value">{totalTimelineDuration.toFixed(1)}s</span></div>
                            
                            {selection.start !== null && selection.end !== null && (
                                <div className="cut-preview-stats animate-fade-in">
                                    <div className="stats-divider" />
                                    <div className="info-item highlight">
                                        <span className="info-label">Selection to Cut:</span>
                                        <span className="info-value">-{Math.abs(selection.end - selection.start).toFixed(2)}s</span>
                                    </div>
                                    <div className="info-item">
                                        <span className="info-label">Duration After Cut:</span>
                                        <span className="info-value">{(totalTimelineDuration - Math.abs(selection.end - selection.start)).toFixed(2)}s</span>
                                    </div>
                                </div>
                            )}
                        </div>
                    </div>
                    <div className="montage-timeline-resizer" onMouseDown={handleResizeMouseDown}><div className="resizer-handle-line"></div></div>
                    <div className="montage-timeline-container" ref={containerRef} onMouseDown={handleMouseDownGlobal} style={{ height: `${timelineHeight}px`, flex: 'none' }}>
                        <div className="montage-timeline-wrapper" style={{ width: `${totalTimelineDuration * zoom + 100}px`, minWidth: '100%' }}>
                            <div className="montage-timeline-ruler">
                                {markers}
                                {selection.start !== null && (
                                    <div 
                                        className="selection-marker start interactive" 
                                        style={{ left: `${selection.start * zoom}px` }}
                                        onMouseDown={(e) => { e.stopPropagation(); setDraggingSelectionSide('start'); }}
                                    >
                                        <div className="marker-handle" />
                                    </div>
                                )}
                                {selection.end !== null && (
                                    <div 
                                        className="selection-marker end interactive" 
                                        style={{ left: `${selection.end * zoom}px` }}
                                        onMouseDown={(e) => { e.stopPropagation(); setDraggingSelectionSide('end'); }}
                                    >
                                        <div className="marker-handle" />
                                    </div>
                                )}
                                {selection.start !== null && selection.end !== null && <div className="selection-range" style={{ left: `${Math.min(selection.start, selection.end) * zoom}px`, width: `${Math.abs(selection.end - selection.start) * zoom}px` }} />}
                                
                                {cutJunctions.map((j, i) => (
                                    <div key={i} className="timeline-cut-junction" style={{ left: `${j.position * zoom}px` }}>
                                        <div className="junction-icon">✂</div>
                                        <div className="junction-tooltip">Cut: -{j.durationRemoved.toFixed(2)}s</div>
                                    </div>
                                ))}

                                <div className="audio-track-reference" style={{ width: `${totalTimelineDuration * zoom}px` }}><span>Audio Sequence</span></div>
                            </div>
                            <div className="montage-timeline-tracks"><div className="montage-track">{clipElements}</div></div>
                            <div className="montage-playhead" style={{ left: `${currentTime * zoom}px` }}><div className="playhead-handle" /></div>
                        </div>
                    </div>
                </div>
                <div className="montage-editor-footer">
                    <button className="montage-btn secondary" onClick={() => onCancel(task.id)}>{t('common.cancel')}</button>
                    <button className="montage-btn primary" onClick={() => {
                        const ds = clips.map(c => c.duration.toFixed(3)).join(',');
                        const ss = audioSegments.map(s => `${s.start.toFixed(3)},${s.end.toFixed(3)}`).join('|');
                        onConfirm(task.id, `confirm:${ds};segments:${ss}`);
                    }}>{t('common.save')} & {t('queue.start')}</button>
                </div>
            </div>
        </div>
    );
};
