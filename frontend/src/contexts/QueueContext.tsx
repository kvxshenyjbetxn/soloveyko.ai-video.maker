import React, { createContext, useContext, useState, useCallback, ReactNode } from 'react';

export type TaskStatus = 'pending' | 'running' | 'completed' | 'failed';

export interface QueueTask {
    id: string;
    name: string;
    type: 'translate' | 'rewrite';
    content: string;
    status: TaskStatus;
    progress: number;
    timestamp: number;
    settings: any;
}

interface QueueContextType {
    tasks: QueueTask[];
    addTask: (type: 'translate' | 'rewrite', content: string, settings: any, name?: string) => void;
    removeTask: (id: string) => void;
    clearQueue: () => void;
    updateTaskStatus: (id: string, status: TaskStatus, progress?: number) => void;
}

const QueueContext = createContext<QueueContextType | undefined>(undefined);

export const QueueProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
    const [tasks, setTasks] = useState<QueueTask[]>([]);
    const [taskCounter, setTaskCounter] = useState(1);

    const addTask = useCallback((type: 'translate' | 'rewrite', content: string, settings: any, name?: string) => {
        const finalName = name?.trim() || `Task ${taskCounter}`;

        const newTask: QueueTask = {
            id: Math.random().toString(36).substr(2, 9),
            name: finalName,
            type,
            content,
            status: 'pending',
            progress: 0,
            timestamp: Date.now(),
            settings
        };

        setTasks(prev => [...prev, newTask]);
        if (!name?.trim()) {
            setTaskCounter(prev => prev + 1);
        }
    }, [taskCounter]);

    const removeTask = useCallback((id: string) => {
        setTasks(prev => prev.filter(t => t.id !== id));
    }, []);

    const clearQueue = useCallback(() => {
        setTasks([]);
    }, []);

    const updateTaskStatus = useCallback((id: string, status: TaskStatus, progress?: number) => {
        setTasks(prev => prev.map(t =>
            t.id === id ? { ...t, status, progress: progress ?? t.progress } : t
        ));
    }, []);

    return (
        <QueueContext.Provider value={{ tasks, addTask, removeTask, clearQueue, updateTaskStatus }}>
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
