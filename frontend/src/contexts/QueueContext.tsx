import React, { createContext, useContext, useState, useCallback, ReactNode, useEffect, useRef } from 'react';
import { ProcessTask, SubmitImageControlResult, SubmitExistingFilesResult, ClearGallery, SendControlAction, CancelQueue, ResetQueueCancellation } from '../../wailsjs/go/main/App';
import { EventsOn } from '../../wailsjs/runtime/runtime';
import { useToast } from './ToastContext';
import { useI18n } from './I18nContext';

export type TaskStatus = 'pending' | 'waiting' | 'running' | 'processing' | 'completed' | 'failed';

export interface QueueTask {
    id: string; taskNumber?: number; name: string; folderName: string; subName: string;
    type: 'translate' | 'rewrite' | 'voiceover'; content: string; settings: any;
    status: TaskStatus; progress: number; resultLength?: number;
    isAwaitingControl?: boolean; isAwaitingImageControl?: boolean; isAwaitingMontageControl?: boolean; montagePlanData?: string; isAwaitingExistingFilesCheck?: boolean; controlContent?: string;
    existingFilesData?: any;
    textStatus: TaskStatus; voiceStatus: TaskStatus; imageStatus: TaskStatus;
    subtitleStatus: TaskStatus; montageStatus: TaskStatus; montageMsg?: string;
    voiceDuration?: string; imagesMessage?: string; timestamp: number;
}

interface QueueContextType {
    tasks: QueueTask[]; isProcessing: boolean;
    completionModal: { isOpen: boolean; duration: string; taskCount: number; };
    imageControlNotification: { isOpen: boolean; };
    montageControlNotification: { isOpen: boolean; };
    addTasks: (type: any, content: string, tasksData: any[], name?: string, skippedStages?: string[]) => void;
    addTask: (type: any, content: string, settings: any, name?: string, subName?: string, skippedStages?: string[], existingData?: any) => void;
    removeTask: (id: string) => void; clearQueue: () => void; startQueue: () => Promise<void>;
    getNextTaskName: () => string;
    updateTaskStatus: (id: string, s: TaskStatus, p?: number, l?: number) => void;
    resumeTask: (id: string, text: string) => Promise<void>;
    regenerateTask: (id: string, text: string, settings?: any) => Promise<void>;
    cancelTask: (id: string) => Promise<void>;
    cancelQueue: () => Promise<void>;
    resumeImageControl: () => Promise<void>;
    resumeMontageControl: (id: string, resultData: string) => Promise<void>;
    resumeWithExistingFiles: (id: string, skipStages: string[]) => Promise<void>;
    closeCompletionModal: () => void; closeImageControlNotification: () => void; closeMontageControlNotification: () => void;
    regeneratingPaths: Set<string>;
    addRegeneratingPath: (path: string) => void;
    removeRegeneratingPath: (path: string) => void;
}

const QueueContext = createContext<QueueContextType | undefined>(undefined);

