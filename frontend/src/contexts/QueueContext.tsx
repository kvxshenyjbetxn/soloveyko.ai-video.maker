// @ts-ignore
import { ProcessTask, SubmitControlResult } from '../../wailsjs/go/main/App';
import React, { createContext, useContext, useState, useCallback, useEffect, ReactNode, useRef } from 'react';
import { useToast } from './ToastContext';
import { useI18n } from './I18nContext';

export type TaskStatus = 'pending' | 'waiting' | 'running' | 'completed' | 'failed';

export interface QueueTask {
    id: string;
    name: string; // Відображається в UI (Завдання - Шаблон)
    folderName: string; // Базова папка завдання
    subName: string; // Підпапка шаблону
    type: 'translate' | 'rewrite' | 'voiceover';
    content: string;
    status: TaskStatus;
    textStatus: TaskStatus;
    voiceStatus: TaskStatus;
    imageStatus: TaskStatus;
    progress: number;
    timestamp: number;
    settings: any;
    resultLength?: number;
    taskNumber?: number; // Простий порядковий номер завдання
    isAwaitingControl?: boolean;
    controlContent?: string;
    voiceDuration?: string;
    imagesMessage?: string;
}

interface QueueContextType {
    tasks: QueueTask[];
    addTask: (type: 'translate' | 'rewrite' | 'voiceover', content: string, settings: any, name?: string, subName?: string) => void;
    addTasks: (type: 'translate' | 'rewrite' | 'voiceover', content: string, tasksData: { settings: any, subName?: string }[], name?: string) => void;
    removeTask: (id: string) => void;
    clearQueue: () => void;
    updateTaskStatus: (id: string, status: TaskStatus, progress?: number) => void;
    startQueue: () => Promise<void>;
    isProcessing: boolean;
    completionModal: {
        isOpen: boolean;
        duration: string;
        taskCount: number;
    };
    closeCompletionModal: () => void;
    resumeTask: (id: string, editedContent: string) => Promise<void>;
}

const QueueContext = createContext<QueueContextType | undefined>(undefined);

