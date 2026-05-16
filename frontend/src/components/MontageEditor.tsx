import React, { useState, useEffect, useRef, useMemo, useCallback } from 'react';
import { useI18n } from '../contexts/I18nContext';
import './MontageEditor.css';
import { QueueTask } from '../contexts/QueueContext';
import { RegenerateModal } from './RegenerateModal';
import { OnFileDrop, OnFileDropOff } from '../../wailsjs/runtime/runtime';

interface MontageClip {
    path: string;
    duration: number;
    isVideo: boolean;
    actualDuration?: number;
    source?: 'footage' | 'generated';
}

interface MontageSegment {
    start: number;
    end: number;
}

interface MontageTrigger {
    phrase: string;
    path: string;
    startTime: number;
    duration: number;
    isVideo: boolean;
    x: number;
    y: number;
    w: number;
    h: number;
}

interface MontageWatermark {
    id: string;
    path: string;
    startTime: number;
    duration: number;
    x: number;
    y: number;
    w: number;
    h: number;
    opacity: number;
    trackId?: string;
    isVideo?: boolean;
    initialPosition?: string;
}

export interface MontageTrack {
    id: string;
    name: string;
    type: 'image' | 'video' | 'watermark';
    color: string;
}

interface ProjectPlan {
    baseW: number;
    baseH: number;
    audioDuration: number;
    audioPath: string | null;
    transDuration: number;
    isFadeFast: boolean;
    clips: MontageClip[];
    subtitlePath?: string;
    audioSegments?: MontageSegment[];
    triggers?: MontageTrigger[];
    watermarks?: MontageWatermark[];
    extraTracks?: MontageTrack[];
    introPath?: string;
    introDuration?: number;
    introIsVideo?: boolean;
}

interface MontagePlan extends ProjectPlan {}

const WATERMARK_POSITIONS: Record<string, React.CSSProperties> = {
    'top-left': { top: '5%', left: '5%' },
    'top-center': { top: '5%', left: '50%', transform: 'translateX(-50%)' },
    'top-right': { top: '5%', right: '5%' },
    'bottom-left': { bottom: '5%', left: '5%' },
    'bottom-center': { bottom: '5%', left: '50%', transform: 'translateX(-50%)' },
    'bottom-right': { bottom: '5%', right: '5%' },
    'center': { top: '50%', left: '50%', transform: 'translate(-50%, -50%)' },
};