export const QueueProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
    const { t } = useI18n(); const { showToast } = useToast();
    const [tasks, setTasks] = useState<QueueTask[]>([]);
    const [isProcessing, setIsProcessing] = useState(false);
    const [completionModal, setCompletionModal] = useState({ isOpen: false, duration: '', taskCount: 0 });
    const [imageControlNotification, setImageControlNotification] = useState({ isOpen: false });
    const [montageControlNotification, setMontageControlNotification] = useState({ isOpen: false });
    const taskCounterRef = useRef(1);
    const activeBatchRef = useRef<string[]>([]);
    const hasShownImageBatchNotificationRef = useRef(false);
    const hasShownMontageBatchNotificationRef = useRef(false);
    const tasksRef = useRef<QueueTask[]>([]);
    useEffect(() => { tasksRef.current = tasks; }, [tasks]);

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
            const sysEnabled = await window.go.main.App.GetSystemNotificationsEnabled();
            if (sysEnabled && ("Notification" in window) && Notification.permission === "granted") {
                new Notification("Soloveyko.AI", {
                    body: msg.replace(/\*/g, ''), // remove markdown bold for system notification
                    icon: '/icon.png'
                });
            }
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

        const newTask: QueueTask = {
            id: `t_${Date.now()}_${Math.random().toString(36).substr(2, 5)}`,
            name: subName ? `${fName} - ${subName}` : fName, folderName: fName, subName: subName || "",
            type, content, status: 'pending', progress: 0,
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

            return {
                id: `ts_${now}_${i}_${Math.random().toString(36).substr(2, 5)}`,
                name: d.subName ? `${fName} - ${d.subName}` : fName, folderName: fName, subName: d.subName || "",
                type, content,
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

    const removeTask = useCallback((id: string) => setTasks(prev => prev.filter(t => t.id !== id)), []);
    const clearQueue = useCallback(() => {
        setTasks([]);
        setIsProcessing(false);
        ClearGallery();
    }, []);

    const resumeTask = async (id: string, text: string) => {
        setTasks(prev => prev.map(t => t.id === id ? { ...t, isAwaitingControl: false, content: text } : t));
        await SendControlAction(id, "confirm", text, {});
    };

    const regenerateTask = async (id: string, text: string, settings?: any) => {
        setTasks(prev => prev.map(t => t.id === id ? { ...t, isAwaitingControl: false, status: 'running', textStatus: 'running' } : t));
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
        await ResetQueueCancellation();
        const pendingIds = pending.map(t => t.id);
        activeBatchRef.current = pendingIds;
        hasShownImageBatchNotificationRef.current = false;
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
                    const res = await ProcessTask(task.id, task.taskNumber || 0, task.type, task.content, task.settings, task.folderName, task.subName);
                    updateTaskStatus(task.id, 'completed', 100, res.length);
                } catch (err) {
                    updateTaskStatus(task.id, 'failed', 0);
                }
            });
            try { await Promise.all(promises); } finally {
                setIsProcessing(false); activeBatchRef.current = [];
                const dur = Math.round((Date.now() - startTime) / 1000);
                const h = Math.floor(dur / 3600);
                const m = Math.floor((dur % 3600) / 60);
                const s = dur % 60;
                let durStr = "";
                if (h > 0) durStr = `${h}${t('common.unit_h')} ${m}${t('common.unit_m')} ${s}${t('common.unit_s')}`;
                else if (m > 0) durStr = `${m}${t('common.unit_m')} ${s}${t('common.unit_s')}`;
                else durStr = `${s}${t('common.unit_s')}`;
                setTimeout(() => setCompletionModal({ isOpen: true, taskCount: pending.length, duration: durStr }), 800);

                // Send Telegram Notification if enabled
                const msg = `${t('notifications.queue_completed_title')}\n\n${t('notifications.queue_completed_msg')}\n${t('notifications.tasks_completed')}: ${pending.length}\n${t('notifications.duration')}: ${durStr}`;
                await sendNotification(msg);
            }
        };
        run();
    }, [tasks, isProcessing, updateTaskStatus]);

    useEffect(() => {
        const uStatus = EventsOn("taskStatus", (id: string, s: string, p: number, l?: number) => {
            setTasks(prev => prev.map(t => t.id === id ? { ...t, status: s as TaskStatus, progress: p, resultLength: l ?? t.resultLength } : t));
        });
        const uStage = EventsOn("stageStatus", (id: string, stage: string, status: string, msg?: string) => {
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
            setTasks(prev => prev.map(t => t.id === id ? { ...t, isAwaitingControl: true, controlContent: text } : t));
        });
        const uImgReq = EventsOn("requestImageControl", (id: string) => {
            const task = tasksRef.current.find(t => t.id === id);
            if (task) {
                const msg = `${t('notifications.review_images_title')}\n\n*${t('notifications.task_name')}*: ${task.subName || task.name}\n*${t('notifications.template')}*: ${task.folderName}`;
                sendNotification(msg);
            }
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
        return () => { uStatus(); uStage(); uReq(); uImgReq(); uMontageReq(); uFilesReq(); uTextResult(); };
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

    return (
        <QueueContext.Provider value={{
            tasks, isProcessing, completionModal, imageControlNotification, montageControlNotification,
            addTasks, addTask, removeTask, clearQueue, startQueue, getNextTaskName,
            updateTaskStatus, resumeTask, regenerateTask, cancelTask, cancelQueue, resumeImageControl, resumeMontageControl, resumeWithExistingFiles, closeCompletionModal, closeImageControlNotification, closeMontageControlNotification,
            regeneratingPaths, addRegeneratingPath, removeRegeneratingPath
        }}>{children}</QueueContext.Provider>
    );
};

export const useQueue = () => {
    const context = useContext(QueueContext);
    if (!context) throw new Error('useQueue must be used within a QueueProvider');
    return context;
};
