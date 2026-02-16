import React, { createContext, useContext, useState, ReactNode, useCallback } from 'react';

export type LogLevel = 'INFO' | 'ERROR' | 'WARN' | 'DEBUG';

export interface LogEntry {
    id: string;
    timestamp: Date;
    level: LogLevel;
    message: string;
}

interface LoggerContextType {
    logs: LogEntry[];
    addLog: (level: LogLevel, message: string) => void;
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

    const addLog = useCallback((level: LogLevel, message: string) => {
        const newLog: LogEntry = {
            id: Math.random().toString(36).substr(2, 9),
            timestamp: new Date(),
            level,
            message,
        };
        setLogs(prevLogs => [newLog, ...prevLogs]);
    }, []);

    const clearLogs = useCallback(() => {
        setLogs([]);
    }, []);

    return (
        <LoggerContext.Provider value={{ logs, addLog, clearLogs }}>
            {children}
        </LoggerContext.Provider>
    );
};