const WatermarkPreviewItem = React.memo(({ 
    w, 
    plan, 
    selectedWatermarkIdx, 
    draggingWatermarkPosIdx, 
    setSelectedWatermarkIdx, 
    setSelectedTriggerIdx, 
    setActiveInfoTab, 
    setDraggingWatermarkPosIdx, 
    setDragStartCoords, 
    setResizingWatermarkIdx, 
    setResizingWatermarkHandle, 
    getUrl, 
    adjustedTime,
    isPlaying,
    handleDeleteWatermark,
    onDimensionsLoad
}: any) => {
    const videoRef = useRef<HTMLVideoElement>(null);

    useEffect(() => {
        if (w.isVideo && videoRef.current) {
            const target = adjustedTime - w.startTime;
            if (!isPlaying) {
                if (Math.abs(videoRef.current.currentTime - target) > 0.03) {
                    videoRef.current.currentTime = Math.max(0, target);
                }
            } else {
                const drift = Math.abs(videoRef.current.currentTime - target);
                if (drift > 0.3) {
                    videoRef.current.currentTime = Math.max(0, target);
                }
                if (videoRef.current.paused) {
                    videoRef.current.play().catch(() => {});
                }
            }
        }
    }, [adjustedTime, w.startTime, w.isVideo, isPlaying]);

    return (
        <div 
            className={`preview-watermark-overlay ${selectedWatermarkIdx === w.index ? 'selected' : ''} ${draggingWatermarkPosIdx === w.index ? 'dragging' : ''}`}
            style={{
                left: `${(w.x / (plan.baseW || 1920)) * 100}%`,
                top: `${(w.y / (plan.baseH || 1080)) * 100}%`,
                width: `${(w.w / (plan.baseW || 1920)) * 100}%`,
                height: 'auto',
                minHeight: `${(w.h / (plan.baseH || 1080)) * 100}%`,
                opacity: w.opacity
            }}
            onMouseDown={(e) => {
                e.stopPropagation();
                setSelectedWatermarkIdx(w.index);
                setSelectedTriggerIdx(null);
                setActiveInfoTab('stats');
                setDraggingWatermarkPosIdx(w.index);
                setDragStartCoords({ x: w.x, y: w.y, mouseX: e.clientX, mouseY: e.clientY });
            }}
        >
            {w.isVideo ? (
                <video 
                    ref={videoRef} 
                    src={getUrl(w.path)} 
                    muted 
                    playsInline 
                    style={{ width: '100%', height: 'auto', objectFit: 'contain', pointerEvents: 'none', background: 'transparent' }} 
                    onLoadedMetadata={(e) => {
                        const v = e.currentTarget;
                        if (v.videoWidth && v.videoHeight) {
                            onDimensionsLoad(w.index, v.videoWidth, v.videoHeight);
                        }
                    }}
                />
            ) : (
                <img 
                    src={getUrl(w.path)} 
                    alt="Watermark" 
                    style={{ width: '100%', height: 'auto', objectFit: 'contain', pointerEvents: 'none' }} 
                    onLoad={(e) => {
                        const img = e.currentTarget;
                        if (img.naturalWidth && img.naturalHeight) {
                            onDimensionsLoad(w.index, img.naturalWidth, img.naturalHeight);
                        }
                    }}
                />
            )}
            {selectedWatermarkIdx === w.index && (
                <>
                    <div className="watermark-resize-handle br" onMouseDown={(e) => { e.stopPropagation(); setResizingWatermarkIdx(w.index); setResizingWatermarkHandle('br'); setDragStartCoords({ x: w.w, y: w.h, mouseX: e.clientX, mouseY: e.clientY }); }} />
                    <div className="watermark-resize-handle tl" onMouseDown={(e) => { e.stopPropagation(); setResizingWatermarkIdx(w.index); setResizingWatermarkHandle('tl'); setDragStartCoords({ x: w.w, y: w.h, x2: w.x, y2: w.y, mouseX: e.clientX, mouseY: e.clientY }); }} />
                    <button className="preview-delete-btn" title="Delete" onMouseDown={(e) => { e.stopPropagation(); handleDeleteWatermark(w.index); setSelectedWatermarkIdx(null); }}>✕</button>
                </>
            )}
        </div>
    );
});

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
    const [activeInfoTab, setActiveInfoTab] = useState<'library' | 'stats'>('library');
    const [selection, setSelection] = useState<{ start: number | null, end: number | null }>({ start: null, end: null });
    const [audioSegments, setAudioSegments] = useState<MontageSegment[]>([]);
    const [isCuttingMode, setIsCuttingMode] = useState<boolean>(false);
    const [draggingSelectionSide, setDraggingSelectionSide] = useState<null | 'start' | 'end'>(null);
    const [cutJunctions, setCutJunctions] = useState<{ position: number, durationRemoved: number }[]>([]);
    const [subtitles, setSubtitles] = useState<SubtitleEntry[]>([]);
    const [introVideo, setIntroVideo] = useState<MontageClip | null>(null);

    const [triggers, setTriggers] = useState<MontageTrigger[]>([]);
    const [draggingTriggerIdx, setDraggingTriggerIdx] = useState<number | null>(null);
    const [draggingTriggerSide, setDraggingTriggerSide] = useState<null | 'start' | 'end'>(null);
    const [dragTriggerStartPos, setDragTriggerStartPos] = useState<number>(0);
    const [dragTriggerStartDur, setDragTriggerStartDur] = useState<number>(0);
    const [dragTriggerOffsetX, setDragTriggerOffsetX] = useState<number>(0);
    const [draggingTriggerPosIdx, setDraggingTriggerPosIdx] = useState<number | null>(null);
    const [dragStartCoords, setDragStartCoords] = useState<{ x: number, y: number, mouseX: number, mouseY: number, x2?: number, y2?: number } | null>(null);

    const [localSettings, setLocalSettings] = useState(task.settings || {});
    const [watermarks, setWatermarks] = useState<MontageWatermark[]>([]);
    const [draggingWatermarkIdx, setDraggingWatermarkIdx] = useState<number | null>(null);
    const [draggingWatermarkPosIdx, setDraggingWatermarkPosIdx] = useState<number | null>(null);
    const [draggingWatermarkSide, setDraggingWatermarkSide] = useState<null | 'start' | 'end'>(null);
    const [resizingWatermarkIdx, setResizingWatermarkIdx] = useState<number | null>(null);
    const [resizingWatermarkHandle, setResizingWatermarkHandle] = useState<string | null>(null);
    const [dragWatermarkStartPos, setDragWatermarkStartPos] = useState<number>(0);
    const [dragWatermarkStartDur, setDragWatermarkStartDur] = useState<number>(0);
    const [selectedWatermarkIdx, setSelectedWatermarkIdx] = useState<number | null>(null);
    const [selectedTriggerIdx, setSelectedTriggerIdx] = useState<number | null>(null);
    const lastMousePos = useRef<{ x: number, y: number } | null>(null);

    const previewVideoRef = useRef<HTMLVideoElement>(null);
    const previewAudioRef = useRef<HTMLAudioElement>(null);
    const previewWrapRef = useRef<HTMLDivElement>(null);
    const containerRef = useRef<HTMLDivElement>(null);
    const poolRef = useRef<HTMLDivElement>(null);
    const hasMovedRef = useRef<boolean>(false);
    const requestRef = useRef<number>();
    const lastTimeRef = useRef<number>(0);

    const [draggingIdx, setDraggingIdx] = useState<number | null>(null);
    const [startX, setStartX] = useState<number>(0);
    const [startDurations, setStartDurations] = useState<{ current: number, next: number }>({ current: 0, next: 0 });
    const [timelineHeight, setTimelineHeight] = useState<number>(() => {
        const saved = localStorage.getItem('montage-timeline-height');
        return saved ? parseInt(saved, 10) : 300;
    });
    const [infoPanelWidth, setInfoPanelWidth] = useState<number>(() => {
        const saved = localStorage.getItem('montage-info-width');
        return saved ? parseInt(saved, 10) : 340;
    });
    const isResizingRef = useRef<boolean>(false);

    // CLIP ACTIONS STATE
    const [isRegModalOpen, setIsRegModalOpen] = useState<boolean>(false);
    const [regIdx, setRegIdx] = useState<number | null>(null);
    const [regeneratingIndices, setRegeneratingIndices] = useState<Set<number>>(new Set());
    const [clipBusters, setClipBusters] = useState<Record<string, number>>({});
    const [prompts, setPrompts] = useState<string[]>([]);
    const [mediaPool, setMediaPool] = useState<MontageClip[]>([]);
    const [isDraggingFromPool, setIsDraggingFromPool] = useState<MontageClip | null>(null);
    const dragPoolItemRef = useRef<MontageClip | null>(null);
    const [dragPos, setDragPos] = useState<{x: number, y: number} | null>(null);
    const [extraTracks, setExtraTracks] = useState<MontageTrack[]>([]);
    const [draggingHoverTrack, setDraggingHoverTrack] = useState<'clips' | 'triggers' | 'watermarks' | 'intro' | string | null>(null);
    const [isDraggingExternal, setIsDraggingExternal] = useState(false);
    const dragCounter = useRef(0);
    const [dropPreview, setDropPreview] = useState<number | null>(null);
    const [hoveredMediaIdx, setHoveredMediaIdx] = useState<number | null>(null);
    const [hoveredTimelinePoolIdx, setHoveredTimelinePoolIdx] = useState<number | null>(null);
    const [fullscreenClipIdx, setFullscreenClipIdx] = useState<number | null>(null);
    const [fullscreenPoolIdx, setFullscreenPoolIdx] = useState<number | null>(null);
    const [editingTriggerIdx, setEditingTriggerIdx] = useState<number | null>(null);
    const [tempTriggerPhrase, setTempTriggerPhrase] = useState<string>("");
    const [previewAspect, setPreviewAspect] = useState<number>(1);

    const importFiles = useCallback(async (paths: string[]) => {
        for (const path of paths) {
            try {
                // @ts-ignore
                const res = await window.go.main.App.ImportMediaFile(
                    task.id, task.folderName, task.type, task.subName, task.settings || {}, path
                );
                if (res) {
                    setMediaPool(prev => [...prev, {
                        path: res.path,
                        duration: res.duration,
                        isVideo: res.isVideo,
                        actualDuration: res.actualDuration
                    }]);
                }
            } catch (e) {
                console.error("Import failed for", path, e);
            }
        }
    }, [task]);

    // Initial Load & File Drop Registration
    useEffect(() => {
        const handleFileDrop = (x: number, y: number, paths: string[]) => {
            setIsDraggingExternal(false);
            dragCounter.current = 0;
            if (paths && paths.length > 0) {
                importFiles(paths);
            }
        };

        OnFileDrop(handleFileDrop, true);
        return () => OnFileDropOff();
    }, [importFiles]);

    useEffect(() => {
        if (task.montagePlanData) {
            try {
                const parsed = JSON.parse(task.montagePlanData);
                setPlan(parsed);
                setClips(parsed.clips.map((c: MontageClip) => ({
                    ...c,
                    isVideo: c.isVideo || c.path.toLowerCase().endsWith('.mp4')
                })));
                if (parsed.poolFiles && parsed.poolFiles.length > 0) {
                    setMediaPool(parsed.poolFiles.map((c: MontageClip) => ({
                        ...c,
                        isVideo: c.isVideo || c.path.toLowerCase().endsWith('.mp4')
                    })));
                }
                if (parsed.audioSegments && parsed.audioSegments.length > 0) {
                    setAudioSegments(parsed.audioSegments);
                } else {
                    setAudioSegments([{ start: 0, end: parsed.audioDuration }]);
                }
                if (parsed.triggers) {
                    setTriggers(parsed.triggers.map((t: MontageTrigger) => ({ ...t })));
                } else {
                    setTriggers([]);
                }

                let initialWatermarks = parsed.watermarks ? parsed.watermarks.map((w: MontageWatermark) => ({ ...w })) : [];
                
                // AUTO-CONVERT GLOBAL WATERMARK IF ENABLED
                const s = task.settings || {};
                const hasGlobalWM = initialWatermarks.some((wm: MontageWatermark) => wm.id && wm.id.includes('wm_global_'));

                if (s.montageWatermarkEnabled && s.montageWatermarkPath && !hasGlobalWM) {
                    const bw = parsed.baseW || 1920;
                    const bh = parsed.baseH || 1080;
                    const sz = s.montageWatermarkSize || 15;
                    const op = s.montageWatermarkOpacity !== undefined ? s.montageWatermarkOpacity : 0.8;
                    const pos = s.montageWatermarkPosition || 'bottom-right';
                    
                    const wmW = Math.round(bw * (sz / 100));
                    const wmH = Math.round(bh * (sz / 100));
                    let wmX = 0; let wmY = 0;
                    const margin = 20;
                    
                    switch(pos) {
                        case 'top-left': wmX = margin; wmY = margin; break;
                        case 'top-center': wmX = (bw - wmW) / 2; wmY = margin; break;
                        case 'top-right': wmX = bw - wmW - margin; wmY = margin; break;
                        case 'bottom-left': wmX = margin; wmY = bh - wmH - margin; break;
                        case 'bottom-center': wmX = (bw - wmW) / 2; wmY = bh - wmH - margin; break;
                        case 'bottom-right': wmX = bw - wmW - margin; wmY = bh - wmH - margin; break;
                        case 'center': wmX = (bw - wmW) / 2; wmY = (bh - wmH) / 2; break;
                        default: wmX = bw - wmW - margin; wmY = bh - wmH - margin;
                    }

                    initialWatermarks.push({
                        id: 'wm_global_auto_' + Date.now(),
                        path: s.montageWatermarkPath,
                        startTime: 0,
                        duration: parsed.audioDuration,
                        x: wmX, y: wmY, w: wmW, h: wmH,
                        opacity: op,
                        isVideo: s.montageWatermarkPath.toLowerCase().endsWith('.mp4'),
                        initialPosition: pos
                    });
                    
                    // We'll set the local setting to disabled to avoid redundant conversion/rendering
                    setLocalSettings((prev: any) => ({ ...prev, montageWatermarkEnabled: false }));
                }

                setWatermarks(initialWatermarks);

                if (parsed.extraTracks) {
                    setExtraTracks(parsed.extraTracks);
                } else {
                    setExtraTracks([]);
                }
                
                if (parsed.introPath) {
                    setIntroVideo({
                        path: parsed.introPath,
                        duration: parsed.introDuration || 0,
                        isVideo: parsed.introIsVideo || true
                    });
                } else {
                    setIntroVideo(null);
                }

                // Try to load prompts.txt for regeneration
                if (parsed.audioPath) {
                    const parts = parsed.audioPath.split(/[\\/]voice\.mp3/);
                    const taskDir = parts.length > 0 ? parts[0] : null;
                    if (taskDir) {
                        const promptsPath = `${taskDir}/prompts.txt`;
                        // @ts-ignore
                        window.go.main.App.ReadFile(promptsPath).then(content => {
                            if (content) {
                                const pStrs = content.split('\n\n--------------------\n\n').map((s: string) => s.trim());
                                setPrompts(pStrs);
                            }
                        }).catch(() => console.log("No prompts.txt found"));
                    }
                }
            } catch (e) {
                console.error("Failed to parse montage plan:", e);
            }
        }
    }, [task.montagePlanData]);

    useEffect(() => {
        if (fullscreenClipIdx === null && fullscreenPoolIdx === null) return;
        const handler = (e: KeyboardEvent) => {
            if (fullscreenClipIdx !== null) {
                if (e.key === 'Escape') { setFullscreenClipIdx(null); }
                else if (e.key === 'ArrowRight') { setFullscreenClipIdx(prev => prev !== null ? Math.min(prev + 1, clips.length - 1) : null); }
                else if (e.key === 'ArrowLeft') { setFullscreenClipIdx(prev => prev !== null ? Math.max(prev - 1, 0) : null); }
            } else if (fullscreenPoolIdx !== null) {
                if (e.key === 'Escape') { setFullscreenPoolIdx(null); }
                else if (e.key === 'ArrowRight') { setFullscreenPoolIdx(prev => prev !== null ? Math.min(prev + 1, mediaPool.length - 1) : null); }
                else if (e.key === 'ArrowLeft') { setFullscreenPoolIdx(prev => prev !== null ? Math.max(prev - 1, 0) : null); }
            }
        };
        window.addEventListener('keydown', handler);
        return () => window.removeEventListener('keydown', handler);
    }, [fullscreenClipIdx, fullscreenPoolIdx, clips.length, mediaPool.length]);

    const getUrl = useCallback((p: string) => {
        let url = `local/${p.replace(/\\/g, '/')}`;
        if (clipBusters[p]) {
            url += `?buster=${clipBusters[p]}`;
        }
        return url;
    }, [clipBusters]);

    // Safety: Lock total duration to audio duration
    useEffect(() => {
        if (!plan || clips.length === 0) return;
        const currentTotal = clips.reduce((sum, c) => sum + c.duration, 0);
        const targetTotal = plan.audioDuration;
        if (Math.abs(currentTotal - targetTotal) > 0.005) {
            setClips(prev => {
                if (prev.length === 0) return prev;
                const next = [...prev];
                const lastIdx = next.length - 1;
                const diff = targetTotal - currentTotal;
                if (next[lastIdx].duration + diff > 0.05) {
                    next[lastIdx] = { ...next[lastIdx], duration: next[lastIdx].duration + diff };
                }
                return next;
            });
        }
    }, [clips, plan]);

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
    }, [plan?.subtitlePath, getUrl]);

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
            return { clip, idx, width, x, isIntro: false };
        });
    }, [clips, zoom, plan, introVideo]);

    const actualVideoDuration = useMemo(() => {
        if (clipLayouts.length === 0) return 0;
        const last = clipLayouts[clipLayouts.length - 1];
        return (last.x + last.width) / zoom;
    }, [clipLayouts, zoom]);

    const effectiveIntroDuration = useMemo(() => {
        return introVideo ? introVideo.duration : 0;
    }, [introVideo]);

    const introWidth = useMemo(() => {
        // We always reserve space for the intro slot to prevent jarring timeline shifts
        // and ensure the drop target is always visible to the user.
        return introVideo ? Math.max(introVideo.duration * zoom, 160) : 160;
    }, [introVideo, zoom]);

    // Refs so stable mouse handlers can read latest values without stale closures
    const zoomRef = useRef(zoom);
    zoomRef.current = zoom;
    const introWidthRef = useRef(introWidth);
    introWidthRef.current = introWidth;
    const introVideoRef = useRef(introVideo);
    introVideoRef.current = introVideo;

    const totalTimelineDuration = useMemo(() => {
        const audioTotal = audioSegments.reduce((sum, seg) => sum + (seg.end - seg.start), 0);
        return Math.max(actualVideoDuration, audioTotal) + effectiveIntroDuration;
    }, [actualVideoDuration, audioSegments, effectiveIntroDuration]);

    const activeClipIdx = useMemo(() => {
        if (introVideo && currentTime < effectiveIntroDuration) return -1;
        const adjustedTime = introVideo ? currentTime - effectiveIntroDuration : currentTime;
        for (let i = clipLayouts.length - 1; i >= 0; i--) {
            const layout = clipLayouts[i];
            const startTime = layout.x / zoom;
            const endTime = (layout.x + layout.width) / zoom;
            if (adjustedTime >= startTime && adjustedTime <= endTime + 0.001) {
                return i;
            }
        }
        return null;
    }, [clipLayouts, currentTime, introVideo, effectiveIntroDuration]);

    const activeClipLayout = useMemo(() => {
        if (activeClipIdx === -1) return {
            clip: introVideo!,
            idx: -1,
            width: introWidth,
            x: 0,
            isIntro: true
        };
        if (activeClipIdx === null) return null;
        return clipLayouts[activeClipIdx];
    }, [activeClipIdx, clipLayouts, introVideo, introWidth]);

    const timeInClip = useMemo(() => {
        if (!activeClipLayout) return 0;
        const adjustedTime = (introVideo && !activeClipLayout.isIntro) ? currentTime - effectiveIntroDuration : currentTime;
        let t = activeClipLayout.isIntro ? adjustedTime : (adjustedTime - activeClipLayout.x / zoom);
        
        // BOOMERANG LOGIC
        if (activeClipLayout.clip.isVideo && activeClipLayout.clip.actualDuration && activeClipLayout.clip.actualDuration < activeClipLayout.clip.duration) {
            const actualDur = activeClipLayout.clip.actualDuration;
            const cycle = actualDur * 2;
            const pos = t % cycle;
            if (pos <= actualDur) t = pos;
            else t = actualDur - (pos - actualDur);
        }
        return t;
    }, [activeClipLayout, currentTime, introVideo, effectiveIntroDuration, zoom]);

    const activeClipInfo = useMemo(() => {
        if (!activeClipLayout) return null;
        return { ...activeClipLayout, timeInClip };
    }, [activeClipLayout, timeInClip]);

    const animStateRef = useRef<AnimationState>({
        currentTime, selection, isPlaying, audioSegments, clips, zoom, totalDuration: totalTimelineDuration
    });
    animStateRef.current = { currentTime, selection, isPlaying, audioSegments, clips, zoom, totalDuration: totalTimelineDuration };

    const getOriginalTime = useCallback((timelineTime: number) => {
        let currentTimeline = introVideo ? effectiveIntroDuration : 0;
        if (timelineTime < currentTimeline) return audioSegments[0]?.start || 0;

        let contentTime = introVideo ? timelineTime - effectiveIntroDuration : timelineTime;
        let trackPos = 0;

        for (const seg of audioSegments) {
            const segDur = seg.end - seg.start;
            if (contentTime <= trackPos + segDur + 0.001) {
                return seg.start + (contentTime - trackPos);
            }
            trackPos += segDur;
        }
        return audioSegments.length > 0 ? audioSegments[audioSegments.length - 1].end : timelineTime;
    }, [audioSegments, introVideo, effectiveIntroDuration]);

    const currentSubtitle = useMemo(() => {
        if (introVideo && currentTime < effectiveIntroDuration) return undefined;
        const origTime = getOriginalTime(currentTime);
        return subtitles.find(s => origTime >= s.start && origTime <= s.end);
    }, [subtitles, currentTime, getOriginalTime, introVideo, effectiveIntroDuration]);

    const activeClipInfoRef = useRef(activeClipInfo);
    activeClipInfoRef.current = activeClipInfo;

    useEffect(() => {
        if (!isPlaying && previewVideoRef.current && activeClipInfo?.clip.isVideo) {
            const targetV = activeClipInfo.timeInClip;
            // Scrubbing/Dragging: Faster sync
            if (Math.abs(previewVideoRef.current.currentTime - targetV) > 0.05) {
                previewVideoRef.current.currentTime = targetV;
            }
        }
    }, [activeClipInfo, isPlaying]);

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
                if (Math.abs(previewAudioRef.current.currentTime - targetOrig) > 0.5) {
                    previewAudioRef.current.currentTime = targetOrig;
                }
                const isIntroNow = !!introVideo && next < effectiveIntroDuration;
                if (previewAudioRef.current.muted !== isIntroNow) {
                    previewAudioRef.current.muted = isIntroNow;
                }
            }
            if (previewVideoRef.current && activeClipInfoRef.current?.clip.isVideo) {
                 const targetV = activeClipInfoRef.current.timeInClip;
                 // Jitter prevention: only sync if significantly off
                 const drift = Math.abs(previewVideoRef.current.currentTime - targetV);
                 if (drift > 0.4) {
                     previewVideoRef.current.currentTime = targetV;
                 }
                 if (previewVideoRef.current.paused) {
                     previewVideoRef.current.play().catch(() => {});
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
                previewAudioRef.current.muted = !!introVideo && currentTime < effectiveIntroDuration;
                previewAudioRef.current.play().catch((e) => console.error("Audio play failed:", e));
            }
            if (previewVideoRef.current && activeClipInfo?.clip.isVideo) {
                previewVideoRef.current.currentTime = activeClipInfo.timeInClip;
                previewVideoRef.current.muted = false; // Ensure unmuted
                previewVideoRef.current.play().catch(() => {});
            }
        } else {
            if (requestRef.current) cancelAnimationFrame(requestRef.current);
            if (previewAudioRef.current) previewAudioRef.current.pause();
            if (previewVideoRef.current) previewVideoRef.current.pause();
        }
        return () => { if (requestRef.current) cancelAnimationFrame(requestRef.current); };
    }, [isPlaying, animate, getOriginalTime]); // Removed currentTime and activeClipInfo specific details to prevent constant restarts

    useEffect(() => {
        if (!isPlaying) {
            if (previewAudioRef.current) previewAudioRef.current.currentTime = getOriginalTime(currentTime);
            if (previewVideoRef.current && activeClipInfo?.clip.isVideo) {
                previewVideoRef.current.currentTime = activeClipInfo.timeInClip;
            }
        }
    }, [currentTime, isPlaying, activeClipInfo?.idx, getOriginalTime]);

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
        const xRaw = e.clientX - rect.left + containerRef.current.scrollLeft;
        
        let targetTime = 0;
        if (introVideo) {
            targetTime = xRaw / zoom;
        } else {
            targetTime = (xRaw - introWidth) / zoom;
        }
        
        setCurrentTime(Math.max(0, Math.min(animStateRef.current.totalDuration, targetTime)));
    }, [introWidth, introVideo, zoom]);

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

    const handleMouseUp = useCallback(() => {
        setDraggingIdx(null);
        setDraggingSelectionSide(null);
        setDraggingTriggerIdx(null);
        setDraggingTriggerSide(null);
        setDraggingTriggerPosIdx(null);
        setDraggingWatermarkIdx(null);
        setDraggingWatermarkPosIdx(null);
        setDraggingWatermarkSide(null);
        setResizingWatermarkIdx(null);
        setResizingWatermarkHandle(null);
        setDragStartCoords(null);
        setIsScrubbing(false);
        hasMovedRef.current = false;
    }, []);

    const handleDeleteClip = useCallback((idx: number) => {
        setClips(prevClips => {
            if (prevClips.length <= 1) return prevClips;
            const newClips = [...prevClips];
            const removedDuration = newClips[idx].duration;
            if (idx > 0) {
                newClips[idx - 1] = { ...newClips[idx - 1], duration: newClips[idx - 1].duration + removedDuration };
            } else {
                newClips[idx + 1] = { ...newClips[idx + 1], duration: newClips[idx + 1].duration + removedDuration };
            }
            newClips.splice(idx, 1);
            return newClips;
        });
    }, []);

    const activeTriggersAtTime = useMemo(() => {
        const adjustedTime = introVideo ? currentTime - effectiveIntroDuration : currentTime;
        return triggers.map((t, i) => ({ ...t, index: i }))
                       .filter(tr => adjustedTime >= tr.startTime && adjustedTime <= tr.startTime + tr.duration);
    }, [triggers, currentTime, introVideo, effectiveIntroDuration]);

    const activeWatermarksAtTime = useMemo(() => {
        const adjustedTime = introVideo ? currentTime - effectiveIntroDuration : currentTime;
        return watermarks.map((w, i) => ({ ...w, index: i }))
                          .filter(w => adjustedTime >= w.startTime && adjustedTime <= w.startTime + w.duration);
    }, [watermarks, currentTime, introVideo, effectiveIntroDuration]);

    const handleWatermarkDimensionsLoad = useCallback((idx: number, naturalW: number, naturalH: number) => {
        setWatermarks(prev => {
            const next = [...prev];
            const wm = next[idx];
            if (!wm) return prev;
            
            const currentW = wm.w;
            const aspect = naturalW / naturalH;
            const newH = Math.round(currentW / aspect);
            
            if (Math.abs(wm.h - newH) > 1) {
                let newX = wm.x;
                let newY = wm.y;
                if (wm.id && wm.id.startsWith('wm_global_auto_') && wm.initialPosition) {
                    const bw = plan?.baseW || 1920;
                    const bh = plan?.baseH || 1080;
                    const margin = 20;
                    const pos = wm.initialPosition;
                    switch(pos) {
                        case 'top-left': newX = margin; newY = margin; break;
                        case 'top-center': newX = (bw - currentW) / 2; newY = margin; break;
                        case 'top-right': newX = bw - currentW - margin; newY = margin; break;
                        case 'bottom-left': newX = margin; newY = bh - newH - margin; break;
                        case 'bottom-center': newX = (bw - currentW) / 2; newY = bh - newH - margin; break;
                        case 'bottom-right': newX = bw - currentW - margin; newY = bh - newH - margin; break;
                        case 'center': newX = (bw - currentW) / 2; newY = (bh - newH) / 2; break;
                    }
                }
                next[idx] = { ...wm, h: newH, x: newX, y: newY };
                return next;
            }
            return prev;
        });
    }, [plan, task.settings]);

    const handleDeleteWatermark = useCallback((idx: number) => {
        setWatermarks(prev => prev.filter((_, i) => i !== idx));
        setSelectedWatermarkIdx(null);
    }, []);

    const handleDeleteTrigger = useCallback((idx: number) => {
        setTriggers(prev => prev.filter((_, i) => i !== idx));
        setSelectedTriggerIdx(null);
    }, []);

    const handleSaveTriggerPhrase = () => {
        if (editingTriggerIdx !== null) {
            setTriggers(p => {
                const nt = [...p];
                nt[editingTriggerIdx] = { ...nt[editingTriggerIdx], phrase: tempTriggerPhrase };
                return nt;
            });
            setEditingTriggerIdx(null);
        }
    };

    const handleAnimateClip = useCallback(async (idx: number) => {
        const targetClip = clips[idx];
        if (targetClip.isVideo) return;
        setRegeneratingIndices(prev => new Set(prev).add(idx));
        try {
            // @ts-ignore
            const newPath = await window.go.main.App.AnimateGalleryImage(targetClip.path);
            if (newPath) {
                let actualDur = 0;
                const v = document.createElement('video');
                v.src = `local/${newPath.replace(/\\/g, '/')}`;
                await new Promise(r => {
                    v.onloadedmetadata = () => { actualDur = v.duration; r(null); };
                    v.onerror = () => r(null);
                    setTimeout(() => r(null), 2000);
                });
                setClips(prev => {
                    const next = [...prev];
                    next[idx] = { ...next[idx], path: newPath, isVideo: true, actualDuration: actualDur };
                    return next;
                });
                setClipBusters(prev => ({ ...prev, [newPath]: Date.now() }));
            }
        } catch (err) {
            console.error("Animation failed:", err);
        } finally {
            setRegeneratingIndices(prev => { const next = new Set(prev); next.delete(idx); return next; });
        }
    }, [clips]);

    const handleOpenRegenerate = useCallback((idx: number) => {
        setRegIdx(idx);
        setIsRegModalOpen(true);
    }, []);

    const handleRegenerateConfirm = useCallback(async (prompt: string, service: string, settings: any) => {
        if (regIdx === null) return;
        const targetClip = clips[regIdx];
        setRegeneratingIndices(prev => new Set(prev).add(regIdx));
        try {
            // @ts-ignore
            const newPath = await window.go.main.App.RegenerateGalleryImage(targetClip.path, prompt, service, settings);
            if (newPath) {
                let actualDur = targetClip.actualDuration || 0;
                if (newPath.toLowerCase().endsWith('.mp4')) {
                    const v = document.createElement('video');
                    v.src = `local/${newPath.replace(/\\/g, '/')}`;
                    await new Promise(r => {
                        v.onloadedmetadata = () => { actualDur = v.duration; r(null); };
                        v.onerror = () => r(null);
                        setTimeout(() => r(null), 2000); 
                    });
                }
                setClips(prev => {
                    const next = [...prev];
                    next[regIdx] = { 
                        ...next[regIdx], 
                        path: newPath,
                        isVideo: newPath.toLowerCase().endsWith('.mp4'),
                        actualDuration: actualDur
                    };
                    return next;
                });
                setClipBusters(prev => ({ ...prev, [newPath]: Date.now() }));
            }
        } catch (err) {
            console.error("Regeneration failed:", err);
        } finally {
            setRegeneratingIndices(prev => {
                const next = new Set(prev);
                next.delete(regIdx);
                return next;
            });
            setRegIdx(null);
        }
    }, [regIdx, clips]);

    const handleAddMedia = async () => {
        try {
            // @ts-ignore
            const paths = await window.go.main.App.SelectFiles();
            if (paths && paths.length > 0) {
                await importFiles(paths);
            }
        } catch (err) {
            console.error("Failed to pick files:", err);
        }
    };

    const handleRemoveFromPool = (idx: number) => {
        setMediaPool(prev => prev.filter((_, i) => i !== idx));
    };

    const handleInternalDragStart = (item: MontageClip) => {
        console.log('[DND] dragStart', item.path);
        dragPoolItemRef.current = item;
        setIsDraggingFromPool(item);
        setDraggingHoverTrack(null);
    };

    const handleDeleteIntro = () => {
        setIntroVideo(null);
    };

    const handleInternalDrop = useCallback((dropTime: number) => {
        console.log('[DND] handleInternalDrop dropTime=', dropTime, 'isDraggingFromPool=', !!isDraggingFromPool);
        if (!isDraggingFromPool) return;

        // Intro Drop detection (very beginning of timeline)
        if (dropTime < 0.2) {
            setIntroVideo({ ...isDraggingFromPool, duration: isDraggingFromPool.actualDuration || 3.0 });
            dragPoolItemRef.current = null;
            setIsDraggingFromPool(null);
            setDraggingHoverTrack(null);
            setDropPreview(null);
            return;
        }

        setClips(prev => {
            const next: MontageClip[] = [];
            let currentTimePos = introVideo ? introVideo.duration : 0;
            let targetIdx = -1;
            for (let i = 0; i < prev.length; i++) {
                const start = currentTimePos;
                const end = currentTimePos + prev[i].duration;
                if (dropTime >= start && dropTime < end) {
                    targetIdx = i;
                    break;
                }
                currentTimePos += prev[i].duration;
            }
            if (targetIdx === -1) return prev;
            const targetClip = prev[targetIdx];
            const targetStart = (introVideo ? introVideo.duration : 0) + prev.slice(0, targetIdx).reduce((s, c) => s + c.duration, 0);
            const beforeDur = dropTime - targetStart;
            const newClipDur = Math.min(2.0, targetClip.duration - 0.2); 
            if (newClipDur <= 0.1) {
                return prev.map((c, i) => i === targetIdx ? { ...isDraggingFromPool, duration: c.duration } : c);
            }
            const afterDur = targetClip.duration - beforeDur - newClipDur;
            for (let i = 0; i < prev.length; i++) {
                if (i === targetIdx) {
                    if (beforeDur > 0.05) next.push({ ...targetClip, duration: beforeDur });
                    next.push({ ...isDraggingFromPool, duration: newClipDur });
                    if (afterDur > 0.05) next.push({ ...targetClip, duration: afterDur });
                } else {
                    next.push(prev[i]);
                }
            }
            return next;
        });
        dragPoolItemRef.current = null;
        setIsDraggingFromPool(null);
        setDraggingHoverTrack(null);
        setDropPreview(null);
    }, [isDraggingFromPool]);

    const handleTriggerDrop = useCallback((dropTime: number) => {
        if (!isDraggingFromPool) return;
        
        const newTrigger: MontageTrigger = {
            phrase: isDraggingFromPool.path.split(/[\\/]/).pop()?.split('.')[0] || "Trigger",
            path: isDraggingFromPool.path,
            startTime: Math.max(0, dropTime),
            duration: isDraggingFromPool.actualDuration && isDraggingFromPool.actualDuration > 0 ? isDraggingFromPool.actualDuration : 3.0,
            isVideo: isDraggingFromPool.isVideo,
            x: 0,
            y: 0,
            w: plan?.baseW || 1920,
            h: plan?.baseH || 1080
        };

        setTriggers(prev => {
            const nt = [...prev, newTrigger];
            setSelectedTriggerIdx(nt.length - 1);
            setSelectedWatermarkIdx(null);
            setActiveInfoTab('stats');
            return nt;
        });
        dragPoolItemRef.current = null;
        setIsDraggingFromPool(null);
        setDraggingHoverTrack(null);
        setDropPreview(null);
    }, [isDraggingFromPool, plan]);

    const handleWatermarkDrop = useCallback((dropTime: number, trackId?: string) => {
        if (!isDraggingFromPool) return;
        const newW: MontageWatermark = {
            id: Math.random().toString(36).substr(2, 9),
            path: isDraggingFromPool.path,
            startTime: Math.max(0, dropTime),
            duration: 5.0,
            x: 50, y: 50, w: 200, h: 200,
            opacity: 1.0,
            trackId,
            isVideo: isDraggingFromPool.isVideo
        };
        setWatermarks(prev => {
            const nw = [...prev, newW];
            setSelectedWatermarkIdx(nw.length - 1);
            setSelectedTriggerIdx(null);
            setActiveInfoTab('stats');
            return nw;
        });
        dragPoolItemRef.current = null;
        setIsDraggingFromPool(null);
        setDraggingHoverTrack(null);
        setDropPreview(null);
    }, [isDraggingFromPool]);

    const handleAddTrack = useCallback(() => {
        const id = 'row_' + Math.random().toString(36).substr(2, 5);
        const colors = ['#3b82f6', '#a855f7', '#ec4899', '#10b981', '#f59e0b', '#ef4444'];
        const color = colors[extraTracks.length % colors.length];
        setExtraTracks(prev => [...prev, {
            id,
            name: `Row ${prev.length + 1}`,
            color,
            type: 'image'
        }]);
    }, [extraTracks]);

    const handleRemoveTrack = useCallback((tid: string) => {
        setExtraTracks(prev => prev.filter(t => t.id !== tid));
        setWatermarks(prev => prev.filter(w => w.trackId !== tid));
    }, []);

    useEffect(() => {
        if (isDraggingFromPool) {
            document.body.style.cursor = 'grabbing';
            document.body.style.userSelect = 'none';
        } else {
            document.body.style.cursor = '';
            document.body.style.userSelect = '';
        }
        return () => { document.body.style.cursor = ''; document.body.style.userSelect = ''; };
    }, [isDraggingFromPool]);

    // Mouse-based drag-and-drop (stable, works in WKWebView where HTML5 drop event doesn't fire)
    useEffect(() => {
        const findDropTarget = (x: number, y: number): HTMLElement | null => {
            let el = document.elementFromPoint(x, y) as HTMLElement | null;
            while (el) {
                if (el.dataset?.droptarget) return el;
                el = el.parentElement;
            }
            return null;
        };

        const getDropTime = (clientX: number): number => {
            const container = containerRef.current;
            if (!container) return 0;
            const rect = container.getBoundingClientRect();
            const x = clientX - rect.left + container.scrollLeft;
            return Math.max(0, (x - introWidthRef.current) / zoomRef.current);
        };

        const onMouseMove = (e: MouseEvent) => {
            if (!dragPoolItemRef.current) return;
            setDragPos({ x: e.clientX, y: e.clientY });
            const target = findDropTarget(e.clientX, e.clientY);
            if (!target) { setDraggingHoverTrack(null); setDropPreview(null); return; }
            const dt = target.dataset.droptarget!;
            if (dt === 'intro') {
                setDraggingHoverTrack('intro'); setDropPreview(0);
            } else if (dt === 'clips') {
                setDraggingHoverTrack('clips'); setDropPreview(getDropTime(e.clientX));
            } else if (dt === 'triggers') {
                setDraggingHoverTrack('triggers'); setDropPreview(getDropTime(e.clientX));
            } else if (dt === 'watermarks') {
                setDraggingHoverTrack('watermarks'); setDropPreview(getDropTime(e.clientX));
            } else if (dt === 'extra') {
                setDraggingHoverTrack(target.dataset.trackid || null); setDropPreview(getDropTime(e.clientX));
            } else {
                setDraggingHoverTrack(null); setDropPreview(null);
            }
        };

        const onMouseUp = (e: MouseEvent) => {
            const dragItem = dragPoolItemRef.current;
            if (!dragItem) return;
            dragPoolItemRef.current = null;
            setIsDraggingFromPool(null);
            setDragPos(null);
            setDraggingHoverTrack(null);
            setDropPreview(null);

            const target = findDropTarget(e.clientX, e.clientY);
            if (!target) return;
            const dt = target.dataset.droptarget!;
            const dropTime = getDropTime(e.clientX);

            if (dt === 'intro' || (dt === 'clips' && dropTime < 0.2)) {
                setIntroVideo({ ...dragItem, duration: dragItem.actualDuration || 3.0 });
            } else if (dt === 'clips') {
                setClips(prev => {
                    let currentTimePos = introVideoRef.current ? introVideoRef.current.duration : 0;
                    let targetIdx = -1;
                    for (let i = 0; i < prev.length; i++) {
                        if (dropTime >= currentTimePos && dropTime < currentTimePos + prev[i].duration) { targetIdx = i; break; }
                        currentTimePos += prev[i].duration;
                    }
                    if (targetIdx === -1) return prev;
                    const targetClip = prev[targetIdx];
                    const targetStart = (introVideoRef.current ? introVideoRef.current.duration : 0) + prev.slice(0, targetIdx).reduce((s, c) => s + c.duration, 0);
                    const beforeDur = dropTime - targetStart;
                    const newClipDur = Math.min(2.0, targetClip.duration - 0.2);
                    if (newClipDur <= 0.1) return prev.map((c, i) => i === targetIdx ? { ...dragItem, duration: c.duration } : c);
                    const afterDur = targetClip.duration - beforeDur - newClipDur;
                    const next: MontageClip[] = [];
                    for (let i = 0; i < prev.length; i++) {
                        if (i === targetIdx) {
                            if (beforeDur > 0.05) next.push({ ...targetClip, duration: beforeDur });
                            next.push({ ...dragItem, duration: newClipDur });
                            if (afterDur > 0.05) next.push({ ...targetClip, duration: afterDur });
                        } else { next.push(prev[i]); }
                    }
                    return next;
                });
            } else if (dt === 'triggers') {
                setTriggers(prev => {
                    const nt = [...prev, {
                        phrase: dragItem.path.split(/[\\/]/).pop()?.split('.')[0] || 'Trigger',
                        path: dragItem.path,
                        startTime: dropTime,
                        duration: dragItem.actualDuration && dragItem.actualDuration > 0 ? dragItem.actualDuration : 3.0,
                        isVideo: dragItem.isVideo,
                        x: 50, y: 50, w: 200, h: 200,
                    }];
                    setSelectedTriggerIdx(nt.length - 1);
                    setSelectedWatermarkIdx(null);
                    setActiveInfoTab('stats');
                    return nt;
                });
            } else if (dt === 'watermarks' || dt === 'extra') {
                const trackId = dt === 'extra' ? target.dataset.trackid : undefined;
                setWatermarks(prev => {
                    const nw = [...prev, {
                        id: Math.random().toString(36).substr(2, 9),
                        path: dragItem.path,
                        startTime: dropTime,
                        duration: 5.0,
                        x: 50, y: 50, w: 200, h: 200,
                        opacity: 1.0,
                        trackId,
                        isVideo: dragItem.isVideo,
                    }];
                    setSelectedWatermarkIdx(nw.length - 1);
                    setSelectedTriggerIdx(null);
                    setActiveInfoTab('stats');
                    return nw;
                });
            }
        };

        window.addEventListener('mousemove', onMouseMove);
        window.addEventListener('mouseup', onMouseUp);
        return () => {
            window.removeEventListener('mousemove', onMouseMove);
            window.removeEventListener('mouseup', onMouseUp);
        };
    }, []);

    const handleConfirm = useCallback(() => {
        const finalPlan = {
            ...plan,
            clips,
            triggers,
            watermarks,
            audioSegments,
            extraTracks
        };
        onConfirm(JSON.stringify(finalPlan), localSettings);
    }, [plan, clips, triggers, watermarks, audioSegments, extraTracks, localSettings, onConfirm]);

    useEffect(() => {
        const kd = (e: KeyboardEvent) => {
            if (e.target instanceof HTMLInputElement) return;
            if (e.key === '[') handleMarkIn();
            if (e.key === ']') handleMarkOut();
            if (e.key === 'Backspace' || e.key === 'Delete') {
                if (selection.start !== null) handleCutSelection();
            }
            if (e.key === ' ') { e.preventDefault(); handleTogglePlay(); }
        };
        window.addEventListener('keydown', kd);
        return () => window.removeEventListener('keydown', kd);
    }, [handleMarkIn, handleMarkOut, handleCutSelection, handleTogglePlay, selection.start]);

    const handleResizeMouseDown = (e: React.MouseEvent) => {
        e.preventDefault();
        const mm = (me: MouseEvent) => {
            const h = Math.max(150, Math.min(600, window.innerHeight - me.clientY - 120));
            setTimelineHeight(h);
            localStorage.setItem('montage-timeline-height', h.toString());
        };
        const mu = () => { document.removeEventListener('mousemove', mm); document.removeEventListener('mouseup', mu); };
        document.addEventListener('mousemove', mm);
        document.addEventListener('mouseup', mu);
    };

    const handleInfoResizeMouseDown = (e: React.MouseEvent) => {
        e.preventDefault();
        const startWidth = infoPanelWidth;
        const startX = e.clientX;
        const mm = (me: MouseEvent) => {
            const deltaX = startX - me.clientX;
            const w = Math.max(250, Math.min(800, startWidth + deltaX));
            setInfoPanelWidth(w);
            localStorage.setItem('montage-info-width', w.toString());
        };
        const mu = () => { document.removeEventListener('mousemove', mm); document.removeEventListener('mouseup', mu); };
        document.addEventListener('mousemove', mm);
        document.addEventListener('mouseup', mu);
    };

    const handleMouseDownGlobal = (e: React.MouseEvent) => {
        setIsScrubbing(true);
        handleTimelineMove(e);
    };

    useEffect(() => {
        const mu = handleMouseUp;
        const mm = (e: MouseEvent) => {
            if (draggingIdx !== null) {
                const deltaD = (e.clientX - startX) / zoom;
                let cD = startDurations.current + deltaD;
                let nD = startDurations.next - deltaD;
                if (cD < 0.5) { cD = 0.5; nD = startDurations.current + startDurations.next - 0.5; }
                else if (nD < 0.5) { nD = 0.5; cD = startDurations.current + startDurations.next - 0.5; }
                setClips(p => { const nc = [...p]; nc[draggingIdx] = { ...nc[draggingIdx], duration: cD }; nc[draggingIdx + 1] = { ...nc[draggingIdx + 1], duration: nD }; return nc; });
            } else if (draggingWatermarkSide !== null && draggingWatermarkIdx !== null) {
                const deltaT = (e.clientX - startX) / zoom;
                setWatermarks(p => {
                    const nw = [...p];
                    const w = nw[draggingWatermarkIdx];
                    if (draggingWatermarkSide === 'start') {
                        const newStart = Math.max(0, dragWatermarkStartPos + deltaT);
                        const newDur = Math.max(0.2, dragWatermarkStartDur - (newStart - dragWatermarkStartPos));
                        nw[draggingWatermarkIdx] = { ...w, startTime: newStart, duration: newDur };
                    } else {
                        const newDur = Math.max(0.2, dragWatermarkStartDur + deltaT);
                        nw[draggingWatermarkIdx] = { ...w, duration: newDur };
                    }
                    return nw;
                });
            } else if (draggingWatermarkIdx !== null) {
                const deltaT = (e.clientX - startX) / zoom;
                let newTime = Math.max(0, dragWatermarkStartPos + deltaT);
                setWatermarks(p => {
                    const nw = [...p];
                    nw[draggingWatermarkIdx] = { ...nw[draggingWatermarkIdx], startTime: newTime };
                    return nw;
                });
            } else if (draggingTriggerSide !== null && draggingTriggerIdx !== null) {
                const deltaT = (e.clientX - startX) / zoom;
                setTriggers(p => {
                    const nt = [...p];
                    const tr = nt[draggingTriggerIdx];
                    if (draggingTriggerSide === 'start') {
                        const newStart = Math.max(0, dragTriggerStartPos + deltaT);
                        const newDur = Math.max(0.2, dragTriggerStartDur - (newStart - dragTriggerStartPos));
                        nt[draggingTriggerIdx] = { ...tr, startTime: newStart, duration: newDur };
                    } else {
                        const newDur = Math.max(0.2, dragTriggerStartDur + deltaT);
                        nt[draggingTriggerIdx] = { ...tr, duration: newDur };
                    }
                    return nt;
                });
            } else if (draggingTriggerIdx !== null) {
                const deltaT = (e.clientX - startX) / zoom;
                let newTime = Math.max(0, dragTriggerStartPos + deltaT);
                setTriggers(p => {
                    const nt = [...p];
                    nt[draggingTriggerIdx] = { ...nt[draggingTriggerIdx], startTime: newTime };
                    return nt;
                });
            } else if (draggingTriggerPosIdx !== null && dragStartCoords !== null) {
                const dx = (e.clientX - dragStartCoords.mouseX);
                const dy = (e.clientY - dragStartCoords.mouseY);
                if (!hasMovedRef.current && Math.abs(dx) < 3 && Math.abs(dy) < 3) return;
                hasMovedRef.current = true;
                
                const rect = previewWrapRef.current?.getBoundingClientRect();
                if (rect && plan?.baseW && plan?.baseH) {
                    const scaleX = plan.baseW / rect.width;
                    const scaleY = plan.baseH / rect.height;
                    setTriggers(prev => {
                        const nt = [...prev];
                        nt[draggingTriggerPosIdx] = {
                            ...nt[draggingTriggerPosIdx],
                            x: Math.round(dragStartCoords.x + dx * scaleX),
                            y: Math.round(dragStartCoords.y + dy * scaleY)
                        };
                        return nt;
                    });
                }
            } else if (draggingWatermarkPosIdx !== null && dragStartCoords !== null) {
                const dx = (e.clientX - dragStartCoords.mouseX);
                const dy = (e.clientY - dragStartCoords.mouseY);
                if (!hasMovedRef.current && Math.abs(dx) < 3 && Math.abs(dy) < 3) return;
                hasMovedRef.current = true;

                const rect = previewWrapRef.current?.getBoundingClientRect();
                if (rect && plan?.baseW && plan?.baseH) {
                    const scaleX = plan.baseW / rect.width;
                    const scaleY = plan.baseH / rect.height;
                    setWatermarks(prev => {
                        const nw = [...prev];
                        nw[draggingWatermarkPosIdx] = {
                            ...nw[draggingWatermarkPosIdx],
                            x: Math.round(dragStartCoords.x + dx * scaleX),
                            y: Math.round(dragStartCoords.y + dy * scaleY)
                        };
                        return nw;
                    });
                }
            } else if (resizingWatermarkIdx !== null && dragStartCoords !== null && resizingWatermarkHandle) {
                const dx = (e.clientX - dragStartCoords.mouseX);
                const dy = (e.clientY - dragStartCoords.mouseY);
                const rect = previewWrapRef.current?.getBoundingClientRect();
                if (rect && plan?.baseW && plan?.baseH) {
                    const scaleX = plan.baseW / rect.width;
                    const scaleY = plan.baseH / rect.height;
                    setWatermarks(prev => {
                        const nw = [...prev];
                        const w = nw[resizingWatermarkIdx];
                        if (resizingWatermarkHandle === 'br') {
                            nw[resizingWatermarkIdx] = {
                                ...w,
                                w: Math.max(20, Math.round(dragStartCoords.x + dx * scaleX)),
                                h: Math.max(20, Math.round(dragStartCoords.y + dy * scaleY))
                            };
                        } else if (resizingWatermarkHandle === 'tl') {
                            const newW = Math.max(20, Math.round(dragStartCoords.x - dx * scaleX));
                            const newH = Math.max(20, Math.round(dragStartCoords.y - dy * scaleY));
                            const dxActual = dragStartCoords.x - newW;
                            const dyActual = dragStartCoords.y - newH;
                            nw[resizingWatermarkIdx] = {
                                ...w,
                                x: Math.round((dragStartCoords.x2 || 0) + dxActual),
                                y: Math.round((dragStartCoords.y2 || 0) + dyActual),
                                w: newW,
                                h: newH
                            };
                        }
                        return nw;
                    });
                }
            } else if (draggingSelectionSide !== null && containerRef.current) {
                const rect = containerRef.current.getBoundingClientRect();
                const xRaw = e.clientX - rect.left + containerRef.current.scrollLeft;
                let targetTime = introVideo ? xRaw / zoom : (xRaw - introWidth) / zoom;
                const newTime = Math.max(0, Math.min(animStateRef.current.totalDuration, targetTime));
                setSelection(prev => ({ ...prev, [draggingSelectionSide]: newTime }));
            }
            if (isScrubbing) handleTimelineMove(e);
        };
        if (draggingIdx !== null || isScrubbing || draggingSelectionSide !== null || draggingTriggerIdx !== null || draggingTriggerSide !== null || draggingTriggerPosIdx !== null || draggingWatermarkIdx !== null || draggingWatermarkPosIdx !== null || resizingWatermarkIdx !== null) {
            document.addEventListener('mousemove', mm);
            document.addEventListener('mouseup', mu);
        }
        return () => { document.removeEventListener('mousemove', mm); document.removeEventListener('mouseup', mu); };
    }, [draggingIdx, isScrubbing, draggingSelectionSide, draggingTriggerIdx, draggingTriggerSide, draggingTriggerPosIdx, draggingWatermarkIdx, draggingWatermarkPosIdx, resizingWatermarkIdx, dragStartCoords, handleTimelineMove, zoom, startX, startDurations, dragTriggerStartPos, dragTriggerStartDur, dragWatermarkStartPos, dragWatermarkStartDur, resizingWatermarkHandle, draggingWatermarkSide, handleMouseUp, plan, introWidth]);

    const markers = useMemo(() => {
        const res = [];
        const contentDuration = totalTimelineDuration;
        const introDur = effectiveIntroDuration;
        
        // Total count including intro time if present
        const totalCount = Math.ceil(contentDuration + introDur);
        
        for (let i = 0; i <= totalCount; i++) {
            // Label should be shifted if intro exists
            let label = i;
            if (introDur > 0) {
                label = i; // Total timeline time
            } else {
                label = i; // Regular time
            }

            // The position 'i' is in seconds relative to the very start of things
            const xPos = i * zoom;
            
            if (zoom > 50 || i % 5 === 0) {
                res.push(<div key={i} className="timeline-marker" style={{ left: `${xPos}px` }}><span>{label}s</span></div>);
            } else {
                res.push(<div key={i} className="timeline-marker minor" style={{ left: `${xPos}px` }} />);
            }
        }
        return res;
    }, [totalTimelineDuration, zoom, effectiveIntroDuration]);

    const clipElements = useMemo(() => {
        return clipLayouts.map(({ clip, idx, width, x }) => (
            <div 
                key={idx} 
                className={`montage-clip-block ${clip.isVideo ? 'video' : 'image'} ${activeClipInfo?.idx === idx ? 'active-preview' : ''} ${regeneratingIndices.has(idx) ? 'is-regenerating' : ''}`} 
                style={{ left: `${x}px`, width: `${width}px` }}
            >
                <div className="montage-clip-content">
                    {regeneratingIndices.has(idx) ? (
                        <div className="clip-loading-spinner"><div className="spinner-tiny" /></div>
                    ) : (
                        <div className="montage-clip-thumbnail-placeholder">{clip.isVideo ? '🎬' : '🖼️'}</div>
                    )}
                    <span className="montage-clip-name">{clip.path.split(/[\\/]/).pop()}</span>
                    <span className="montage-clip-duration">{clip.duration.toFixed(1)}s</span>
                    
                    <div className="montage-clip-actions">
                        <button className="clip-action-btn delete" onMouseDown={(e) => { e.stopPropagation(); handleDeleteClip(idx); }}>🗑️</button>
                        <button className="clip-action-btn regenerate" onMouseDown={(e) => { e.stopPropagation(); handleOpenRegenerate(idx); }}>🔄</button>
                        {!clip.isVideo && (
                            <button className="clip-action-btn animate" onMouseDown={(e) => { e.stopPropagation(); handleAnimateClip(idx); }} title="Animate">
                                <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polygon points="23 7 16 12 23 17 23 7"/><rect x="1" y="5" width="15" height="14" rx="2" ry="2"/></svg>
                            </button>
                        )}
                    </div>
                </div>
                {idx < clips.length - 1 && (
                    <div className="montage-clip-resizer right" onMouseDown={(e) => { e.stopPropagation(); setDraggingIdx(idx); setStartX(e.clientX); setStartDurations({ current: clips[idx].duration, next: clips[idx + 1].duration }); }} />
                )}
            </div>
        ));
    }, [clipLayouts, activeClipInfo?.idx, clips, regeneratingIndices, handleDeleteClip, handleOpenRegenerate, handleAnimateClip]);

    const triggerElements = useMemo(() => {
        return triggers.map((tr, i) => {
            const width = Math.max(tr.duration * zoom, 40);
            const x = tr.startTime * zoom;
            const isActive = currentTime >= tr.startTime && currentTime <= tr.startTime + tr.duration;
            return (
                <div 
                    key={i} 
                    className={`montage-trigger-marker ${isActive ? 'active' : ''} ${draggingTriggerIdx === i ? 'dragging' : ''}`}
                    style={{ left: `${x}px`, width: `${width}px` }}
                    title={`Trigger: ${tr.phrase} (${tr.path.split(/[\\/]/).pop()}) - Click to jump, Drag to move`}
                    onMouseDown={(e) => {
                        e.preventDefault();
                        e.stopPropagation();
                        setDraggingTriggerIdx(i);
                        setStartX(e.clientX);
                        setDragTriggerStartPos(tr.startTime);
                    }}
                    onClick={() => setCurrentTime(tr.startTime)}
                >
                    <div className="trigger-icon">🎯</div>
                    <div className="trigger-phrase" onDoubleClick={(e) => {
                        e.stopPropagation();
                        setEditingTriggerIdx(i);
                        setTempTriggerPhrase(tr.phrase);
                    }}>{tr.phrase}</div>
                    <button className="trigger-delete-btn" onClick={(e) => { e.stopPropagation(); handleDeleteTrigger(i); }}>✕</button>
                </div>
            );
        });
    }, [triggers, zoom, currentTime, draggingTriggerIdx, handleDeleteTrigger]);

    if (!plan) return null;

    return (
        <div className="montage-editor-overlay animate-fade">
            <div className="montage-editor-window">
                <div className="montage-editor-header">
                    <div className="montage-editor-title"></div>
                    <div className="montage-editor-controls">
                        <button className="montage-btn icon" onClick={() => setZoom(p => Math.max(p - (p > 50 ? 10 : 5), 5))}>-</button>
                        <span className="montage-zoom-label">{zoom}%</span>
                        <button className="montage-btn icon" onClick={() => setZoom(p => Math.min(p + (p < 50 ? 5 : 10), 400))}>+</button>
                    </div>
                </div>
                <div className="montage-editor-body">
                    <div className="montage-preview-and-info">
                        <div className="montage-preview-container">
                            <div className="montage-preview-wrap">
                                {activeClipInfo ? (
                                    <div 
                                        className="preview-media-wrapper" 
                                        ref={previewWrapRef}
                                        style={{ aspectRatio: `${previewAspect}` }}
                                        onMouseDown={(e) => {
                                            if (e.target === e.currentTarget || (e.target as HTMLElement).tagName === 'VIDEO' || (e.target as HTMLElement).tagName === 'IMG') {
                                                setSelectedWatermarkIdx(null);
                                                setSelectedTriggerIdx(null);
                                                setActiveInfoTab('library');
                                            }
                                        }}
                                    >
                                        {activeClipInfo.clip.isVideo ? (
                                            <video
                                                ref={previewVideoRef}
                                                src={getUrl(activeClipInfo.clip.path)}
                                                playsInline
                                                style={{ background: '#000' }}
                                                onLoadedMetadata={(e) => {
                                                    const v = e.currentTarget;
                                                    if (v.videoWidth && v.videoHeight) setPreviewAspect(v.videoWidth / v.videoHeight);
                                                }}
                                            />
                                        ) : (
                                            <img
                                                src={getUrl(activeClipInfo.clip.path)}
                                                alt="Preview"
                                                onLoad={(e) => {
                                                    const img = e.currentTarget;
                                                    if (img.naturalWidth && img.naturalHeight) setPreviewAspect(img.naturalWidth / img.naturalHeight);
                                                }}
                                            />
                                        )}
                                        <div className="preview-timestamp">{currentTime.toFixed(2)}s</div>
                                        {plan.audioPath && <audio ref={previewAudioRef} src={getUrl(plan.audioPath)} style={{ display: 'none' }} />}

                                        {currentSubtitle && (
                                            <div className="preview-subtitle-overlay animate-fade">
                                                {currentSubtitle.text}
                                            </div>
                                        )}

                                        {activeTriggersAtTime.map((tr) => (
                                            <div 
                                                key={`tr_${tr.index}`}
                                                className={`preview-trigger-overlay ${draggingTriggerPosIdx === tr.index ? 'dragging' : ''}`}
                                                style={{
                                                    left: `${(tr.x / (plan.baseW || 1920)) * 100}%`,
                                                    top: `${(tr.y / (plan.baseH || 1080)) * 100}%`,
                                                    width: `${(tr.w / (plan.baseW || 1920)) * 100}%`,
                                                    height: `${(tr.h / (plan.baseH || 1080)) * 100}%`,
                                                }}
                                                onMouseDown={(e) => {
                                                    e.stopPropagation();
                                                    setSelectedTriggerIdx(tr.index);
                                                    setSelectedWatermarkIdx(null);
                                                    setActiveInfoTab('stats');
                                                    setDraggingTriggerPosIdx(tr.index);
                                                    setDragStartCoords({ x: tr.x, y: tr.y, mouseX: e.clientX, mouseY: e.clientY });
                                                }}
                                            >
                                                <div className="trigger-overlay-handle">🎯</div>
                                                <div className="trigger-overlay-label">{tr.phrase}</div>
                                                <button className="preview-delete-btn" title="Delete Trigger" onMouseDown={(e) => { e.stopPropagation(); handleDeleteTrigger(tr.index); setSelectedTriggerIdx(null); }}>✕</button>
                                            </div>
                                        ))}

                                        {activeWatermarksAtTime.filter((w: MontageWatermark) => w.trackId !== 'auto-gen-overlay').map((w: MontageWatermark) => (
                                            <WatermarkPreviewItem
                                                key={w.id}
                                                w={w}
                                                plan={plan!}
                                                selectedWatermarkIdx={selectedWatermarkIdx}
                                                draggingWatermarkPosIdx={draggingWatermarkPosIdx}
                                                setSelectedWatermarkIdx={setSelectedWatermarkIdx}
                                                setSelectedTriggerIdx={setSelectedTriggerIdx}
                                                setActiveInfoTab={setActiveInfoTab}
                                                setDraggingWatermarkPosIdx={setDraggingWatermarkPosIdx}
                                                setDragStartCoords={setDragStartCoords}
                                                setResizingWatermarkIdx={setResizingWatermarkIdx}
                                                setResizingWatermarkHandle={setResizingWatermarkHandle}
                                                getUrl={getUrl}
                                                adjustedTime={introVideo ? currentTime - effectiveIntroDuration : currentTime}
                                                isPlaying={isPlaying}
                                                handleDeleteWatermark={handleDeleteWatermark}
                                                onDimensionsLoad={handleWatermarkDimensionsLoad}
                                            />
                                        ))}
                                    </div>
                                ) : (
                                    <div className="preview-media-wrapper no-clip">
                                        <div className="no-clip-message">No clip selected</div>
                                    </div>
                                )}
                            </div>
                        </div>
                        <div className="montage-info-resizer-v" onMouseDown={handleInfoResizeMouseDown}><div className="resizer-handle-v-line"></div></div>
                        <div className="montage-info-panel" style={{ width: `${infoPanelWidth}px`, flex: 'none' }}>
                            <div className="info-tabs">
                                <button className={`info-tab ${activeInfoTab === 'library' ? 'active' : ''}`} onClick={() => setActiveInfoTab('library')}>Library</button>
                                <button className={`info-tab ${activeInfoTab === 'stats' ? 'active' : ''}`} onClick={() => setActiveInfoTab('stats')}>Properties</button>
                            </div>
                            <div 
                                className={`media-pool-container ${isDraggingExternal ? 'dragging-external' : ''}`} 
                                ref={poolRef}
                                onDragEnter={(e) => { if (e.dataTransfer.types.includes('Files')) { e.preventDefault(); dragCounter.current++; setIsDraggingExternal(true); } }}
                                onDragOver={(e) => { if (e.dataTransfer.types.includes('Files')) { e.preventDefault(); } }}
                                onDragLeave={(e) => { e.preventDefault(); dragCounter.current--; if (dragCounter.current <= 0) { dragCounter.current = 0; setIsDraggingExternal(false); } }}
                                onDrop={(e) => { e.preventDefault(); dragCounter.current = 0; setIsDraggingExternal(false); }}
                            >
                                {isDraggingExternal && (
                                    <div className="media-pool-import-overlay">
                                        <div className="import-overlay-content">
                                            <div className="import-icon">📥</div>
                                            <div className="import-text">Drop to Import</div>
                                        </div>
                                    </div>
                                )}
                                {activeInfoTab === 'library' ? (
                                    <>
                                        <div className="media-library-header-compact"><h3 className="media-library-title">Assets</h3></div>
                                        {mediaPool.length > 0 ? (
                                            <div className="media-pool-grid">
                                                {mediaPool.map((m, i) => (
                                                    <div
                                                        key={i}
                                                        className="pool-item"
                                                        onMouseDown={(e) => { e.preventDefault(); console.log('[DND] mouseDown pool', m.path); dragPoolItemRef.current = m; setIsDraggingFromPool(m); setDragPos({ x: e.clientX, y: e.clientY }); setDraggingHoverTrack(null); }}
                                                        onMouseEnter={() => setHoveredMediaIdx(i)}
                                                        onMouseLeave={() => setHoveredMediaIdx(null)}
                                                        onClick={() => setFullscreenPoolIdx(i)}
                                                        title={m.path.split(/[\\/]/).pop()}
                                                    >
                                                        <button className="pool-item-delete" onClick={(e) => { e.stopPropagation(); handleRemoveFromPool(i); }} title="Remove">✕</button>
                                                        <div className="pool-thumb-wrapper">
                                                            {m.isVideo ? (
                                                                <video
                                                                    key={`${m.path}-${hoveredMediaIdx === i}`}
                                                                    src={getUrl(m.path)}
                                                                    className={`pool-thumb-img ${hoveredMediaIdx === i ? 'video-preview' : ''}`}
                                                                    autoPlay={hoveredMediaIdx === i}
                                                                    muted
                                                                    loop={hoveredMediaIdx === i}
                                                                    playsInline
                                                                    preload="metadata"
                                                                    draggable={false}
                                                                />
                                                            ) : (
                                                                <img src={getUrl(m.path)} alt="thumb" className="pool-thumb-img" draggable={false} />
                                                            )}
                                                            {m.isVideo && hoveredMediaIdx !== i && <div className="pool-video-overlay">🎬</div>}
                                                        </div>
                                                        <div className="pool-dur">{m.duration.toFixed(1)}s</div>
                                                        {m.source && (
                                                            <div className={`pool-source-badge pool-source-${m.source}`}>
                                                                {m.source === 'footage' ? 'Футаж' : 'AI'}
                                                            </div>
                                                        )}
                                                    </div>
                                                ))}
                                            </div>
                                        ) : (
                                            <div className="pool-empty-state"><div className="empty-icon">📁</div><p>Empty</p><button className="add-files-btn-center" onClick={handleAddMedia}>Add Files</button></div>
                                        )}
                                        <div className="pool-section-divider" />
                                        <div className="timeline-pool-section">
                                            <div className="media-library-header-compact"><h3 className="media-library-title">Timeline</h3></div>
                                            <div className="media-pool-grid">
                                                {clips.map((clip, i) => (
                                                    <div
                                                        key={i}
                                                        className={`pool-item timeline-pool-item`}
                                                        onMouseDown={(e) => { e.preventDefault(); dragPoolItemRef.current = clip; setIsDraggingFromPool(clip); setDragPos({ x: e.clientX, y: e.clientY }); setDraggingHoverTrack(null); }}
                                                        onMouseEnter={() => setHoveredTimelinePoolIdx(i)}
                                                        onMouseLeave={() => setHoveredTimelinePoolIdx(null)}
                                                        onClick={() => setFullscreenClipIdx(i)}
                                                        title={clip.path.split(/[\\/]/).pop()}
                                                    >
                                                        {regeneratingIndices.has(i) && (
                                                            <div className="clip-loading-spinner"><div className="spinner-tiny" /></div>
                                                        )}
                                                        <div className="pool-thumb-wrapper">
                                                            {clip.isVideo ? (
                                                                <video
                                                                    key={`tl-${clip.path}`}
                                                                    src={getUrl(clip.path)}
                                                                    className="pool-thumb-img"
                                                                    muted
                                                                    loop
                                                                    playsInline
                                                                    preload="metadata"
                                                                    draggable={false}
                                                                    onLoadedMetadata={(e) => { (e.target as HTMLVideoElement).currentTime = 0.1; }}
                                                                    onMouseEnter={(e) => (e.currentTarget as HTMLVideoElement).play().catch(() => {})}
                                                                    onMouseLeave={(e) => { (e.currentTarget as HTMLVideoElement).pause(); (e.currentTarget as HTMLVideoElement).currentTime = 0.1; }}
                                                                />
                                                            ) : (
                                                                <img src={getUrl(clip.path)} alt="thumb" className="pool-thumb-img" draggable={false} />
                                                            )}
                                                            <div className="pool-type-badge">
                                                                {clip.isVideo
                                                                    ? <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5"><polygon points="23 7 16 12 23 17 23 7"/><rect x="1" y="5" width="15" height="14" rx="2"/></svg>
                                                                    : <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>
                                                                }
                                                            </div>
                                                        </div>
                                                        <div className="pool-dur">{clip.duration.toFixed(1)}s</div>
                                                        <div className="pool-item-actions-row">
                                                            <button className="pool-action-btn" onClick={(e) => { e.stopPropagation(); handleDeleteClip(i); }} title="Delete">🗑️</button>
                                                            <button className="pool-action-btn" onClick={(e) => { e.stopPropagation(); handleOpenRegenerate(i); }} title="Regenerate">🔄</button>
                                                            {!clip.isVideo && (
                                                                <button className="pool-action-btn animate" onClick={(e) => { e.stopPropagation(); handleAnimateClip(i); }} title="Animate">
                                                                    <svg width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polygon points="23 7 16 12 23 17 23 7"/><rect x="1" y="5" width="15" height="14" rx="2" ry="2"/></svg>
                                                                </button>
                                                            )}
                                                        </div>
                                                    </div>
                                                ))}
                                            </div>
                                        </div>
                                    </>
                                ) : (
                                    <div className="project-stats-tab animate-fade-in">
                                        {selectedWatermarkIdx !== null ? (
                                            <div className="properties-panel">
                                                <h4>{watermarks[selectedWatermarkIdx]?.isVideo ? 'Video Overlay' : 'Image Overlay'}</h4>
                                                {!watermarks[selectedWatermarkIdx]?.isVideo && (
                                                    <div className="prop-group">
                                                        <label>Opacity: {Math.round(watermarks[selectedWatermarkIdx]?.opacity * 100)}%</label>
                                                        <input 
                                                            type="range" 
                                                            min="0" 
                                                            max="1" 
                                                            step="0.01" 
                                                            value={watermarks[selectedWatermarkIdx]?.opacity || 1} 
                                                            onChange={(e) => {
                                                                const val = parseFloat(e.target.value);
                                                                setWatermarks(prev => {
                                                                    const nw = [...prev];
                                                                    nw[selectedWatermarkIdx] = { ...nw[selectedWatermarkIdx], opacity: val };
                                                                    return nw;
                                                                });
                                                            }}
                                                        />
                                                    </div>
                                                )}
                                                <div className="prop-group">
                                                    <div className="prop-row">
                                                        <span>Position: {Math.round(watermarks[selectedWatermarkIdx]?.x)}, {Math.round(watermarks[selectedWatermarkIdx]?.y)}</span>
                                                    </div>
                                                    <div className="prop-row">
                                                        <span>Size: {Math.round(watermarks[selectedWatermarkIdx]?.w)}x{Math.round(watermarks[selectedWatermarkIdx]?.h)}</span>
                                                    </div>
                                                </div>
                                                <button className="prop-btn danger" onClick={() => { handleDeleteWatermark(selectedWatermarkIdx); setSelectedWatermarkIdx(null); }}>Delete</button>
                                            </div>
                                        ) : selectedTriggerIdx !== null ? (
                                            <div className="properties-panel">
                                                <h4>Trigger Properties</h4>
                                                <p>Phrase: <strong>{triggers[selectedTriggerIdx]?.phrase}</strong></p>
                                                <button className="prop-btn danger" onClick={() => { handleDeleteTrigger(selectedTriggerIdx); setSelectedTriggerIdx(null); }}>Delete Trigger</button>
                                            </div>
                                        ) : (
                                            <>
                                                <div className="stat-card"><div className="stat-label">Total Clips</div><div className="stat-value">{clips.length + (introVideo ? 1 : 0)}</div></div>
                                                <div className="stat-card"><div className="stat-label">Duration</div><div className="stat-value">{totalTimelineDuration.toFixed(2)}s</div></div>
                                                <div className="stat-card"><div className="stat-label">Audio Sync</div><div className="stat-value">{plan?.audioDuration.toFixed(2)}s</div></div>
                                                <div className="stat-card"><div className="stat-label">Transitions</div><div className="stat-value">{plan?.transDuration}s ({plan?.isFadeFast ? 'Fast' : 'Fade'})</div></div>
                                            </>
                                        )}
                                    </div>
                                )}
                            </div>
                        </div>
                    </div>
                    
                    <div className="playback-overlay-controls">
                        <div className="playback-controls">
                            <button className="play-btn" onClick={handleTogglePlay}>
                                {isPlaying ? <svg viewBox="0 0 24 24" width="24" height="24" fill="currentColor"><path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" /></svg> : <svg viewBox="0 0 24 24" width="24" height="24" fill="currentColor"><path d="M8 5v14l11-7z" /></svg>}
                            </button>
                            <div className="volume-control">
                                <input type="range" min="0" max="1" step="0.01" value={volume} onChange={(e) => setVolume(parseFloat(e.target.value))} />
                                <span className="volume-label">{Math.round(volume * 100)}%</span>
                            </div>
                        </div>
                        <div className="selection-controls-container">
                            <button className={`scissors-toggle ${isCuttingMode ? 'active' : ''}`} onClick={() => setIsCuttingMode(!isCuttingMode)} title="Trimming Tools">
                                <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="6" cy="6" r="3"></circle><circle cx="6" cy="18" r="3"></circle><line x1="20" y1="4" x2="8.12" y2="15.88"></line><line x1="14.47" y1="14.48" x2="20" y2="20"></line><line x1="8.12" y1="8.12" x2="12" y2="12"></line></svg>
                            </button>
                            {isCuttingMode && (
                                <div className="selection-controls-expanded animate-slide-in">
                                    <button className={`selection-btn ${selection.start !== null ? 'active' : ''}`} onClick={handleMarkIn}>Set Point 1</button>
                                    <button className={`selection-btn ${selection.end !== null ? 'active' : ''}`} onClick={handleMarkOut}>Set Point 2</button>
                                    <button className="selection-btn danger large" disabled={selection.start === null || selection.end === null} onClick={handleCutSelection}>CONFIRM & CUT</button>
                                    {(selection.start !== null || selection.end !== null) && <button className="selection-btn secondary" onClick={handleClearSelection}>Reset</button>}
                                </div>
                            )}
                        </div>
                    </div>

                    <div className="montage-timeline-resizer" onMouseDown={handleResizeMouseDown}><div className="resizer-handle-line"></div></div>
                    <div 
                        className={`montage-timeline-container ${isDraggingFromPool && draggingHoverTrack === 'clips' ? 'accepting-drop' : ''}`} 
                        style={{ height: `${timelineHeight}px`, flex: 'none' }}
                    >
                        <div className="montage-timeline-wrapper">

                            <div
                                className="montage-timeline-content"
                                ref={containerRef}
                                data-droptarget="clips"
                                onDragOver={(e) => {
                                    if (dragPoolItemRef.current || isDraggingFromPool) {
                                        e.preventDefault();
                                        const rect = e.currentTarget.getBoundingClientRect();
                                        const x = e.clientX - rect.left + e.currentTarget.scrollLeft;
                                        setDropPreview((x - introWidth) / zoom);
                                        setDraggingHoverTrack('clips');
                                    }
                                }}
                                onDragLeave={() => { setDropPreview(null); setDraggingHoverTrack(null); }}
                                onDrop={(e) => {
                                    console.log('[DND] drop on timeline-content, isDraggingFromPool=', !!isDraggingFromPool, 'ref=', !!dragPoolItemRef.current);
                                    if (isDraggingFromPool) {
                                        const rect = e.currentTarget.getBoundingClientRect();
                                        const x = e.clientX - rect.left + e.currentTarget.scrollLeft;
                                        handleInternalDrop((x - introWidth) / zoom);
                                        setDraggingHoverTrack(null);
                                    }
                                }}
                            >
                                <div className="montage-timeline-content-inner" style={{ width: `${totalTimelineDuration * zoom + 500}px`, minWidth: '100%' }}>
                                    <div className="montage-timeline-ruler" onMouseDown={handleMouseDownGlobal}>
                                        <div className="ruler-markers-container" style={{ marginLeft: introVideo ? 0 : `${introWidth}px` }}>
                                            {markers}
                                        </div>
                                        {selection.start !== null && <div className="selection-marker start interactive" style={{ left: `${selection.start * zoom + (introVideo ? 0 : introWidth)}px` }} onMouseDown={(e) => { e.stopPropagation(); setDraggingSelectionSide('start'); }}><div className="marker-handle" /></div>}
                                        {selection.end !== null && <div className="selection-marker end interactive" style={{ left: `${selection.end * zoom + (introVideo ? 0 : introWidth)}px` }} onMouseDown={(e) => { e.stopPropagation(); setDraggingSelectionSide('end'); }}><div className="marker-handle" /></div>}
                                        {selection.start !== null && selection.end !== null && <div className="selection-range" style={{ left: `${Math.min(selection.start, selection.end) * zoom + (introVideo ? 0 : introWidth)}px`, width: `${Math.abs(selection.end - selection.start) * zoom}px` }} />}
                                        {cutJunctions.map((j, i) => (<div key={i} className="timeline-cut-junction" style={{ left: `${j.position * zoom + (introVideo ? 0 : introWidth)}px` }}><div className="junction-icon">✂</div></div>))}
                                        
                                        {introVideo && (
                                            <div className="audio-track-reference intro-audio" style={{ left: 0, width: `${introWidth}px` }}>
                                                <span>Intro Audio</span>
                                            </div>
                                        )}
                                        
                                        <div className="audio-track-reference" style={{ left: `${introWidth}px`, width: `${(totalTimelineDuration - effectiveIntroDuration) * zoom}px` }} onMouseDown={handleMouseDownGlobal}>
                                            <span>Main Audio Sequence</span>
                                        </div>
                                    </div>

                                    <div className="montage-timeline-tracks">
                                        <div className="montage-track clips">
                                            <div
                                                className="intro-slot-container"
                                                style={{ width: `${introWidth}px` }}
                                                data-droptarget="intro"
                                                onDragOver={(e) => { if (dragPoolItemRef.current || isDraggingFromPool) { e.preventDefault(); e.stopPropagation(); setDropPreview(0); setDraggingHoverTrack('intro'); } }}
                                                onDragLeave={() => setDraggingHoverTrack(null)}
                                                onDrop={(e) => { if (isDraggingFromPool) { e.stopPropagation(); handleInternalDrop(0); setDraggingHoverTrack(null); } }}
                                            >
                                                {introVideo ? (
                                                    <div className={`montage-clip-block intro-selected ${isDraggingFromPool && draggingHoverTrack === 'intro' ? 'active' : ''}`} style={{ width: '100%' }}>
                                                        <div className="montage-clip-content">
                                                            <div className="montage-clip-thumbnail-placeholder">🚀</div>
                                                            <span className="montage-clip-name">{introVideo.path.split(/[\\/]/).pop()}</span>
                                                            <span className="montage-clip-duration">{introVideo.duration.toFixed(1)}s</span>
                                                            <div className="montage-clip-actions">
                                                                <button className="clip-action-btn delete" onMouseDown={(e) => { e.stopPropagation(); handleDeleteIntro(); }}>🗑️</button>
                                                            </div>
                                                        </div>
                                                    </div>
                                                ) : (
                                                    <div className={`intro-drop-slot ${isDraggingFromPool && draggingHoverTrack === 'intro' ? 'active' : ''}`}>
                                                        <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor"><path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z" /></svg>
                                                        <span>{t('montage_editor.drop_intro')}</span>
                                                    </div>
                                                )}
                                            </div>
                                            {clipLayouts.map(({ clip, idx, width, x }) => (
                                                <div 
                                                    key={idx} 
                                                    className={`montage-clip-block ${clip.isVideo ? 'video' : 'image'} ${activeClipInfo?.idx === idx ? 'active-preview' : ''} ${regeneratingIndices.has(idx) ? 'is-regenerating' : ''}`} 
                                                    style={{ left: `${x + introWidth}px`, width: `${width}px` }}
                                                >
                                                    <div className="montage-clip-content">
                                                        {regeneratingIndices.has(idx) ? (
                                                            <div className="clip-loading-spinner"><div className="spinner-tiny" /></div>
                                                        ) : (
                                                            <div className="montage-clip-thumbnail-placeholder">{clip.isVideo ? '🎬' : '🖼️'}</div>
                                                        )}
                                                        <span className="montage-clip-name">{clip.path.split(/[\\/]/).pop()}</span>
                                                        <span className="montage-clip-duration">{clip.duration.toFixed(1)}s</span>
                                                        
                                                        <div className="montage-clip-actions">
                                                            <button className="clip-action-btn delete" onMouseDown={(e) => { e.stopPropagation(); handleDeleteClip(idx); }}>🗑️</button>
                                                            <button className="clip-action-btn regenerate" onMouseDown={(e) => { e.stopPropagation(); handleOpenRegenerate(idx); }}>🔄</button>
                                                            {!clip.isVideo && (
                                                                <button className="clip-action-btn animate" onMouseDown={(e) => { e.stopPropagation(); handleAnimateClip(idx); }} title="Animate">
                                                                    <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polygon points="23 7 16 12 23 17 23 7"/><rect x="1" y="5" width="15" height="14" rx="2" ry="2"/></svg>
                                                                </button>
                                                            )}
                                                        </div>
                                                    </div>
                                                    {idx < clips.length - 1 && (
                                                        <div className="montage-clip-resizer right" onMouseDown={(e) => { e.stopPropagation(); setDraggingIdx(idx); setStartX(e.clientX); setStartDurations({ current: clips[idx].duration, next: clips[idx + 1].duration }); }} />
                                                    )}
                                                </div>
                                            ))}
                                            {isDraggingFromPool && dropPreview !== null && dropPreview > 0 && draggingHoverTrack === 'clips' && (
                                                <div className="timeline-drop-ghost-precise" style={{ left: `${dropPreview * zoom + introWidth}px`, width: `${isDraggingFromPool.duration * zoom}px` }}>
                                                    <div className="ghost-indicator">DROP TO INSERT</div>
                                                </div>
                                            )}
                                        </div>

                                        <div
                                            className={`montage-track triggers ${isDraggingFromPool && draggingHoverTrack === 'triggers' ? 'accepting-drop' : ''}`}
                                            data-droptarget="triggers"
                                            onDragOver={(e) => {
                                                if (dragPoolItemRef.current || isDraggingFromPool) {
                                                    e.preventDefault();
                                                    e.stopPropagation();
                                                    const rect = e.currentTarget.getBoundingClientRect();
                                                    const x = e.clientX - rect.left + e.currentTarget.scrollLeft;
                                                    setDropPreview((x - introWidth) / zoom);
                                                    setDraggingHoverTrack('triggers');
                                                }
                                            }}
                                            onDragLeave={() => setDraggingHoverTrack(null)}
                                            onDrop={(e) => { 
                                                if (isDraggingFromPool) { 
                                                    e.stopPropagation(); 
                                                    const rect = e.currentTarget.getBoundingClientRect();
                                                    const x = e.clientX - rect.left + e.currentTarget.scrollLeft;
                                                    handleTriggerDrop((x - introWidth) / zoom); 
                                                    setDraggingHoverTrack(null);
                                                } 
                                            }}
                                        >
                                            {triggers.map((tr, i) => {
                                                const width = Math.max(tr.duration * zoom, 40);
                                                const x = tr.startTime * zoom + introWidth;
                                                const adjustedTime = introVideo ? currentTime - effectiveIntroDuration : currentTime;
                                                const isActive = adjustedTime >= tr.startTime && adjustedTime <= tr.startTime + tr.duration;
                                                return (
                                                    <div 
                                                        key={i} 
                                                        className={`montage-trigger-marker ${isActive ? 'active' : ''} ${selectedTriggerIdx === i ? 'selected' : ''}`}
                                                        style={{ left: `${x}px`, width: `${width}px` }}
                                                        onMouseDown={(e) => {
                                                            e.preventDefault(); e.stopPropagation();
                                                            setDraggingTriggerIdx(i);
                                                            setStartX(e.clientX);
                                                            setDragTriggerStartPos(tr.startTime);
                                                            setSelectedTriggerIdx(i);
                                                            setSelectedWatermarkIdx(null);
                                                            setActiveInfoTab('stats');
                                                        }}
                                                    >
                                                        <div className="trigger-icon">🎯</div>
                                                        <div className="trigger-phrase">{tr.phrase}</div>
                                                        <button className="trigger-delete-btn" onClick={(e) => { e.stopPropagation(); handleDeleteTrigger(i); }}>✕</button>
                                                        
                                                        {!tr.isVideo && (
                                                            <>
                                                                <div className="trigger-resizer-timeline left" onMouseDown={(e) => { e.preventDefault(); e.stopPropagation(); setDraggingTriggerSide('start'); setDraggingTriggerIdx(i); setSelectedTriggerIdx(i); setStartX(e.clientX); setDragTriggerStartPos(tr.startTime); setDragTriggerStartDur(tr.duration); }} />
                                                                <div className="trigger-resizer-timeline right" onMouseDown={(e) => { e.preventDefault(); e.stopPropagation(); setDraggingTriggerSide('end'); setDraggingTriggerIdx(i); setSelectedTriggerIdx(i); setStartX(e.clientX); setDragTriggerStartPos(tr.startTime); setDragTriggerStartDur(tr.duration); }} />
                                                            </>
                                                        )}
                                                    </div>
                                                );
                                            })}
                                            {isDraggingFromPool && dropPreview !== null && dropPreview > 0 && draggingHoverTrack === 'triggers' && (
                                                <div className="timeline-drop-ghost-precise" style={{ left: `${dropPreview * zoom + introWidth}px`, width: '120px' }}>
                                                    <div className="ghost-indicator" style={{ background: '#ffcc00', color: '#000' }}>ADD TRIGGER</div>
                                                </div>
                                            )}
                                        </div>

                                        <div
                                            className={`montage-track watermarks ${isDraggingFromPool && draggingHoverTrack === 'watermarks' ? 'accepting-drop' : ''}`}
                                            data-droptarget="watermarks"
                                            onDragOver={(e) => {
                                                if (dragPoolItemRef.current || isDraggingFromPool) {
                                                    e.preventDefault(); e.stopPropagation();
                                                    const rect = e.currentTarget.getBoundingClientRect();
                                                    const x = e.clientX - rect.left + e.currentTarget.scrollLeft;
                                                    setDropPreview((x - introWidth) / zoom);
                                                    setDraggingHoverTrack('watermarks');
                                                }
                                            }}
                                            onDragLeave={() => setDraggingHoverTrack(null)}
                                            onDrop={(e) => { 
                                                if (isDraggingFromPool) { 
                                                    e.stopPropagation(); 
                                                    const rect = e.currentTarget.getBoundingClientRect();
                                                    const x = e.clientX - rect.left + e.currentTarget.scrollLeft;
                                                    handleWatermarkDrop((x - introWidth) / zoom); 
                                                    setDraggingHoverTrack(null);
                                                } 
                                            }}
                                        >
                                            <div className="track-label-mini">Watermarks</div>
                                            {watermarks.filter(w => !w.trackId).map((w, i_raw) => {
                                                const i = watermarks.findIndex(wm => wm.id === w.id);
                                                const width = w.duration * zoom;
                                                const x = w.startTime * zoom + introWidth;
                                                const adjustedTime = introVideo ? currentTime - effectiveIntroDuration : currentTime;
                                                const isActive = adjustedTime >= w.startTime && adjustedTime <= w.startTime + w.duration;
                                                return (
                                                    <div 
                                                        key={w.id} 
                                                        className={`montage-watermark-marker ${isActive ? 'active' : ''} ${selectedWatermarkIdx === i ? 'selected' : ''}`}
                                                        style={{ left: `${x}px`, width: `${width}px` }}
                                                        onMouseDown={(e) => {
                                                            e.preventDefault(); e.stopPropagation();
                                                            setDraggingWatermarkIdx(i);
                                                            setStartX(e.clientX);
                                                            setDragWatermarkStartPos(w.startTime);
                                                            setSelectedWatermarkIdx(i);
                                                            setSelectedTriggerIdx(null);
                                                            setActiveInfoTab('stats');
                                                        }}
                                                    >
                                                        <div className="watermark-icon">{w.isVideo ? '🎬' : '🖼️'}</div>
                                                        <div className="watermark-name">{w.path.split(/[\\/]/).pop()}</div>
                                                        <button className="trigger-delete-btn" onClick={(e) => { e.stopPropagation(); handleDeleteWatermark(i); }}>✕</button>
                                                        
                                                        <div className="watermark-resizer-timeline left" onMouseDown={(e) => { e.preventDefault(); e.stopPropagation(); setDraggingWatermarkSide('start'); setDraggingWatermarkIdx(i); setStartX(e.clientX); setDragWatermarkStartPos(w.startTime); setDragWatermarkStartDur(w.duration); }} />
                                                        <div className="watermark-resizer-timeline right" onMouseDown={(e) => { e.preventDefault(); e.stopPropagation(); setDraggingWatermarkSide('end'); setDraggingWatermarkIdx(i); setStartX(e.clientX); setDragWatermarkStartPos(w.startTime); setDragWatermarkStartDur(w.duration); }} />
                                                    </div>
                                                );
                                            })}
                                            {isDraggingFromPool && dropPreview !== null && dropPreview > 0 && draggingHoverTrack === 'watermarks' && (
                                                <div className="timeline-drop-ghost-precise" style={{ left: `${dropPreview * zoom + introWidth}px`, width: '120px' }}>
                                                    <div className="ghost-indicator" style={{ background: 'var(--accent-color)', color: '#fff' }}>ADD WATERMARK</div>
                                                </div>
                                            )}
                                        </div>

                                        {extraTracks.map((track) => (
                                            <div
                                                key={track.id}
                                                className={`montage-track extra-overlay ${draggingHoverTrack === track.id ? 'accepting-drop' : ''}`}
                                                data-droptarget="extra"
                                                data-trackid={track.id}
                                                style={{
                                                    borderLeft: `4px solid ${track.color}`,
                                                    background: draggingHoverTrack === track.id
                                                        ? `color-mix(in srgb, ${track.color}, transparent 80%)`
                                                        : `color-mix(in srgb, ${track.color}, transparent 95%)`
                                                }}
                                                onDragOver={(e) => {
                                                    if (dragPoolItemRef.current || isDraggingFromPool) {
                                                        e.preventDefault(); e.stopPropagation();
                                                        const rect = e.currentTarget.getBoundingClientRect();
                                                        const x = e.clientX - rect.left + e.currentTarget.scrollLeft;
                                                        setDropPreview((x - introWidth) / zoom);
                                                        setDraggingHoverTrack(track.id);
                                                    }
                                                }}
                                                onDragLeave={() => setDraggingHoverTrack(null)}
                                                onDrop={(e) => {
                                                    if (dragPoolItemRef.current || isDraggingFromPool) {
                                                        e.preventDefault(); e.stopPropagation();
                                                        const rect = e.currentTarget.getBoundingClientRect();
                                                        const x = e.clientX - rect.left + e.currentTarget.scrollLeft;
                                                        handleWatermarkDrop((x - introWidth) / zoom, track.id);
                                                        setDraggingHoverTrack(null);
                                                        setDropPreview(null);
                                                    }
                                                }}
                                            >
                                                <div className="track-label-mini" style={{ color: track.color }}>
                                                    {track.name}
                                                    <button className="remove-track-btn" onClick={() => handleRemoveTrack(track.id)}>✕</button>
                                                </div>
                                                {watermarks.filter(w => w.trackId === track.id).map((w) => {
                                                    const originalIdx = watermarks.findIndex(wm => wm.id === w.id);
                                                    const width = w.duration * zoom;
                                                    const x = w.startTime * zoom + introWidth;
                                                    const adjustedTime = introVideo ? currentTime - effectiveIntroDuration : currentTime;
                                                    const isActive = adjustedTime >= w.startTime && adjustedTime <= w.startTime + w.duration;
                                                    return (
                                                        <div 
                                                            key={w.id} 
                                                            className={`montage-watermark-marker ${isActive ? 'active' : ''} ${selectedWatermarkIdx === originalIdx ? 'selected' : ''}`}
                                                            style={{ 
                                                                left: `${x}px`, 
                                                                width: `${width}px`,
                                                                background: isActive ? track.color : `color-mix(in srgb, ${track.color}, transparent 85%)`,
                                                                borderColor: isActive ? track.color : `color-mix(in srgb, ${track.color}, transparent 70%)`
                                                            }}
                                                            onMouseDown={(e) => {
                                                                e.preventDefault(); e.stopPropagation();
                                                                setDraggingWatermarkIdx(originalIdx);
                                                                setStartX(e.clientX);
                                                                setDragWatermarkStartPos(w.startTime);
                                                                setSelectedWatermarkIdx(originalIdx);
                                                                setSelectedTriggerIdx(null);
                                                                setActiveInfoTab('stats');
                                                            }}
                                                        >
                                                            <div className="watermark-icon">{w.isVideo ? '🎬' : '🖼️'}</div>
                                                            <div className="watermark-name" style={{ color: isActive ? '#000' : 'inherit' }}>{w.path.split(/[\\/]/).pop()}</div>
                                                            <button className="trigger-delete-btn" onClick={(e) => { e.stopPropagation(); handleDeleteWatermark(originalIdx); }}>✕</button>
                                                            
                                                            <div className="watermark-resizer-timeline left" onMouseDown={(e) => { e.preventDefault(); e.stopPropagation(); setDraggingWatermarkSide('start'); setDraggingWatermarkIdx(originalIdx); setStartX(e.clientX); setDragWatermarkStartPos(w.startTime); setDragWatermarkStartDur(w.duration); }} />
                                                            <div className="watermark-resizer-timeline right" onMouseDown={(e) => { e.preventDefault(); e.stopPropagation(); setDraggingWatermarkSide('end'); setDraggingWatermarkIdx(originalIdx); setStartX(e.clientX); setDragWatermarkStartPos(w.startTime); setDragWatermarkStartDur(w.duration); }} />
                                                        </div>
                                                    );
                                                })}
                                                {isDraggingFromPool && dropPreview !== null && dropPreview >= 0 && draggingHoverTrack === track.id && (
                                                    <div className="timeline-drop-ghost-precise" style={{ left: `${dropPreview * zoom + introWidth}px`, width: '120px' }}>
                                                        <div className="ghost-indicator" style={{ background: track.color, color: '#fff' }}>ADD TO {track.name.toUpperCase()}</div>
                                                    </div>
                                                )}
                                            </div>
                                        ))}

                                        <div className="montage-track-controls">
                                            <button className="add-track-btn" onClick={handleAddTrack}>
                                                <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor"><path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z" /></svg>
                                                Add Overlay Track
                                            </button>
                                        </div>
                                    </div>
                                    {isDraggingFromPool && dropPreview !== null && dropPreview > 0 && draggingHoverTrack === 'clips' && (
                                        <div className="timeline-insertion-guide" style={{ left: `${dropPreview * zoom + introWidth}px`, height: '100%' }}><div className="guide-line" /></div>
                                    )}
                                    <div className="montage-playhead" style={{ left: `${currentTime * zoom + (introVideo ? 0 : introWidth)}px` }}><div className="playhead-handle" /></div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
                <div className="montage-editor-footer">
                    <button className="montage-btn secondary" onClick={() => onCancel(task.id)}>{t('common.cancel')}</button>
                    <button className="montage-btn primary premium-button" onClick={() => {
                        const clipData = clips.map(c => `${c.path}|${c.duration.toFixed(3)}|${c.isVideo ? 'v' : 'i'}`).join('::');
                        const ss = audioSegments.map(s => `${s.start.toFixed(3)},${s.end.toFixed(3)}`).join('|');
                        const trData = triggers.map(t => `${t.phrase}|${t.path}|${t.startTime.toFixed(3)}|${t.duration.toFixed(3)}|${t.x}|${t.y}|${t.w}|${t.h}|${t.isVideo ? 'v' : 'i'}`).join('::');
                        const wmData = watermarks.map(w => {
                            const tid = w.trackId || "";
                            const isV = w.isVideo ? 'v' : 'i';
                            return `${w.id}|${w.path}|${w.startTime.toFixed(3)}|${w.duration.toFixed(3)}|${w.x}|${w.y}|${w.w}|${w.h}|${w.opacity.toFixed(2)}|${tid}|${isV}`;
                        }).join('::');
                        const introData = introVideo ? introVideo.path : "none";
                        const etData = JSON.stringify(extraTracks);
                        onConfirm(task.id, `confirm_v2:${clipData};segments:${ss};triggers:${trData};watermarks:${wmData};intro:${introData};extraTracks:${etData}`);
                    }}>{t('common.save')}</button>
                </div>
            </div>
            <RegenerateModal 
                isOpen={isRegModalOpen}
                initialPrompt={regIdx !== null ? prompts[regIdx] || "" : ""}
                imagePath={regIdx !== null ? clips[regIdx].path : ""}
                onClose={() => setIsRegModalOpen(false)}
                onConfirm={handleRegenerateConfirm}
            />

            {editingTriggerIdx !== null && (
                <div className="montage-modal-overlay animate-fade-in">
                    <div className="montage-modal-window trigger-edit animate-scale-up">
                        <div className="montage-modal-header">
                            <h3 className="modal-title">Edit Trigger Phrase</h3>
                            <button className="close-btn" onClick={() => setEditingTriggerIdx(null)}>✕</button>
                        </div>
                        <div className="montage-modal-body">
                            <p className="modal-hint">Enter the phrase from the script to sync this trigger with, or just a descriptive label.</p>
                            <textarea 
                                className="trigger-phrase-input"
                                value={tempTriggerPhrase}
                                onChange={(e) => setTempTriggerPhrase(e.target.value)}
                                placeholder="Enter trigger phrase..."
                                autoFocus
                            />
                        </div>
                        <div className="montage-modal-footer">
                            <button className="montage-btn secondary" onClick={() => setEditingTriggerIdx(null)}>Cancel</button>
                            <button className="montage-btn primary" onClick={handleSaveTriggerPhrase}>Apply Changes</button>
                        </div>
                    </div>
                </div>
            )}
            {fullscreenPoolIdx !== null && mediaPool[fullscreenPoolIdx] && (
                <div className="clip-fullscreen-overlay" onClick={() => setFullscreenPoolIdx(null)}>
                    {fullscreenPoolIdx > 0 && (
                        <button className="clip-fs-nav left" onClick={(e) => { e.stopPropagation(); setFullscreenPoolIdx(fullscreenPoolIdx - 1); }}>‹</button>
                    )}
                    {fullscreenPoolIdx < mediaPool.length - 1 && (
                        <button className="clip-fs-nav right" onClick={(e) => { e.stopPropagation(); setFullscreenPoolIdx(fullscreenPoolIdx + 1); }}>›</button>
                    )}
                    <div className="clip-fullscreen-media" onClick={(e) => e.stopPropagation()}>
                        {mediaPool[fullscreenPoolIdx].isVideo ? (
                            <video src={getUrl(mediaPool[fullscreenPoolIdx].path)} className="clip-fullscreen-element" controls autoPlay muted playsInline />
                        ) : (
                            <img src={getUrl(mediaPool[fullscreenPoolIdx].path)} alt="fullscreen" className="clip-fullscreen-element" />
                        )}
                        <div className="clip-fullscreen-topbar">
                            <div className="clip-fullscreen-actions">
                                <button className="clip-fs-btn delete" onClick={() => { handleRemoveFromPool(fullscreenPoolIdx); setFullscreenPoolIdx(null); }} title="Remove">🗑️</button>
                            </div>
                            <div className="clip-fullscreen-dur">{mediaPool[fullscreenPoolIdx].duration.toFixed(1)}s</div>
                            <button className="clip-fs-btn close" onClick={() => setFullscreenPoolIdx(null)} title="Close">✕</button>
                        </div>
                    </div>
                </div>
            )}
            {fullscreenClipIdx !== null && clips[fullscreenClipIdx] && (
                <div className="clip-fullscreen-overlay" onClick={() => setFullscreenClipIdx(null)}>
                    {fullscreenClipIdx > 0 && (
                        <button className="clip-fs-nav left" onClick={(e) => { e.stopPropagation(); setFullscreenClipIdx(fullscreenClipIdx - 1); }}>‹</button>
                    )}
                    {fullscreenClipIdx < clips.length - 1 && (
                        <button className="clip-fs-nav right" onClick={(e) => { e.stopPropagation(); setFullscreenClipIdx(fullscreenClipIdx + 1); }}>›</button>
                    )}
                    <div className="clip-fullscreen-media" onClick={(e) => e.stopPropagation()}>
                        {clips[fullscreenClipIdx].isVideo ? (
                            <video
                                src={getUrl(clips[fullscreenClipIdx].path)}
                                className="clip-fullscreen-element"
                                controls
                                autoPlay
                                muted
                                playsInline
                            />
                        ) : (
                            <img
                                src={getUrl(clips[fullscreenClipIdx].path)}
                                alt="fullscreen"
                                className="clip-fullscreen-element"
                            />
                        )}
                        <div className="clip-fullscreen-topbar">
                            <div className="clip-fullscreen-actions">
                                <button className="clip-fs-btn delete" onClick={() => { handleDeleteClip(fullscreenClipIdx); setFullscreenClipIdx(null); }} title="Delete">🗑️</button>
                                <button className="clip-fs-btn regen" onClick={() => { handleOpenRegenerate(fullscreenClipIdx); setFullscreenClipIdx(null); }} title="Regenerate">🔄</button>
                                {!clips[fullscreenClipIdx].isVideo && (
                                    <button className="clip-fs-btn animate" onClick={() => { handleAnimateClip(fullscreenClipIdx); setFullscreenClipIdx(null); }} title="Animate">
                                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polygon points="23 7 16 12 23 17 23 7"/><rect x="1" y="5" width="15" height="14" rx="2" ry="2"/></svg>
                                    </button>
                                )}
                            </div>
                            <div className="clip-fullscreen-dur">{clips[fullscreenClipIdx].duration.toFixed(1)}s</div>
                            <button className="clip-fs-btn close" onClick={() => setFullscreenClipIdx(null)} title="Close">✕</button>
                        </div>
                    </div>
                </div>
            )}
            {dragPos && isDraggingFromPool && (
                <div className="pool-drag-ghost" style={{ left: dragPos.x + 14, top: dragPos.y + 14 }}>
                    <span className="ghost-name">{isDraggingFromPool.path.split(/[\\/]/).pop()}</span>
                    <span className="ghost-dur">{isDraggingFromPool.duration.toFixed(1)}s</span>
                </div>
            )}
        </div>
    );
};
