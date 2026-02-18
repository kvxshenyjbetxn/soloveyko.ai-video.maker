import React, { useState, useEffect } from 'react';
import { useI18n } from '../contexts/I18nContext';
import './queue.css';
import { useQueue, QueueTask } from '../contexts/QueueContext';
import { useLogger } from '../contexts/LoggerContext';
import { ConfirmModal } from '../components/ConfirmModal';

interface QueueProps {
    setCurrentPath?: (path: string) => void;
}

const LightbulbIcon = () => (
    <svg className="lightbulb-icon" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12,2C8.14,2,5,5.14,5,9c0,2.38,1.19,4.47,3,5.74V17c0,0.55,0.45,1,1,1h6c0.55,0,1-0.45,1-1v-2.26 c1.81-1.27,3-3.36,3-5.74C19,5.14,15.86,2,12,2z M14,19c0,0.55-0.45,1-1,1h-2c-0.55,0-1-0.45-1-1v-1h4V19z" />
    </svg>
);

export const Queue = ({ setCurrentPath }: QueueProps) => {
    const { t } = useI18n();
    const { tasks, removeTask, clearQueue, startQueue, isProcessing } = useQueue();
    const { logs } = useLogger();
    const [expandedTaskIds, setExpandedTaskIds] = useState<string[]>([]);

    // Custom Modal State
    const [confirmModal, setConfirmModal] = useState<{
        isOpen: boolean;
        title: string;
        message?: string;
        onConfirm: () => void;
    }>({
        isOpen: false,
        title: '',
        onConfirm: () => { }
    });

    // Redirect if last task is removed
    useEffect(() => {
        if (tasks.length === 0 && setCurrentPath) {
            const timer = setTimeout(() => {
                setCurrentPath('text.translate');
            }, 300);
            return () => clearTimeout(timer);
        }
    }, [tasks.length, setCurrentPath]);

    const handleClearQueue = () => {
        if (isProcessing) return;
        setConfirmModal({
            isOpen: true,
            title: t('queue.clear_all'),
            message: t('queue.delete_all_confirm'),
            onConfirm: () => {
                clearQueue();
                if (setCurrentPath) setCurrentPath('text.translate');
                setConfirmModal(prev => ({ ...prev, isOpen: false }));
            }
        });
    };

    const handleRemoveTask = (id: string) => {
        if (isProcessing) return;
        setConfirmModal({
            isOpen: true,
            title: t('common.delete'),
            message: t('queue.delete_confirm'),
            onConfirm: () => {
                removeTask(id);
                setConfirmModal(prev => ({ ...prev, isOpen: false }));
            }
        });
    };

    const toggleExpand = (id: string) => {
        setExpandedTaskIds(prev =>
            prev.includes(id) ? prev.filter(tid => tid !== id) : [...prev, id]
        );
    };

    const renderTaskItem = (task: QueueTask) => {
        const isExpanded = expandedTaskIds.includes(task.id);

        return (
            <div key={task.id} className={`task-card-wrapper ${isExpanded ? 'expanded' : ''}`}>
                <div
                    className={`task-card animate-sidebar-item ${isExpanded ? 'active' : ''}`}
                    onClick={() => toggleExpand(task.id)}
                >
                    <div className="task-card-header">
                        <span className={`task-type-badge ${task.type}`}>
                            {task.type === 'translate' ? t('text.translate') : t('text.rewrite')}
                        </span>
                        <button
                            className="remove-task-btn"
                            disabled={isProcessing}
                            onClick={(e) => {
                                e.stopPropagation();
                                handleRemoveTask(task.id);
                            }}
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
                        </button>
                    </div>

                    <div className="task-card-name" title={task.name}>
                        {task.name}
                    </div>

                    <div className="task-stages-list">
                        <div className={`task-stage-item status-${task.status}`}>
                            <div className="stage-left">
                                <LightbulbIcon />
                                <span>{task.type === 'translate' ? t('text.translate') : t('text.rewrite')}</span>
                            </div>
                            <span className="stage-status-text badge-status">
                                {task.status === 'completed' ? `${task.resultLength || 0} chars` :
                                    task.status === 'running' ? `${task.progress}%` :
                                        task.status === 'waiting' ? 'В черзі' :
                                            task.status === 'failed' ? t('queue.status_failed') : t('queue.status_pending')}
                            </span>
                        </div>
                    </div>

                    {task.status === 'running' && (
                        <div className="progress-bar-container">
                            <div
                                className="progress-bar-fill"
                                style={{ width: `${task.progress}%` }}
                            ></div>
                        </div>
                    )}

                    <div className="task-card-footer">
                        {new Date(task.timestamp).toLocaleTimeString()}
                    </div>
                </div>

                <div className="task-inline-log" onClick={(e) => e.stopPropagation()}>
                    <div className="log-header">
                        <span className="log-title">{t('tabs.logs')}</span>
                    </div>
                    <div className="log-content premium-scrollbar">
                        {logs.filter(l => l.taskId === task.id).length === 0 ? (
                            <div className="log-empty">
                                <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z" /><polyline points="14 2 14 8 20 8" /><line x1="16" y1="13" x2="8" y2="13" /><line x1="16" y1="17" x2="8" y2="17" /><line x1="10" y1="9" x2="8" y2="9" /></svg>
                                <span>{t('logsTab.empty')}</span>
                            </div>
                        ) : (
                            <div className="task-logs-list">
                                {logs.filter(l => l.taskId === task.id).map(log => (
                                    <div key={log.id} className={`task-log-entry level-${log.level.toLowerCase()}`}>
                                        <span className="task-log-time">{log.timestamp.toLocaleTimeString()}</span>
                                        <span className="task-log-message">{log.message}</span>
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                </div>
            </div>
        );
    };

    return (
        <div className="content-wrapper animate-fade">
            <div className="queue-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <h2 style={{ fontSize: '24px', fontWeight: 800, margin: 0 }}>{t('queue.title')}</h2>
                <div style={{ display: 'flex', gap: '12px' }}>
                    {tasks.length > 0 && (
                        <button className="clear-queue-btn" onClick={handleClearQueue} disabled={isProcessing}>
                            {t('queue.clear_all') || 'Clear All'}
                        </button>
                    )}
                    {tasks.length > 0 && (
                        <button
                            className={`start-queue-btn ${isProcessing ? 'processing' : ''}`}
                            onClick={startQueue}
                            disabled={isProcessing}
                        >
                            {isProcessing ? (
                                <>
                                    <div className="spinner-small" />
                                    <span>{t('queue.processing')}</span>
                                </>
                            ) : (
                                <>
                                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z" /></svg>
                                    <span>{t('queue.start')}</span>
                                </>
                            )}
                        </button>
                    )}
                </div>
            </div>

            <div className="queue-container premium-scrollbar">
                {tasks.length === 0 ? (
                    <div className="queue-empty">
                        <svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1" strokeLinecap="round" strokeLinejoin="round" style={{ opacity: 0.1, marginBottom: '20px' }}><path d="M6 18H18" /><path d="M6 12H18" /><path d="M6 6H18" /><circle cx="3" cy="6" r="1" /><circle cx="3" cy="12" r="1" /><circle cx="3" cy="18" r="1" /></svg>
                        <p>{t('queue.empty')}</p>
                    </div>
                ) : (
                    <div className="tasks-list">
                        {tasks.map(renderTaskItem)}
                    </div>
                )}
            </div>

            <ConfirmModal
                isOpen={confirmModal.isOpen}
                onClose={() => setConfirmModal(prev => ({ ...prev, isOpen: false }))}
                onConfirm={confirmModal.onConfirm}
                title={confirmModal.title}
                message={confirmModal.message}
            />
        </div>
    );
};
