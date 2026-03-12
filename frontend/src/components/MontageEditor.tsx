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

interface MontagePlan {
    audioDuration: number;
    audioPath: string | null;
    transDuration: number;
    isFadeFast: boolean;
    clips: MontageClip[];
    subtitlePath?: string;
    audioSegments?: MontageSegment[];
    triggers?: MontageTrigger[];
    baseW?: number;
    baseH?: number;
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
    const [activeInfoTab, setActiveInfoTab] = useState<'library' | 'stats'>('library');
    const [selection, setSelection] = useState<{ start: number | null, end: number | null }>({ start: null, end: null });
    const [audioSegments, setAudioSegments] = useState<MontageSegment[]>([]);
    const [isCuttingMode, setIsCuttingMode] = useState<boolean>(false);
    const [draggingSelectionSide, setDraggingSelectionSide] = useState<null | 'start' | 'end'>(null);
    const [cutJunctions, setCutJunctions] = useState<{ position: number, durationRemoved: number }[]>([]);
    const [subtitles, setSubtitles] = useState<SubtitleEntry[]>([]);

    const [triggers, setTriggers] = useState<MontageTrigger[]>([]);
    const [draggingTriggerIdx, setDraggingTriggerIdx] = useState<number | null>(null);
    const [dragTriggerStartPos, setDragTriggerStartPos] = useState<number>(0);
    const [dragTriggerOffsetX, setDragTriggerOffsetX] = useState<number>(0);
    const [draggingTriggerPosIdx, setDraggingTriggerPosIdx] = useState<number | null>(null);
    const [dragStartCoords, setDragStartCoords] = useState<{ x: number, y: number, mouseX: number, mouseY: number } | null>(null);

    const previewVideoRef = useRef<HTMLVideoElement>(null);
    const previewAudioRef = useRef<HTMLAudioElement>(null);
    const previewWrapRef = useRef<HTMLDivElement>(null);
    const containerRef = useRef<HTMLDivElement>(null);
    const poolRef = useRef<HTMLDivElement>(null);
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
    const [isDraggingExternal, setIsDraggingExternal] = useState(false);
    const dragCounter = useRef(0);
    const [dropPreview, setDropPreview] = useState<number | null>(null);
    const [hoveredMediaIdx, setHoveredMediaIdx] = useState<number | null>(null);
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
                setClips(parsed.clips.map((c: MontageClip) => ({ ...c })));
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

