import React from 'react';
import { useI18n } from '../contexts/I18nContext';
import './queue.css';

import { useQueue } from '../contexts/QueueContext';

interface QueueProps {
    setCurrentPath?: (path: string) => void;
}

export const Queue = ({ setCurrentPath }: QueueProps) => {
    const { t } = useI18n();
    const { tasks, removeTask, clearQueue } = useQueue();

    const handleClearQueue = () => {
        clearQueue();
        if (setCurrentPath) {
            setCurrentPath('text.translate');
        }
    };

    return (
        <div className="content-wrapper animate-fade">
            <div className="queue-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <h2 style={{ fontSize: '24px', fontWeight: 800, margin: 0 }}>{t('queue.title')}</h2>
                {tasks.length > 0 && (
                    <button className="clear-queue-btn" onClick={handleClearQueue}>
                        {t('queue.clear_all') || 'Clear All'}
                    </button>
                )}
            </div>

            <div className="queue-container">
                {tasks.length === 0 ? (
                    <div className="queue-empty">
                        <svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1" strokeLinecap="round" strokeLinejoin="round" style={{ opacity: 0.2, marginBottom: '20px' }}><path d="M6 18H18" /><path d="M6 12H18" /><path d="M6 6H18" /><circle cx="3" cy="6" r="1" /><circle cx="3" cy="12" r="1" /><circle cx="3" cy="18" r="1" /></svg>
                        <p>{t('queue.empty')}</p>
                    </div>
                ) : (
                    <div className="tasks-list">
                        {tasks.map((task) => (
                            <div key={task.id} className="task-item animate-sidebar-item" style={{ background: 'rgba(255,255,255,0.03)', border: '1px solid var(--border-color)', borderRadius: '12px', padding: '16px', marginBottom: '12px' }}>
                                <div className="task-info" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '12px' }}>
                                    <div style={{ display: 'flex', gap: '12px' }}>
                                        <div className={`task-badge ${task.type}`} style={{
                                            padding: '4px 8px', borderRadius: '6px', fontSize: '10px', fontWeight: 800, color: 'white',
                                            background: task.type === 'translate' ? '#3f51b5' : '#9c27b0'
                                        }}>
                                            {task.type.toUpperCase()}
                                        </div>
                                        <div>
                                            <div className="task-name" style={{ fontSize: '14px', fontWeight: 600, color: 'var(--text-primary)', marginBottom: '4px' }}>
                                                {task.content.substring(0, 80)}{task.content.length > 80 ? '...' : ''}
                                            </div>
                                            <div style={{ fontSize: '11px', color: 'var(--text-secondary)' }}>
                                                {new Date(task.timestamp).toLocaleString()}
                                            </div>
                                        </div>
                                    </div>
                                    <div style={{ display: 'flex', gap: '8px' }}>
                                        <span className={`task-status ${task.status.toLowerCase()}`} style={{
                                            fontSize: '11px', fontWeight: 700, padding: '4px 10px', borderRadius: '20px',
                                            background: task.status === 'running' ? 'rgba(255, 193, 7, 0.1)' : task.status === 'completed' ? 'rgba(76, 175, 80, 0.1)' : 'rgba(255, 255, 255, 0.05)',
                                            color: task.status === 'running' ? '#FFC107' : task.status === 'completed' ? '#4caf50' : 'var(--text-secondary)'
                                        }}>
                                            {task.status.toUpperCase()}
                                        </span>
                                        <button
                                            onClick={() => removeTask(task.id)}
                                            style={{ background: 'transparent', border: 'none', color: 'var(--text-placeholder)', cursor: 'pointer', padding: '4px' }}
                                        >
                                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
                                        </button>
                                    </div>
                                </div>

                                {task.status === 'running' && (
                                    <div className="progress-container" style={{ height: '6px', background: 'rgba(255,255,255,0.05)', borderRadius: '3px', overflow: 'hidden' }}>
                                        <div
                                            className="progress-bar"
                                            style={{
                                                width: `${task.progress}%`,
                                                height: '100%',
                                                background: 'var(--accent-primary)',
                                                transition: 'width 0.3s ease'
                                            }}
                                        ></div>
                                    </div>
                                )}
                            </div>
                        ))}
                    </div>
                )}
            </div>
        </div>
    );
};
