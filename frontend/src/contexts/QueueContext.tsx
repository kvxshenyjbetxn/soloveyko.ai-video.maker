import React, { createContext, useContext, useState, useCallback, ReactNode, useEffect, useRef } from 'react';
import { ProcessTask, SubmitImageControlResult, SubmitExistingFilesResult, ClearGallery, SendControlAction, CancelQueue, ResetQueueCancellation } from '../../wailsjs/go/main/App';
import { EventsOn } from '../../wailsjs/runtime/runtime';
import { useToast } from './ToastContext';
import { useI18n } from './I18nContext';

export type TaskStatus = 'pending' | 'waiting' | 'running' | 'processing' | 'completed' | 'failed';

export interface QueueTask {
    id: string; taskNumber?: number; name: string; folderName: string; subName: string;
    type: 'translate' | 'rewrite' | 'voiceover'; content: string; originalLength?: number; settings: any;
    status: TaskStatus; progress: number; resultLength?: number;
    isAwaitingControl?: boolean; isAwaitingImageControl?: boolean; isAwaitingMontageControl?: boolean; montagePlanData?: string; isAwaitingExistingFilesCheck?: boolean; controlContent?: string;
    existingFilesData?: any;
    textStatus: TaskStatus; voiceStatus: TaskStatus; imageStatus: TaskStatus;
    subtitleStatus: TaskStatus; montageStatus: TaskStatus; montageMsg?: string;
    voiceDuration?: string; imagesMessage?: string; timestamp: number;
    workerName?: string;
    remoteId?: string;
}

interface QueueDataContextType {
    tasks: QueueTask[]; isProcessing: boolean;
    completionModal: { 
        isOpen: boolean; 
        duration: string; 
        activeDuration: string;
        total_montage: string;
        avg_montage: string;
        taskCount: number; 
    };
    imageControlNotification: { isOpen: boolean; };
    isImageBatchReady: boolean;
    montageControlNotification: { isOpen: boolean; };
    regeneratingPaths: Set<string>;
}

interface QueueActionsContextType {
    addTasks: (type: any, content: string, tasksData: any[], name?: string, skippedStages?: string[]) => void;
    addTask: (type: any, content: string, settings: any, name?: string, subName?: string, skippedStages?: string[], existingData?: any) => void;
    removeTask: (id: string) => void; clearQueue: () => void; startQueue: () => Promise<void>;
    getNextTaskName: () => string;
    updateTaskStatus: (id: string, s: TaskStatus, p?: number, l?: number) => void;
    resumeTask: (id: string, text: string) => Promise<void>;
    regenerateTask: (id: string, text: string, settings?: any) => Promise<void>;
    cancelTask: (id: string) => Promise<void>;
    cancelQueue: () => Promise<void>;
    startRemoteQueue: (workerId: string, workerName: string) => Promise<void>;
    resumeImageControl: () => Promise<void>;
    resumeMontageControl: (id: string, resultData: string) => Promise<void>;
    resumeWithExistingFiles: (id: string, skipStages: string[]) => Promise<void>;
    closeCompletionModal: () => void; closeImageControlNotification: () => void; closeMontageControlNotification: () => void;
    addRegeneratingPath: (path: string) => void;
    removeRegeneratingPath: (path: string) => void;
}

const QueueDataContext = createContext<QueueDataContextType | undefined>(undefined);
const QueueActionsContext = createContext<QueueActionsContextType | undefined>(undefined);