export const QueueProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
    const { t } = useI18n();
    const { showToast } = useToast();
    const [tasks, setTasks] = useState<QueueTask[]>([]);
    const [taskCounter, setTaskCounter] = useState(1);
    const taskCounterRef = useRef(1);
    const [isProcessing, setIsProcessing] = useState(false);
    const [completionModal, setCompletionModal] = useState({
        isOpen: false,
        duration: '',
        taskCount: 0
    });

    const closeCompletionModal = useCallback(() => {
        setCompletionModal(prev => ({ ...prev, isOpen: false }));
    }, []);

    const addTask = useCallback((type: 'translate' | 'rewrite' | 'voiceover', content: string, settings: any, name?: string, subName?: string) => {
        const currentCount = taskCounterRef.current;
        const baseName = t('queue.task_default_name') || 'Task';
        const folderName = name?.trim() || `${baseName} ${currentCount}`;
        const displayName = subName ? `${folderName} - ${subName}` : folderName;

        const newTask: QueueTask = {
            id: Math.random().toString(36).substr(2, 9),
            name: displayName,
            folderName: folderName,
            subName: subName || "",
            type,
            content,
            status: 'pending',
            textStatus: 'pending',
            voiceStatus: 'pending',
            imageStatus: 'pending',
            progress: 0,
            timestamp: Date.now(),
            settings,
            taskNumber: currentCount
        };

        setTasks(prev => [...prev, newTask]);

        // Оновлюємо реф та стан для майбутніх завдань
        taskCounterRef.current += 1;
        setTaskCounter(taskCounterRef.current);
    }, []);

    const addTasks = useCallback((type: 'translate' | 'rewrite' | 'voiceover', content: string, tasksData: { settings: any, subName?: string }[], name?: string) => {
        if (tasksData.length === 0) return;

        const currentCount = taskCounterRef.current;
        const baseName = t('queue.task_default_name') || 'Task';
        const folderName = name?.trim() || `${baseName} ${currentCount}`;

        const newTasks: QueueTask[] = tasksData.map(data => {
            const displayName = data.subName ? `${folderName} - ${data.subName}` : folderName;
            return {
                id: Math.random().toString(36).substr(2, 9),
                name: displayName,
                folderName: folderName,
                subName: data.subName || "",
                type,
                content,
                status: 'pending',
                textStatus: 'pending',
                voiceStatus: 'pending',
                imageStatus: 'pending',
                progress: 0,
                timestamp: Date.now(),
                settings: data.settings,
                taskNumber: currentCount
            };
        });

        setTasks(prev => [...prev, ...newTasks]);

        // Оновлюємо лічильник один раз для всієї пачки завдань
        taskCounterRef.current += 1;
        setTaskCounter(taskCounterRef.current);
    }, []);

    const removeTask = useCallback((id: string) => {
        setTasks(prev => prev.filter(t => t.id !== id));
    }, []);

    const clearQueue = useCallback(() => {
        setTasks([]);
    }, []);

    const updateTaskStatus = useCallback((id: string, status: TaskStatus, progress?: number, resultLength?: number) => {
        setTasks(prev => prev.map(t =>
            t.id === id ? {
                ...t,
                status,
                progress: progress ?? t.progress,
                resultLength: resultLength ?? t.resultLength,
                // Коли завдання завершено або впало, оновлюємо і стейджі для лампочок
                textStatus: status === 'completed' ? 'completed' : (status === 'failed' ? 'failed' : (status === 'waiting' ? 'waiting' : t.textStatus)),
                voiceStatus: status === 'completed' ? 'completed' : (status === 'failed' ? 'failed' : (status === 'waiting' ? 'waiting' : t.voiceStatus)),
                imageStatus: status === 'completed' ? 'completed' : (status === 'failed' ? 'failed' : (status === 'waiting' ? 'waiting' : t.imageStatus))
            } : t
        ));
    }, []);

    const updateStageStatus = useCallback((id: string, stage: 'text' | 'voice' | 'image', status: TaskStatus, message?: string) => {
        setTasks(prev => prev.map(t =>
            t.id === id ? {
                ...t,
                [stage === 'text' ? 'textStatus' : stage === 'image' ? 'imageStatus' : 'voiceStatus']: status,
                ...(stage === 'voice' && message ? { voiceDuration: message } : {}),
                ...(stage === 'image' && message ? { imagesMessage: message } : {})
            } : t
        ));
    }, []);

    const updateTaskControl = useCallback((id: string, text: string) => {
        setTasks(prev => prev.map(t =>
            t.id === id ? {
                ...t,
                status: 'running',
                isAwaitingControl: true,
                controlContent: text
            } : t
        ));
    }, []);

    const resumeTask = useCallback(async (id: string, editedContent: string) => {
        setTasks(prev => prev.map(t =>
            t.id === id ? {
                ...t,
                isAwaitingControl: false,
                controlContent: undefined,
                resultLength: editedContent.length,
                textStatus: 'completed'
            } : t
        ));
        await SubmitControlResult(id, editedContent);
    }, []);

    const updateTaskResultLength = useCallback((id: string, length: number) => {
        setTasks(prev => prev.map(t =>
            t.id === id ? {
                ...t,
                resultLength: length
            } : t
        ));
    }, []);

    const startQueue = useCallback(async () => {
        if (isProcessing) return;

        const pendingTasks = tasks.filter(t => t.status === 'pending');
        const count = pendingTasks.length;

        if (count === 0) {
            return;
        }

        const startTime = Date.now();
        setIsProcessing(true);
        showToast("Початок обробки черги...", "info", 2000);

        await Promise.all(pendingTasks.map(async (task) => {
            updateTaskStatus(task.id, 'waiting', 0);

            try {
                // Fallback для старих завдань, які ще в черзі
                const fName = task.folderName || task.name || "Task";
                const sName = task.subName || "";

                const result = await ProcessTask(task.id, task.taskNumber || 0, task.type, task.content, task.settings, fName, sName);
                updateTaskStatus(task.id, 'completed', 100, result.length);
            } catch (error) {
                console.error(`Task ${task.id} failed:`, error);
                updateTaskStatus(task.id, 'failed', 0);
                showToast(`Помилка: ${task.name} не вдалося обробити`, "error", 4000);
            }
        }));

        const endTime = Date.now();
        const durationSeconds = Math.round((endTime - startTime) / 1000);
        const durationText = durationSeconds > 60
            ? `${Math.floor(durationSeconds / 60)} хв ${durationSeconds % 60} сек`
            : `${durationSeconds} сек`;

        setIsProcessing(false);

        // Trigger OS Flash
        try {
            // @ts-ignore
            if (window.runtime && typeof window.runtime.WindowFlash === 'function') {
                // @ts-ignore
                window.runtime.WindowFlash(true);
            }
        } catch (e) {
            console.error("Failed to flash window:", e);
        }

        setCompletionModal({
            isOpen: true,
            duration: durationText,
            taskCount: count
        });

        showToast("Черга закінчила обробку!", "success", 5000);
    }, [tasks, isProcessing, updateTaskStatus, showToast]);

    useEffect(() => {
        // @ts-ignore
        if (window.runtime) {
            // @ts-ignore
            const unsubStatus = window.runtime.EventsOn("taskStatus", (id: string, status: string, progress: number) => {
                updateTaskStatus(id, status as TaskStatus, progress);
            });
            // @ts-ignore
            const unsubStage = window.runtime.EventsOn("stageStatus", (id: string, stage: string, status: string, message?: string) => {
                updateStageStatus(id, stage as 'text' | 'voice' | 'image', status as TaskStatus, message);
            });
            // @ts-ignore
            const unsubResult = window.runtime.EventsOn("textResult", (id: string, length: number) => {
                updateTaskResultLength(id, length);
            });
            // @ts-ignore
            const unsubControl = window.runtime.EventsOn("requestControl", (id: string, text: string) => {
                updateTaskControl(id, text);
            });
            return () => {
                unsubStatus();
                unsubStage();
                unsubResult();
                unsubControl();
            };
        }
    }, [updateTaskStatus, updateStageStatus]);

    return (
        <QueueContext.Provider value={{ tasks, addTask, addTasks, removeTask, clearQueue, updateTaskStatus, startQueue, isProcessing, completionModal, closeCompletionModal, resumeTask }}>
            {children}
        </QueueContext.Provider>
    );
};

export const useQueue = () => {
    const context = useContext(QueueContext);
    if (!context) {
        throw new Error('useQueue must be used within a QueueProvider');
    }
    return context;
};