                // Try to load prompts.txt for regeneration
                if (parsed.audioPath) {
                    const taskDir = parsed.audioPath.split(/[\\/]voice\.mp3/)[0];
                    const promptsPath = `${taskDir}/prompts.txt`;
                    // @ts-ignore
                    window.go.main.App.ReadFile(promptsPath).then(content => {
                        if (content) {
                            const pStrs = content.split('\n\n--------------------\n\n').map((s: string) => s.trim());
                            setPrompts(pStrs);
                        }
                    }).catch(() => console.log("No prompts.txt found"));
                }
            } catch (e) {
                console.error("Failed to parse montage plan:", e);
            }
        }
    }, [task.montagePlanData]);

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
            if (currentTime >= startTime && currentTime <= endTime + 0.001) {
                let timeInClip = currentTime - startTime;
                
                // BOOMERANG LOGIC
                if (layout.clip.isVideo && layout.clip.actualDuration && layout.clip.actualDuration < layout.clip.duration) {
                    const actualDur = layout.clip.actualDuration;
                    const cycle = actualDur * 2;
                    const pos = timeInClip % cycle;
                    if (pos <= actualDur) timeInClip = pos;
                    else timeInClip = actualDur - (pos - actualDur);
                }
                
                return { ...layout, timeInClip };
            }
        }
        return null;
    }, [clipLayouts, currentTime, zoom]);

    const animStateRef = useRef<AnimationState>({
        currentTime, selection, isPlaying, audioSegments, clips, zoom, totalDuration: totalTimelineDuration
    });
    animStateRef.current = { currentTime, selection, isPlaying, audioSegments, clips, zoom, totalDuration: totalTimelineDuration };

    const getOriginalTime = useCallback((timelineTime: number) => {
        let currentTimeline = 0;
        for (const seg of audioSegments) {
            const segDur = seg.end - seg.start;
            if (timelineTime <= currentTimeline + segDur + 0.001) {
                return seg.start + (timelineTime - currentTimeline);
            }
            currentTimeline += segDur;
        }
        return audioSegments.length > 0 ? audioSegments[audioSegments.length - 1].end : timelineTime;
    }, [audioSegments]);

    const currentSubtitle = useMemo(() => {
        const origTime = getOriginalTime(currentTime);
        return subtitles.find(s => origTime >= s.start && origTime <= s.end);
    }, [subtitles, currentTime, getOriginalTime]);

    const activeClipInfoRef = useRef(activeClipInfo);
    activeClipInfoRef.current = activeClipInfo;

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
            if (previewVideoRef.current && activeClipInfoRef.current?.clip.isVideo) {
                 const targetV = activeClipInfoRef.current.timeInClip;
                 if (Math.abs(previewVideoRef.current.currentTime - targetV) > 0.15) {
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
                previewAudioRef.current.play().catch((e) => console.error("Audio play failed:", e));
            }
            if (previewVideoRef.current && activeClipInfo?.clip.isVideo) {
                previewVideoRef.current.currentTime = activeClipInfo.timeInClip;
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
        const x = e.clientX - rect.left + containerRef.current.scrollLeft - 24;
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
        setDraggingTriggerPosIdx(null);
        setDragStartCoords(null);
        setIsScrubbing(false);
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
        return triggers.map((t, i) => ({ ...t, index: i }))
                       .filter(tr => currentTime >= tr.startTime && currentTime <= tr.startTime + tr.duration);
    }, [triggers, currentTime]);

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
        setIsDraggingFromPool(item);
    };

    const handleInternalDrop = useCallback((dropTime: number) => {
        if (!isDraggingFromPool) return;
        setClips(prev => {
            const next: MontageClip[] = [];
            let currentTimePos = 0;
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
            const targetStart = prev.slice(0, targetIdx).reduce((s, c) => s + c.duration, 0);
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
        setIsDraggingFromPool(null);
        setDropPreview(null);
    }, [isDraggingFromPool]);

    const handleTriggerDrop = useCallback((dropTime: number) => {
        if (!isDraggingFromPool) return;
        
        const newTrigger: MontageTrigger = {
            phrase: isDraggingFromPool.path.split(/[\\/]/).pop()?.split('.')[0] || "Trigger",
            path: isDraggingFromPool.path,
            startTime: dropTime,
            duration: isDraggingFromPool.actualDuration && isDraggingFromPool.actualDuration > 0 ? isDraggingFromPool.actualDuration : 3.0,
            isVideo: isDraggingFromPool.isVideo,
            x: 0,
            y: 0,
            w: plan?.baseW || 1920,
            h: plan?.baseH || 1080
        };

        setTriggers(prev => [...prev, newTrigger]);
        setIsDraggingFromPool(null);
        setDropPreview(null);
    }, [isDraggingFromPool, plan]);

    const handleDeleteTrigger = useCallback((idx: number) => {
        setTriggers(prev => prev.filter((_, i) => i !== idx));
    }, []);

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
            } else if (draggingSelectionSide !== null && containerRef.current) {
                const rect = containerRef.current.getBoundingClientRect();
                const x = e.clientX - rect.left + containerRef.current.scrollLeft - 24;
                const newTime = Math.max(0, Math.min(animStateRef.current.totalDuration, x / zoom));
                setSelection(prev => ({ ...prev, [draggingSelectionSide]: newTime }));
            }
            if (isScrubbing) handleTimelineMove(e);
        };
        if (draggingIdx !== null || isScrubbing || draggingSelectionSide !== null || draggingTriggerIdx !== null || draggingTriggerPosIdx !== null) {
            document.addEventListener('mousemove', mm);
            document.addEventListener('mouseup', mu);
        }
        return () => { document.removeEventListener('mousemove', mm); document.removeEventListener('mouseup', mu); };
    }, [draggingIdx, isScrubbing, draggingSelectionSide, draggingTriggerIdx, draggingTriggerPosIdx, dragStartCoords, handleTimelineMove, zoom, startX, startDurations, dragTriggerStartPos, handleMouseUp, plan]);

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
            <div key={idx} className={`montage-clip-block ${clip.isVideo ? 'video' : 'image'} ${activeClipInfo?.idx === idx ? 'active-preview' : ''} ${regeneratingIndices.has(idx) ? 'is-regenerating' : ''}`} style={{ left: `${x}px`, width: `${width}px` }}>
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
                    </div>
                </div>
                {idx < clips.length - 1 && (
                    <div className="montage-clip-resizer right" onMouseDown={(e) => { e.stopPropagation(); setDraggingIdx(idx); setStartX(e.clientX); setStartDurations({ current: clips[idx].duration, next: clips[idx + 1].duration }); }} />
                )}
            </div>
        ));
    }, [clipLayouts, activeClipInfo?.idx, clips, regeneratingIndices, handleDeleteClip, handleOpenRegenerate]);

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
                    <div className="montage-editor-title">{t('pipeline.montage_control') || 'Montage Editor'} - {task.name}</div>
                    <div className="montage-editor-controls">
                        <button className="montage-btn icon" onClick={() => setZoom(p => Math.max(p - (p > 50 ? 10 : 5), 5))}>-</button>
                        <span className="montage-zoom-label">{zoom}%</span>
                        <button className="montage-btn icon" onClick={() => setZoom(p => Math.min(p + (p < 50 ? 5 : 10), 400))}>+</button>
                    </div>
                </div>
                <div className="montage-editor-body">
                    <div className="montage-preview-and-info">
                        <div className="montage-preview-container">
                            {activeClipInfo ? (
                                <div className="montage-preview-wrap">
                                    <div 
                                        className="preview-media-wrapper" 
                                        ref={previewWrapRef}
                                        style={{ aspectRatio: `${previewAspect}` }}
                                    >
                                        {activeClipInfo.clip.isVideo ? (
                                            <video 
                                                ref={previewVideoRef} 
                                                src={getUrl(activeClipInfo.clip.path)} 
                                                muted 
                                                playsInline 
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
                                                key={tr.index}
                                                className={`preview-trigger-overlay ${draggingTriggerPosIdx === tr.index ? 'dragging' : ''}`}
                                                style={{
                                                    left: `${(tr.x / (plan.baseW || 1920)) * 100}%`,
                                                    top: `${(tr.y / (plan.baseH || 1080)) * 100}%`,
                                                    width: `${(tr.w / (plan.baseW || 1920)) * 100}%`,
                                                    height: `${(tr.h / (plan.baseH || 1080)) * 100}%`,
                                                }}
                                                onMouseDown={(e) => {
                                                    e.stopPropagation();
                                                    setDraggingTriggerPosIdx(tr.index);
                                                    setDragStartCoords({ x: tr.x, y: tr.y, mouseX: e.clientX, mouseY: e.clientY });
                                                }}
                                            >
                                                <div className="trigger-overlay-handle">🎯</div>
                                                <div className="trigger-overlay-label">{tr.phrase}</div>
                                            </div>
                                        ))}
                                    </div>
                                </div>
                            ) : <div className="montage-preview-placeholder">No preview</div>}
                        </div>
                        <div className="montage-info-resizer-v" onMouseDown={handleInfoResizeMouseDown}><div className="resizer-handle-v-line"></div></div>
                        <div className="montage-info-panel" style={{ width: `${infoPanelWidth}px`, flex: 'none' }}>
                            <div className="info-tabs">
                                <button className={`info-tab ${activeInfoTab === 'library' ? 'active' : ''}`} onClick={() => setActiveInfoTab('library')}>Library</button>
                                <button className={`info-tab ${activeInfoTab === 'stats' ? 'active' : ''}`} onClick={() => setActiveInfoTab('stats')}>Stats</button>
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
                                                        draggable 
                                                        onDragStart={() => handleInternalDragStart(m)} 
                                                        onDragEnd={() => setIsDraggingFromPool(null)} 
                                                        onMouseEnter={() => setHoveredMediaIdx(i)}
                                                        onMouseLeave={() => setHoveredMediaIdx(null)}
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
                                                                />
                                                            ) : (
                                                                <img src={getUrl(m.path)} alt="thumb" className="pool-thumb-img" />
                                                            )}
                                                            {m.isVideo && hoveredMediaIdx !== i && <div className="pool-video-overlay">🎬</div>}
                                                        </div>
                                                        <div className="pool-dur">{m.duration.toFixed(1)}s</div>
                                                    </div>
                                                ))}
                                            </div>
                                        ) : (
                                            <div className="pool-empty-state"><div className="empty-icon">📁</div><p>Empty</p><button className="add-files-btn-center" onClick={handleAddMedia}>Add Files</button></div>
                                        )}
                                    </>
                                ) : (
                                    <div className="project-stats-tab animate-fade-in">
                                        <div className="stat-card"><div className="stat-label">Total Clips</div><div className="stat-value">{clips.length}</div></div>
                                        <div className="stat-card"><div className="stat-label">Duration</div><div className="stat-value">{totalTimelineDuration.toFixed(2)}s</div></div>
                                        <div className="stat-card"><div className="stat-label">Audio Sync</div><div className="stat-value">{plan?.audioDuration.toFixed(2)}s</div></div>
                                        <div className="stat-card"><div className="stat-label">Transitions</div><div className="stat-value">{plan?.transDuration}s ({plan?.isFadeFast ? 'Fast' : 'Fade'})</div></div>
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
                        className={`montage-timeline-container ${isDraggingFromPool ? 'accepting-drop' : ''}`} 
                        ref={containerRef} 
                        onDragOver={(e) => { if (isDraggingFromPool) { e.preventDefault(); const rect = containerRef.current!.getBoundingClientRect(); const x = e.clientX - rect.left + containerRef.current!.scrollLeft - 24; setDropPreview(x / zoom); } }}
                        onDragLeave={() => setDropPreview(null)}
                        onDrop={(e) => { if (isDraggingFromPool) { const rect = containerRef.current!.getBoundingClientRect(); const x = e.clientX - rect.left + containerRef.current!.scrollLeft - 24; handleInternalDrop(x / zoom); } }}
                        style={{ height: `${timelineHeight}px`, flex: 'none' }}
                    >
                        <div className="montage-timeline-wrapper" style={{ width: `${totalTimelineDuration * zoom + 100}px`, minWidth: '100%' }}>
                            <div className="montage-timeline-ruler" onMouseDown={handleMouseDownGlobal}>
                                {markers}
                                {selection.start !== null && <div className="selection-marker start interactive" style={{ left: `${selection.start * zoom}px` }} onMouseDown={(e) => { e.stopPropagation(); setDraggingSelectionSide('start'); }}><div className="marker-handle" /></div>}
                                {selection.end !== null && <div className="selection-marker end interactive" style={{ left: `${selection.end * zoom}px` }} onMouseDown={(e) => { e.stopPropagation(); setDraggingSelectionSide('end'); }}><div className="marker-handle" /></div>}
                                {selection.start !== null && selection.end !== null && <div className="selection-range" style={{ left: `${Math.min(selection.start, selection.end) * zoom}px`, width: `${Math.abs(selection.end - selection.start) * zoom}px` }} />}
                                {cutJunctions.map((j, i) => (<div key={i} className="timeline-cut-junction" style={{ left: `${j.position * zoom}px` }}><div className="junction-icon">✂</div></div>))}
                                <div className="audio-track-reference" style={{ width: `${totalTimelineDuration * zoom}px` }} onMouseDown={handleMouseDownGlobal}><span>Audio Sequence</span></div>
                            </div>
                            <div className="montage-timeline-tracks">
                                <div className="montage-track clips">
                                    {clipElements}
                                    {isDraggingFromPool && dropPreview !== null && (
                                        <div className="timeline-drop-ghost-precise" style={{ left: `${dropPreview * zoom}px`, width: `${isDraggingFromPool.duration * zoom}px` }}>
                                            <div className="ghost-indicator">DROP TO INSERT</div>
                                        </div>
                                    )}
                                </div>

                                <div 
                                    className={`montage-track triggers ${isDraggingFromPool ? 'accepting-drop' : ''}`}
                                    onDragOver={(e) => { if (isDraggingFromPool) { e.preventDefault(); e.stopPropagation(); const rect = containerRef.current!.getBoundingClientRect(); const x = e.clientX - rect.left + containerRef.current!.scrollLeft - 24; setDropPreview(x / zoom); } }}
                                    onDrop={(e) => { if (isDraggingFromPool) { e.stopPropagation(); const rect = containerRef.current!.getBoundingClientRect(); const x = e.clientX - rect.left + containerRef.current!.scrollLeft - 24; handleTriggerDrop(x / zoom); } }}
                                >
                                    <div className="track-label">Triggers ({triggers.length})</div>
                                    {triggerElements}
                                    {isDraggingFromPool && dropPreview !== null && (
                                        <div className="trigger-drop-ghost" style={{ left: `${dropPreview * zoom}px`, width: `${(isDraggingFromPool.actualDuration || 3.0) * zoom}px` }}>
                                            <div className="ghost-text">DROP TRIGGER</div>
                                        </div>
                                    )}
                                </div>
                            </div>
                            {isDraggingFromPool && dropPreview !== null && (
                                <div className="timeline-insertion-guide" style={{ left: `${dropPreview * zoom}px`, height: '100%' }}><div className="guide-line" /></div>
                            )}
                            <div className="montage-playhead" style={{ left: `${currentTime * zoom}px` }}><div className="playhead-handle" /></div>
                        </div>
                    </div>
                </div>
                <div className="montage-editor-footer">
                    <button className="montage-btn secondary" onClick={() => onCancel(task.id)}>{t('common.cancel')}</button>
                    <button className="montage-btn primary premium-button" onClick={() => {
                        const clipData = clips.map(c => `${c.path}|${c.duration.toFixed(3)}|${c.isVideo ? 'v' : 'i'}`).join('::');
                        const ss = audioSegments.map(s => `${s.start.toFixed(3)},${s.end.toFixed(3)}`).join('|');
                        const trData = triggers.map(t => `${t.phrase}|${t.path}|${t.startTime.toFixed(3)}|${t.duration.toFixed(3)}|${t.x}|${t.y}|${t.w}|${t.h}|${t.isVideo ? 'v' : 'i'}`).join('::');
                        onConfirm(task.id, `confirm_v2:${clipData};segments:${ss};triggers:${trData}`);
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
        </div>
    );
};