export const QueueProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
    const { t } = useI18n(); const { showToast } = useToast();
    const [tasks, setTasks] = useState<QueueTask[]>([]);
    const [isProcessing, setIsProcessing] = useState(false);
    const [completionModal, setCompletionModal] = useState({ 
        isOpen: false, 
        duration: '', 
        activeDuration: '',
        total_montage: '',
        avg_montage: '',
        taskCount: 0 
    });
    const [imageControlNotification, setImageControlNotification] = useState({ isOpen: false });
    const [isImageBatchReady, setIsImageBatchReady] = useState(false);
    const [montageControlNotification, setMontageControlNotification] = useState({ isOpen: false });
    const taskCounterRef = useRef(1);
    const activeBatchRef = useRef<string[]>([]);
    const hasShownImageBatchNotificationRef = useRef(false);
    const hasShownMontageBatchNotificationRef = useRef(false);
    const tasksRef = useRef<QueueTask[]>([]);
    const taskContentRef = useRef<Map<string, string>>(new Map());

    // Tracking time
    const totalPausedTimeRef = useRef(0);
    const pauseStartRef = useRef<number | null>(null);
    const montageStartTimesRef = useRef<Map<string, number>>(new Map());
    const totalMontageTimeRef = useRef(0);
    const completedMontageCountRef = useRef(0);

    useEffect(() => { tasksRef.current = tasks; }, [tasks]);

    // Check if any task is awaiting control to track pause time
    useEffect(() => {
        const anyAwaiting = tasks.some(t => t.isAwaitingControl || t.isAwaitingImageControl || t.isAwaitingMontageControl);
        
        if (anyAwaiting && pauseStartRef.current === null) {
            pauseStartRef.current = Date.now();
        } else if (!anyAwaiting && pauseStartRef.current !== null) {
            totalPausedTimeRef.current += Date.now() - pauseStartRef.current;
            pauseStartRef.current = null;
        }
    }, [tasks]);

    const formatDuration = (ms: number) => {
        const dur = Math.round(ms / 1000);
        const h = Math.floor(dur / 3600);
        const m = Math.floor((dur % 3600) / 60);
        const s = dur % 60;
        if (h > 0) return `${h}${t('common.unit_h')} ${m}${t('common.unit_m')} ${s}${t('common.unit_s')}`;
        if (m > 0) return `${m}${t('common.unit_m')} ${s}${t('common.unit_s')}`;
        return `${s}${t('common.unit_s')}`;
    };

    const sendNotification = useCallback(async (msg: string) => {
        try {
            // Telegram
            // @ts-ignore
            const tgEnabled = await window.go.main.App.GetTelegramNotificationsEnabled();
            if (tgEnabled) {
                // @ts-ignore
                const chatID = await window.go.main.App.GetTelegramChatID();
                if (chatID) {
                    // @ts-ignore
                    await window.go.main.App.SendTelegramNotification(chatID, msg);
                }
            }

            // System
            // @ts-ignore
            await window.go.main.App.SendSystemNotification(msg, "");
        } catch (err) {
            console.error("Failed to send notification:", err);
        }
    }, []);

    const closeCompletionModal = () => setCompletionModal(prev => ({ ...prev, isOpen: false }));
    const closeImageControlNotification = () => setImageControlNotification({ isOpen: false });
    const closeMontageControlNotification = () => setMontageControlNotification({ isOpen: false });
    const [regeneratingPaths, setRegeneratingPaths] = useState<Set<string>>(new Set());

    const addRegeneratingPath = useCallback((path: string) => {
        setRegeneratingPaths(prev => {
            const next = new Set(prev);
            next.add(path);
            return next;
        });
    }, []);

    const removeRegeneratingPath = useCallback((path: string) => {
        setRegeneratingPaths(prev => {
            const next = new Set(prev);
            next.delete(path);
            return next;
        });
    }, []);

    const updateTaskStatus = useCallback((id: string, status: TaskStatus, progress?: number, resultLength?: number) => {
        setTasks(prev => prev.map(t => t.id === id ? { ...t, status, progress: progress ?? t.progress, resultLength: resultLength ?? t.resultLength } : t));
    }, []);

    const getNextTaskName = useCallback(() => {
        return `${t('queue.task_default_name')} ${taskCounterRef.current}`;
    }, [t]);

    const addTask = useCallback((type: any, content: string, settings: any, name?: string, subName?: string, skippedStages?: string[], existingData?: any) => {
        const nr = taskCounterRef.current++; const fName = name?.trim() || `${t('queue.task_default_name')} ${nr}`;

        let imgMsg = "";
        if (existingData?.imageCount > 0 || existingData?.videoCount > 0 || existingData?.promptCount > 0) {
            const parts = [];
            if (existingData.promptCount > 0) parts.push(`p:${existingData.promptCount}`);
            if (existingData.imageCount > 0) parts.push(`i:${existingData.imageCount}`);
            if (existingData.videoCount > 0) parts.push(`v:${existingData.videoCount}`);
            imgMsg = parts.join(' ');
        }

        const effectiveSkip = (skippedStages && existingData && existingData.foundStages)
            ? skippedStages.filter(s => existingData.foundStages.includes(s))
            : skippedStages;

        const id = `t_${Date.now()}_${Math.random().toString(36).substr(2, 5)}`;
        taskContentRef.current.set(id, content);

        const newTask: QueueTask = {
            id,
            name: subName ? `${fName} - ${subName}` : fName, folderName: fName, subName: subName || "",
            type, content: "", originalLength: content.length, // Content moved to Ref
            status: 'pending', progress: 0,
            textStatus: effectiveSkip?.includes('text') ? 'completed' : 'pending',
            voiceStatus: effectiveSkip?.includes('voice') ? 'completed' : 'pending',
            imageStatus: effectiveSkip?.includes('image') ? 'completed' : 'pending',
            subtitleStatus: effectiveSkip?.includes('subtitle') ? 'completed' : 'pending',
            montageStatus: 'pending', montageMsg: undefined, timestamp: Date.now(),
            settings: {
                ...settings,
                skippedStages: effectiveSkip,
                voiceoverEnabled: effectiveSkip?.includes('voice') ? true : settings.voiceoverEnabled,
                imageEnabled: effectiveSkip?.includes('image') ? true : settings.imageEnabled,
                subtitleEnabled: effectiveSkip?.includes('subtitle') ? true : settings.subtitleEnabled,
                montageEnabled: effectiveSkip?.includes('montage') ? true : settings.montageEnabled,
            },
            resultLength: existingData?.textChars || 0,
            voiceDuration: existingData?.voiceDuration || "",
            imagesMessage: imgMsg
        };
        setTasks(prev => [...prev, newTask]);
    }, [t]);

    const addTasks = useCallback((type: any, content: string, tasksData: any[], name?: string, skippedStages?: string[]) => {
        const nr = taskCounterRef.current++; const fName = name?.trim() || `${t('queue.task_default_name')} ${nr}`;
        const now = Date.now();

        const newItems: QueueTask[] = tasksData.map((d, i) => {
            const existingData = d.existing;
            // For batch adding, skippedStages is the union of all found stages.
            // We MUST intersect it with this specific task's foundStages to avoid marking
            // non-existent files as "completed" for templates that don't have them.
            const effectiveSkip = (skippedStages && existingData && existingData.foundStages)
                ? skippedStages.filter(s => existingData.foundStages.includes(s))
                : skippedStages;

            let imgMsg = "";
            if (existingData?.imageCount > 0 || existingData?.videoCount > 0 || existingData?.promptCount > 0) {
                const parts = [];
                if (existingData.promptCount > 0) parts.push(`p:${existingData.promptCount}`);
                if (existingData.imageCount > 0) parts.push(`i:${existingData.imageCount}`);
                if (existingData.videoCount > 0) parts.push(`v:${existingData.videoCount}`);
                imgMsg = parts.join(' ');
            }

            const id = `ts_${now}_${i}_${Math.random().toString(36).substr(2, 5)}`;
            taskContentRef.current.set(id, content);

            return {
                id,
                name: d.subName ? `${fName} - ${d.subName}` : fName, folderName: fName, subName: d.subName || "",
                type, content: "", originalLength: content.length, // Content moved to Ref
                settings: {
                    ...d.settings,
                    skippedStages: effectiveSkip,
                    voiceoverEnabled: effectiveSkip?.includes('voice') ? true : d.settings.voiceoverEnabled,
                    imageEnabled: effectiveSkip?.includes('image') ? true : d.settings.imageEnabled,
                    subtitleEnabled: effectiveSkip?.includes('subtitle') ? true : d.settings.subtitleEnabled,
                    montageEnabled: effectiveSkip?.includes('montage') ? true : d.settings.montageEnabled,
                },
                status: 'pending', progress: 0,
                textStatus: effectiveSkip?.includes('text') ? 'completed' : 'pending',
                voiceStatus: effectiveSkip?.includes('voice') ? 'completed' : 'pending',
                imageStatus: effectiveSkip?.includes('image') ? 'completed' : 'pending',
                subtitleStatus: effectiveSkip?.includes('subtitle') ? 'completed' : 'pending',
                montageStatus: 'pending', montageMsg: undefined, taskNumber: i, timestamp: now,
                resultLength: existingData?.textChars || 0,
                voiceDuration: existingData?.voiceDuration || "",
                imagesMessage: imgMsg
            };
        });
        setTasks(prev => [...prev, ...newItems]);
    }, [t]);

    const removeTask = useCallback((id: string) => {
        setTasks(prev => prev.filter(t => t.id !== id));
        taskContentRef.current.delete(id);
    }, []);
    const clearQueue = useCallback(() => {
        setTasks([]);
        taskContentRef.current.clear();
        setIsProcessing(false);
        setIsImageBatchReady(false);
        ClearGallery();
    }, []);

    const resumeTask = async (id: string, text: string) => {
        taskContentRef.current.set(id, text);
        setTasks(prev => prev.map(t => t.id === id ? { ...t, isAwaitingControl: false } : t));
        await SendControlAction(id, "confirm", text, {});
    };

    const regenerateTask = async (id: string, text: string, settings?: any) => {
        taskContentRef.current.set(id, text);
        setTasks(prev => prev.map(t => t.id === id ? { ...t, isAwaitingControl: false, status: 'running', textStatus: 'running', originalLength: text.length } : t));
        await SendControlAction(id, "regenerate", text, settings || {});
    };

    const cancelTask = async (id: string) => {
        setTasks(prev => prev.map(t => t.id === id ? { ...t, isAwaitingControl: false, status: 'failed', textStatus: 'failed' } : t));
        await SendControlAction(id, "cancel", "", {});
    };

    const cancelQueue = async () => {
        await CancelQueue();
        setTasks(prev => prev.map(t => (t.status === 'running' || t.status === 'waiting') ? { ...t, status: 'failed' } : t));
        setIsProcessing(false);
    };

    const resumeImageControl = async () => {
        setImageControlNotification({ isOpen: false });
        setIsImageBatchReady(false);
        const ids = tasks.filter(t => t.isAwaitingImageControl).map(t => t.id);
        if (ids.length === 0) return;
        setTasks(prev => prev.map(t => ids.includes(t.id) ? { ...t, isAwaitingImageControl: false, imageStatus: 'processing' } : t));
        for (const id of ids) await SubmitImageControlResult(id);
    };

    const resumeMontageControl = async (id: string, resultData: string) => {
        setTasks(prev => prev.map(t => t.id === id ? { ...t, isAwaitingMontageControl: false, montageStatus: 'processing' } : t));
        // We will call the backend binding here
        // @ts-ignore
        if (window.go?.main?.App?.SubmitMontageControlResult) {
            // @ts-ignore
            await window.go.main.App.SubmitMontageControlResult(id, resultData);
        }
    };

    const resumeWithExistingFiles = async (id: string, skipStages: string[]) => {
        setTasks(prev => prev.map(t => {
            if (t.id !== id) return t;
            const up: any = { isAwaitingExistingFilesCheck: false };
            if (skipStages.includes('text')) up.textStatus = 'completed';
            if (skipStages.includes('voice')) up.voiceStatus = 'completed';
            if (skipStages.includes('subtitle')) up.subtitleStatus = 'completed';
            if (skipStages.includes('image')) up.imageStatus = 'completed';
            return { ...t, ...up };
        }));
        await SubmitExistingFilesResult(id, skipStages);
    };

    const startRemoteQueue = useCallback(async (workerId: string, workerName: string) => {
        if (isProcessing) return;
        const pending = tasks.filter(t => t.status === 'pending');
        if (pending.length === 0) return;

        setIsProcessing(true);
        
        try {
            for (const task of pending) {
                setTasks(prev => prev.map(t => t.id === task.id ? { ...t, status: 'waiting', workerName } : t));
                
                const content = taskContentRef.current.get(task.id) || "";
                
                // @ts-ignore
                const remoteTask = await window.go.main.App.SendRemoteTaskWithTarget(
                    workerId, 
                    task.name, 
                    content, 
                    task.settings
                );
                
                // remoteTask is the object returned from the server, it has an ID
                if (remoteTask && remoteTask.id) {
                    setTasks(prev => prev.map(t => t.id === task.id ? { ...t, remoteId: remoteTask.id } : t));
                } else {
                    // Fallback if ID is not returned directly
                    setTasks(prev => prev.map(t => t.id === task.id ? { ...t, status: 'completed', progress: 100 } : t));
                }
            }
            
            showToast(t('common.remote_tasks_sent') || "Tasks sent to worker", 'success');
        } catch (e: any) {
            console.error("Remote queue failed:", e);
            showToast(e.toString(), 'error');
        } finally {
            setIsProcessing(false);
        }
    }, [tasks, isProcessing, t, showToast]);

    // Remote Task Status Polling Loop
    useEffect(() => {
        const hasQueued = tasks.some(task => 
            task.remoteId && (task.status?.toLowerCase() === 'waiting' || task.status?.toLowerCase() === 'pending')
        );
        const hasActive = tasks.some(task => 
            task.remoteId && (task.status?.toLowerCase() === 'running' || task.status?.toLowerCase() === 'processing')
        );

        if (!hasQueued && !hasActive) return;

        const pollStatuses = async () => {
            const activeRemoteIds = tasks
                .filter(t => t.remoteId && ['waiting', 'pending', 'running', 'processing'].includes(t.status?.toLowerCase() || ''))
                .map(t => t.remoteId!);
            
            if (activeRemoteIds.length === 0) return;

            try {
                // @ts-ignore
                const statusMap = await window.go.main.App.GetRemoteTasksStatus(activeRemoteIds);
                if (statusMap) {
                    setTasks(prev => prev.map(taskItem => {
                        if (taskItem.remoteId && statusMap[taskItem.remoteId]) {
                            let newStatus = statusMap[taskItem.remoteId] as string;
                            if (newStatus.toLowerCase() === 'prossecing') newStatus = 'processing';
                            
                            if (newStatus.toLowerCase() !== taskItem.status?.toLowerCase()) {
                                let progress = taskItem.progress;
                                if (newStatus.toLowerCase() === 'completed') {
                                    progress = 100;
                                    showToast(`${t('queue.status_completed')}: ${taskItem.name}`, 'success');
                                } else if (newStatus.toLowerCase() === 'failed') {
                                    progress = 0;
                                    showToast(`${t('queue.status_failed')}: ${taskItem.name}`, 'error');
                                }
                                return { ...taskItem, status: newStatus as TaskStatus, progress };
                            }
                        }
                        return taskItem;
                    }));
                }
            } catch (err) {
                console.error("Polling error:", err);
            }
        };

        const intervalMs = hasQueued ? 5000 : 60000;
        const interval = setInterval(pollStatuses, intervalMs);
        return () => clearInterval(interval);
    }, [tasks]);

    const startQueue = useCallback(async () => {
        if (isProcessing) return;
        const pending = tasks.filter(t => t.status === 'pending');
        if (pending.length === 0) return;

        // Check validation before starting
        try {
            // @ts-ignore
            const savedKey = await window.go.main.App.GetSavedAuthKey();
            const sessionKey = sessionStorage.getItem('current_auth_key');
            const key = savedKey || sessionKey || "";

            // @ts-ignore
            const response = await window.go.main.App.ValidateKey(key);

            if (!response || !response.valid) {
                showToast(t('auth.error_expired'), 'error');
                return;
            }
        } catch (e: any) {
            console.error("Queue start auth check failed:", e);
            const errMsg = e?.toString() || "";
            const lowerMsg = errMsg.toLowerCase();

            // If we are already in the app (starting queue), and there is an auth error, 
            // it's almost 100% a subscription issue, even if the server says something generic.
            if (lowerMsg.includes("expired") || lowerMsg.includes("subscription") || lowerMsg.includes("403")) {
                showToast(t('auth.error_expired'), 'error');
            } else if (lowerMsg.includes("hardware")) {
                showToast(t('auth.error_hardware_mismatch'), 'error');
            } else {
                // If we are here, something is wrong, and since the user already logged in before,
                // the most likely culprit is still the subscription/access.
                showToast(t('auth.error_expired'), 'error');
            }
            return;
        }

        setIsProcessing(true); const startTime = Date.now();
        totalPausedTimeRef.current = 0;
        pauseStartRef.current = null;
        totalMontageTimeRef.current = 0;
        completedMontageCountRef.current = 0;
        montageStartTimesRef.current.clear();

        await ResetQueueCancellation();
        const pendingIds = pending.map(t => t.id);
        activeBatchRef.current = pendingIds;
        hasShownImageBatchNotificationRef.current = false;
        setIsImageBatchReady(false);
        hasShownMontageBatchNotificationRef.current = false;

        // Prepare montage synchronization for this batch
        const controlledTaskIds = pending
            .filter(t => {
                const s = t.settings as any;
                // Settings can be nested or flat depending on where they come from
                const mEnabled = s?.montageEnabled ?? s?.stages?.montage;
                const mControl = s?.montageControlEnabled ?? s?.control?.montage;
                return mEnabled && mControl;
            })
            .map(t => t.id);

        // @ts-ignore
        if (window.go?.main?.App?.PrepareMontageBatch && controlledTaskIds.length > 0) {
            // @ts-ignore
            await window.go.main.App.PrepareMontageBatch(controlledTaskIds);
        }

        setTasks(prev => prev.map(t => pendingIds.includes(t.id) ? {
            ...t,
            status: 'waiting',
            textStatus: t.textStatus === 'pending' ? 'waiting' : t.textStatus,
            voiceStatus: t.voiceStatus === 'pending' ? 'waiting' : t.voiceStatus,
            imageStatus: t.imageStatus === 'pending' ? 'waiting' : t.imageStatus,
            subtitleStatus: t.subtitleStatus === 'pending' ? 'waiting' : t.subtitleStatus,
            montageStatus: t.montageStatus === 'pending' ? 'waiting' : t.montageStatus,
            progress: 0
        } : t));

        // Use a small timeout to let the state update propagate before starting Go processes
        // to avoid race conditions with immediate 'completed' events.
        await new Promise(resolve => setTimeout(resolve, 50));

        const run = async () => {
            const promises = pending.map(async (task) => {
                try {
                    const content = taskContentRef.current.get(task.id) || "";
                    const res = await ProcessTask(task.id, task.taskNumber || 0, task.type, content, task.settings, task.folderName, task.subName);
                    updateTaskStatus(task.id, 'completed', 100, res.length);
                } catch (err) {
                    updateTaskStatus(task.id, 'failed', 0);
                }
            });
            try { await Promise.all(promises); } finally {
                setIsProcessing(false); activeBatchRef.current = [];
                
                // Finish current pause if active
                let finalPausedTime = totalPausedTimeRef.current;
                if (pauseStartRef.current !== null) {
                    finalPausedTime += Date.now() - pauseStartRef.current;
                }

                const totalMs = Date.now() - startTime;
                const activeMs = totalMs - finalPausedTime;
                
                const durStr = formatDuration(totalMs);
                const activeDurStr = formatDuration(activeMs);
                const totalMontageStr = formatDuration(totalMontageTimeRef.current);
                const avgMontageMs = completedMontageCountRef.current > 0 ? totalMontageTimeRef.current / completedMontageCountRef.current : 0;
                const avgMontageStr = formatDuration(avgMontageMs);

                setTimeout(() => setCompletionModal({ 
                    isOpen: true, 
                    taskCount: pending.length, 
                    duration: durStr,
                    activeDuration: activeDurStr,
                    total_montage: totalMontageStr,
                    avg_montage: avgMontageStr
                }), 800);

                // Send Telegram Notification if enabled
                const msg = `${t('notifications.queue_completed_title')}\n\n` +
                    `${t('notifications.queue_completed_msg')}\n` +
                    `${t('queue.tasks_completed')}: ${pending.length}\n` +
                    `${t('queue.total_duration')}: ${durStr}\n` +
                    `${t('queue.active_duration')}: ${activeDurStr}\n` +
                    `${t('queue.total_montage')}: ${totalMontageStr}\n` +
                    `${t('queue.avg_montage')}: ${avgMontageStr}`;
                    
                await sendNotification(msg);
            }
        };
        run();
    }, [tasks, isProcessing, updateTaskStatus, t]);

    useEffect(() => {
        const uStatus = EventsOn("taskStatus", (id: string, s: string, p: number, l?: number) => {
            setTasks(prev => prev.map(t => t.id === id ? { ...t, status: s as TaskStatus, progress: p, resultLength: l ?? t.resultLength } : t));
        });
        const uStage = EventsOn("stageStatus", (id: string, stage: string, status: string, msg?: string) => {
            if (stage === 'montage') {
                if (status === 'running') {
                    if (!montageStartTimesRef.current.has(id)) {
                        montageStartTimesRef.current.set(id, Date.now());
                    }
                } else if (status === 'completed') {
                    const start = montageStartTimesRef.current.get(id);
                    if (start) {
                        totalMontageTimeRef.current += Date.now() - start;
                        completedMontageCountRef.current += 1;
                        montageStartTimesRef.current.delete(id);
                    }
                }
            }

            setTasks(prev => prev.map(t => {
                if (t.id !== id) return t; const s = status as TaskStatus; const up: any = {};
                if (stage === 'text') up.textStatus = s;
                else if (stage === 'voice') { up.voiceStatus = s; if (msg) up.voiceDuration = msg; }
                else if (stage === 'image') { up.imageStatus = s; if (msg) up.imagesMessage = msg; }
                else if (stage === 'subtitle') up.subtitleStatus = s;
                else if (stage === 'montage') { up.montageStatus = s; if (msg) up.montageMsg = msg; }
                return { ...t, ...up };
            }));
        });
        const uReq = EventsOn("requestControl", (id: string, text: string) => {
            const task = tasksRef.current.find(t => t.id === id);
            if (task) {
                const msg = `${t('notifications.review_translation_title')}\n\n*${t('notifications.task_name')}*: ${task.subName || task.name}\n*${t('notifications.template')}*: ${task.folderName}`;
                sendNotification(msg);
            }
            taskContentRef.current.set(id, text); // Update content when control is requested
            setTasks(prev => prev.map(t => t.id === id ? { ...t, isAwaitingControl: true, controlContent: text } : t));
        });
        const uImgReq = EventsOn("requestImageControl", (id: string) => {
            setTasks(prev => prev.map(t => t.id === id ? { ...t, isAwaitingImageControl: true } : t));
        });
        const uMontageReq = EventsOn("requestMontageControl", (id: string, planData: string) => {
            const task = tasksRef.current.find(t => t.id === id);
            if (task) {
                const msg = `${t('pipeline.montage_control') || 'Montage Review'}\n\n*${t('notifications.task_name')}*: ${task.subName || task.name}`;
                sendNotification(msg);
            }
            setTasks(prev => prev.map(t => t.id === id ? { ...t, isAwaitingMontageControl: true, montagePlanData: planData } : t));
        });
        const uFilesReq = EventsOn("requestExistingFilesCheck", (data: any) => {
            setTasks(prev => prev.map(t => t.id === data.id ? { ...t, isAwaitingExistingFilesCheck: true, existingFilesData: data } : t));
        });
        const uTextResult = EventsOn("textResult", (id: string, length: number) => {
            setTasks(prev => prev.map(t => t.id === id ? { ...t, resultLength: length } : t));
        });
        const uRemoteTask = EventsOn("remoteTaskClaimed", (data: any) => {
            const { id, name, payload, settings } = data;
            if (taskContentRef.current.has(id)) return;
            taskContentRef.current.set(id, payload);
            
            const type = settings?.taskType || (name.toLowerCase().includes('rewrite') ? 'rewrite' : 'translate');
            
            const newTask: QueueTask = {
                id,
                name: name, folderName: name, subName: "",
                type: type, content: "", settings,
                status: 'waiting', progress: 0,
                textStatus: 'waiting', voiceStatus: 'waiting',
                imageStatus: 'waiting', subtitleStatus: 'waiting',
                montageStatus: 'waiting', timestamp: Date.now(),
            };
            setTasks(prev => [...prev, newTask]);
        });
        return () => { uStatus(); uStage(); uReq(); uImgReq(); uMontageReq(); uFilesReq(); uTextResult(); uRemoteTask(); };
    }, [t]);

    useEffect(() => {
        if (!isProcessing || activeBatchRef.current.length === 0) return;

        const bTasks = tasks.filter(t => activeBatchRef.current.includes(t.id));

        // 1. Image Control logic
        if (!hasShownImageBatchNotificationRef.current) {
            const awaitingImg = bTasks.filter(t => t.isAwaitingImageControl);
            const stillWorkingImg = bTasks.filter(t =>
                t.settings.imageEnabled &&
                t.settings.imageControlEnabled &&
                !t.isAwaitingImageControl &&
                t.imageStatus !== 'completed' &&
                t.imageStatus !== 'failed' &&
                t.status !== 'failed'
            );

            if (awaitingImg.length > 0 && stillWorkingImg.length === 0) {
                hasShownImageBatchNotificationRef.current = true;
                setImageControlNotification({ isOpen: true });
                setIsImageBatchReady(true);
                sendNotification(`${t('notifications.review_images_title')}\n\n${t('pipeline.image_control_notification.message')}`);
            }
        }

        // 2. Montage Control logic
        if (!hasShownMontageBatchNotificationRef.current) {
            const awaitingMontage = bTasks.filter(t => t.isAwaitingMontageControl);
            const stillWorkingMontage = bTasks.filter(t =>
                t.settings.montageEnabled &&
                t.settings.montageControlEnabled &&
                !t.isAwaitingMontageControl &&
                t.montageStatus !== 'completed' &&
                t.montageStatus !== 'failed' &&
                t.status !== 'failed'
            );

            if (awaitingMontage.length > 0 && stillWorkingMontage.length === 0) {
                hasShownMontageBatchNotificationRef.current = true;
                setMontageControlNotification({ isOpen: true });
            }
        }
    }, [tasks, isProcessing]);

    const dataValue = {
        tasks, isProcessing, completionModal, imageControlNotification, isImageBatchReady, montageControlNotification, regeneratingPaths
    };

    const actionsValue = {
        addTasks, addTask, removeTask, clearQueue, startQueue, startRemoteQueue, getNextTaskName,
        updateTaskStatus, resumeTask, regenerateTask, cancelTask, cancelQueue, resumeImageControl, resumeMontageControl, resumeWithExistingFiles,
        closeCompletionModal, closeImageControlNotification, closeMontageControlNotification,
        addRegeneratingPath, removeRegeneratingPath
    };

    return (
        <QueueDataContext.Provider value={dataValue}>
            <QueueActionsContext.Provider value={actionsValue}>
                {children}
            </QueueActionsContext.Provider>
        </QueueDataContext.Provider>
    );
};

export const useQueue = () => {
    const data = useContext(QueueDataContext);
    const actions = useContext(QueueActionsContext);
    if (!data || !actions) throw new Error('useQueue must be used within a QueueProvider');
    return { ...data, ...actions };
};

export const useQueueActions = () => {
    const context = useContext(QueueActionsContext);
    if (!context) throw new Error('useQueueActions must be used within a QueueProvider');
    return context;
};

export const useQueueData = () => {
    const context = useContext(QueueDataContext);
    if (!context) throw new Error('useQueueData must be used within a QueueProvider');
    return context;
};
