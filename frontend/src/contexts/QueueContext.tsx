import React, { createContext, useContext, useState, useCallback, ReactNode, useEffect, useRef } from 'react';
import { ProcessTask, SubmitControlResult, SubmitImageControlResult, SubmitExistingFilesResult } from '../../wailsjs/go/main/App';
import { EventsOn } from '../../wailsjs/runtime/runtime';
import { useToast } from './ToastContext';
import { useI18n } from './I18nContext';

export type TaskStatus = 'pending' | 'waiting' | 'running' | 'processing' | 'completed' | 'failed';

export interface QueueTask {
    id: string; taskNumber?: number; name: string; folderName: string; subName: string;
    type: 'translate' | 'rewrite' | 'voiceover'; content: string; settings: any;
    status: TaskStatus; progress: number; resultLength?: number;
    isAwaitingControl?: boolean; isAwaitingImageControl?: boolean; isAwaitingExistingFilesCheck?: boolean; controlContent?: string;
    existingFilesData?: any;
    textStatus: TaskStatus; voiceStatus: TaskStatus; imageStatus: TaskStatus;
    subtitleStatus: TaskStatus; montageStatus: TaskStatus; montageMsg?: string;
    voiceDuration?: string; imagesMessage?: string; timestamp: number;
}

interface QueueContextType {
    tasks: QueueTask[]; isProcessing: boolean;
    completionModal: { isOpen: boolean; duration: string; taskCount: number; };
    imageControlNotification: { isOpen: boolean; };
    addTasks: (type: any, content: string, tasksData: any[], name?: string, skippedStages?: string[]) => void;
    addTask: (type: any, content: string, settings: any, name?: string, subName?: string, skippedStages?: string[], existingData?: any) => void;
    removeTask: (id: string) => void; clearQueue: () => void; startQueue: () => Promise<void>;
    getNextTaskName: () => string;
    updateTaskStatus: (id: string, s: TaskStatus, p?: number, l?: number) => void;
    resumeTask: (id: string, text: string) => Promise<void>; resumeImageControl: () => Promise<void>;
    resumeWithExistingFiles: (id: string, skipStages: string[]) => Promise<void>;
    closeCompletionModal: () => void; closeImageControlNotification: () => void;
}

const QueueContext = createContext<QueueContextType | undefined>(undefined);

export const QueueProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
    const { t } = useI18n(); const { showToast } = useToast();
    const [tasks, setTasks] = useState<QueueTask[]>([]);
    const [isProcessing, setIsProcessing] = useState(false);
    const [completionModal, setCompletionModal] = useState({ isOpen: false, duration: '', taskCount: 0 });
    const [imageControlNotification, setImageControlNotification] = useState({ isOpen: false });
    const taskCounterRef = useRef(1);
    const activeBatchRef = useRef<string[]>([]);
    const hasShownImageBatchNotificationRef = useRef(false);

    const closeCompletionModal = () => setCompletionModal(prev => ({ ...prev, isOpen: false }));
    const closeImageControlNotification = () => setImageControlNotification({ isOpen: false });

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
            if (existingData.promptCount > 0) parts.push(`prompts: ${existingData.promptCount}`);
            if (existingData.imageCount > 0) parts.push(`images: ${existingData.imageCount}`);
            if (existingData.videoCount > 0) parts.push(`videos: ${existingData.videoCount}`);
            imgMsg = parts.join('\n');
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
                if (existingData.promptCount > 0) parts.push(`prompts: ${existingData.promptCount}`);
                if (existingData.imageCount > 0) parts.push(`images: ${existingData.imageCount}`);
                if (existingData.videoCount > 0) parts.push(`videos: ${existingData.videoCount}`);
                imgMsg = parts.join('\n');
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
    const clearQueue = useCallback(() => { setTasks([]); setIsProcessing(false); }, []);

    const resumeTask = async (id: string, text: string) => {
        setTasks(prev => prev.map(t => t.id === id ? { ...t, isAwaitingControl: false, content: text } : t));
        await SubmitControlResult(id, text);
    };

    const resumeImageControl = async () => {
        setImageControlNotification({ isOpen: false });
        const ids = tasks.filter(t => t.isAwaitingImageControl).map(t => t.id);
        if (ids.length === 0) return;
        setTasks(prev => prev.map(t => ids.includes(t.id) ? { ...t, isAwaitingImageControl: false, imageStatus: 'processing' } : t));
        for (const id of ids) await SubmitImageControlResult(id);
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

        setIsProcessing(true); const startTime = Date.now();
        const pendingIds = pending.map(t => t.id);
        activeBatchRef.current = pendingIds;
        hasShownImageBatchNotificationRef.current = false;

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
                setTimeout(() => setCompletionModal({ isOpen: true, taskCount: pending.length, duration: dur > 60 ? `${Math.floor(dur / 60)}хв ${dur % 60}с` : `${dur}с` }), 800);
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
            setTasks(prev => prev.map(t => t.id === id ? { ...t, isAwaitingControl: true, controlContent: text } : t));
        });
        const uImgReq = EventsOn("requestImageControl", (id: string) => {
            setTasks(prev => prev.map(t => t.id === id ? { ...t, isAwaitingImageControl: true } : t));
        });
        const uFilesReq = EventsOn("requestExistingFilesCheck", (data: any) => {
            setTasks(prev => prev.map(t => t.id === data.id ? { ...t, isAwaitingExistingFilesCheck: true, existingFilesData: data } : t));
        });
        const uTextResult = EventsOn("textResult", (id: string, length: number) => {
            setTasks(prev => prev.map(t => t.id === id ? { ...t, resultLength: length } : t));
        });
        return () => { uStatus(); uStage(); uReq(); uImgReq(); uFilesReq(); uTextResult(); };
    }, []);

    useEffect(() => {
        if (!isProcessing || activeBatchRef.current.length === 0 || hasShownImageBatchNotificationRef.current) return;
        const bTasks = tasks.filter(t => activeBatchRef.current.includes(t.id));
        const allReady = bTasks.every(t => t.isAwaitingImageControl || t.status === 'completed' || t.status === 'failed');
        if (allReady && bTasks.some(t => t.isAwaitingImageControl)) {
            hasShownImageBatchNotificationRef.current = true; setImageControlNotification({ isOpen: true });
        }
    }, [tasks, isProcessing]);

    return (
        <QueueContext.Provider value={{
            tasks, isProcessing, completionModal, imageControlNotification,
            addTasks, addTask, removeTask, clearQueue, startQueue, getNextTaskName,
            updateTaskStatus, resumeTask, resumeImageControl, resumeWithExistingFiles, closeCompletionModal, closeImageControlNotification
        }}>{children}</QueueContext.Provider>
    );
};

export const useQueue = () => {
    const context = useContext(QueueContext);
    if (!context) throw new Error('useQueue must be used within a QueueProvider');
    return context;
};
