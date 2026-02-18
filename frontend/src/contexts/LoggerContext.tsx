import React, { createContext, useContext, useState, ReactNode, useCallback, useEffect } from 'react';
// @ts-ignore
import { EventsOn } from '../../wailsjs/runtime';

export type LogLevel = 'INFO' | 'ERROR' | 'WARN' | 'DEBUG' | 'SUCCESS';

export interface LogEntry {
    id: string;
    timestamp: Date;
    level: LogLevel;
    message: string;
    taskId?: string; // ID завдання, якщо лог належить до нього
    taskLabel?: string; // Назва або номер завдання для відображення в загальному лозі
}

interface LoggerContextType {
    logs: LogEntry[];
    addLog: (level: LogLevel, message: string, taskId?: string, taskLabel?: string) => void;
    clearLogs: () => void;
}

const LoggerContext = createContext<LoggerContextType | undefined>(undefined);

export const useLogger = () => {
    const context = useContext(LoggerContext);
    if (!context) {
        throw new Error('useLogger must be used within a LoggerProvider');
    }
    return context;
};

export const LoggerProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
    const [logs, setLogs] = useState<LogEntry[]>([]);

    const addLog = useCallback((level: LogLevel, message: string, taskId?: string, taskLabel?: string) => {
        const newLog: LogEntry = {
            id: Math.random().toString(36).substr(2, 9),
            timestamp: new Date(),
            level,
            message,
            taskId,
            taskLabel
        };
        setLogs(prevLogs => [newLog, ...prevLogs]);
    }, []);

    const clearLogs = useCallback(() => {
        setLogs([]);
    }, []);

    useEffect(() => {
        const unsubscribe = EventsOn('log', (level: LogLevel, message: string, taskId?: string, taskLabel?: string) => {
            addLog(level, message, taskId, taskLabel);
        });
        return () => {
            if (unsubscribe) unsubscribe();
        };
    }, [addLog]);

    return (
        <LoggerContext.Provider value={{ logs, addLog, clearLogs }}>
            {children}
        </LoggerContext.Provider>
    );
};
