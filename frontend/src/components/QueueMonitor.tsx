import React, { useState, useEffect, useRef } from 'react';
import './QueueMonitor.css';
import { useI18n } from '../contexts/I18nContext';
import { useQueue } from '../contexts/QueueContext';

interface QueueMonitorProps {
    navigateTo?: (path: string) => void;
}

export const QueueMonitor = ({ navigateTo }: QueueMonitorProps) => {
    const { t } = useI18n();
    const { tasks, removeTask, startQueue, isProcessing } = useQueue();
    const [isExpanded, setIsExpanded] = useState(false);
    const [isPinned, setIsPinned] = useState(false);
    const wrapperRef = useRef<HTMLDivElement>(null);

    React.useEffect(() => {
        // @ts-ignore
        if (window.runtime) {
            // @ts-ignore
            const unsub = window.runtime.EventsOn("monitor-opened", (id: string) => {
                if (id !== 'queue' && !isPinned) {
                    setIsExpanded(false);
                }
            });
            return () => unsub();
        }
    }, [isPinned]);

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (wrapperRef.current && !wrapperRef.current.contains(event.target as Node) && isExpanded && !isPinned) {
                setIsExpanded(false);
            }
        };

        document.addEventListener('mousedown', handleClickOutside);
        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
        };
    }, [isExpanded, isPinned]);

    const handleExpand = (val: boolean) => {
        setIsExpanded(val);
        if (val) {
            // @ts-ignore
            window.runtime?.EventsEmit("monitor-opened", 'queue');
        }
    };

    if (tasks.length === 0) return null;

    const runningTasks = tasks.filter(t => t.status === 'running').length;
    const pendingTasksCount = tasks.filter(t => t.status === 'pending').length;

    return (
        <div className={`queue-monitor-wrapper ${isExpanded ? 'expanded' : ''} ${isPinned ? 'pinned' : ''}`} ref={wrapperRef}>
            {/* Expanded Panel */}
            <div className={`queue-mini-panel`}>
                <div className="queue-mini-header">
                    <span className="queue-mini-title">{t('tabs.queue')}</span>
                    <div style={{ display: 'flex', gap: '4px', alignItems: 'center' }}>
                        <button
                            className={`pin-btn ${isPinned ? 'active' : ''}`}
                            onClick={() => setIsPinned(!isPinned)}
                            title={isPinned ? t('common.unpin') : t('common.pin')}
                        >
                            <svg width="14" height="14" viewBox="0 0 24 24" fill={isPinned ? "currentColor" : "none"} stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                                <line x1="12" y1="17" x2="12" y2="22"></line>
                                <path d="M5 17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.79-.9A2 2 0 0 1 15 10.76V6a3 3 0 0 0-3-3 3 3 0 0 0-3 3v4.76a2 2 0 0 1-1.11 1.79l-1.79.9A2 2 0 0 0 5 15.24Z"></path>
                            </svg>
                        </button>
                        <button
                            className={`mini-start-btn ${isProcessing ? 'processing' : ''}`}
                            onClick={(e) => {
                                e.stopPropagation();
                                startQueue();
                            }}
                            disabled={isProcessing}
                            title={t('queue.start')}
                        >
                            {isProcessing ? (
                                <div className="spinner-tiny" />
                            ) : (
                                <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z" /></svg>
                            )}
                        </button>
                        <button
                            className="go-to-queue-btn"
                            title={t('tabs.queue')}
                            onClick={() => {
                                navigateTo?.('queue');
                                setIsExpanded(false);
                            }}
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" /><polyline points="15 3 21 3 21 9" /><line x1="10" y1="14" x2="21" y2="3" /></svg>
                        </button>
                        <button className="queue-close-btn" onClick={() => setIsExpanded(false)}>×</button>
                    </div>
                </div>

                <div className="queue-mini-list premium-scrollbar">
                    {tasks.map((task) => (
                        <div key={task.id} className="queue-mini-item">
                            <div className="queue-mini-item-info">
                                <span className={`task-badge ${task.type}`}>{task.type === 'translate' ? 'TR' : 'RW'}</span>
                                <div style={{ display: 'flex', flexDirection: 'column', gap: '2px', minWidth: 0 }}>
                                    <span className="task-text-preview" title={task.name}>{task.name}</span>
                                    <span className="task-mini-status">
                                        {task.status === 'pending' ? t('queue.status_pending') :
                                            task.status === 'running' ? t('queue.status_running') :
                                                task.status === 'completed' ? t('queue.status_completed') : t('queue.status_failed')}
                                    </span>
                                </div>
                            </div>
                            <div className="queue-mini-item-status">
                                {task.status === 'running' && (
                                    <div className="mini-progress-ring">
                                        <svg viewBox="0 0 36 36" className="circular-chart">
                                            <path className="circle-bg"
                                                d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
                                            />
                                            <path className="circle"
                                                strokeDasharray={`${task.progress}, 100`}
                                                d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
                                            />
                                        </svg>
                                    </div>
                                )}
                                <button className="remove-mini-task" onClick={() => removeTask(task.id)}>
                                    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
                                </button>
                            </div>
                        </div>
                    ))}
                </div>
            </div>

            {/* Floating Circle Button */}
            <div
                className={`queue-monitor-circle ${runningTasks > 0 ? 'is-running' : ''}`}
                onClick={() => handleExpand(!isExpanded)}
            >
                {pendingTasksCount > 0 && <div className="queue-count-badge">{pendingTasksCount}</div>}
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><path d="M6 18H18" /><path d="M6 12H18" /><path d="M6 6H18" /><circle cx="3" cy="6" r="1" /><circle cx="3" cy="12" r="1" /><circle cx="3" cy="18" r="1" /></svg>

                {runningTasks > 0 && <div className="running-indicator"></div>}
            </div>
        </div>
    );
};
