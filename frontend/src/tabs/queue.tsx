import React, { useState, useEffect } from 'react';
import { createPortal } from 'react-dom';
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

const VoiceIcon = () => (
    <svg className="voice-icon" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12,2C9.24,2,7,4.24,7,7v5c0,2.76,2.24,5,5,5s5-2.24,5-5V7C17,4.24,14.76,2,12,2z M12,14c-1.1,0-2-0.9-2-2V7 c0-1.1,0.9-2,2-2s2,0.9,2,2v5C14,13.1,13.1,14,12,14z M19,12c0,3.53-2.61,6.43-6,6.92V21h-2v-2.08c-3.39-0.49-6-3.39-6-6.92h2 c0,2.76,2.24,5,5,5s5-2.24,5-5H19z" />
    </svg>
);

const ImageIcon = () => (
    <svg className="image-icon" viewBox="0 0 24 24" fill="currentColor">
        <path d="M21 19V5c0-1.1-.9-2-2-2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2zM8.5 13.5l2.5 3.01L14.5 12l4.5 6H5l3.5-4.5z" />
    </svg>
);

const SubtitleIcon = () => (
    <svg className="subtitle-icon" viewBox="0 0 24 24" fill="currentColor">
        <path d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm-6 10H6v-2h8v2zm4-4H6V8h12v2z" />
    </svg>
);

const MontageIcon = () => (
    <svg className="montage-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
    </svg>
);

const ControlEditor = ({ task, onConfirm }: { task: QueueTask, onConfirm: (id: string, text: string) => void }) => {
    const [text, setText] = useState(task.controlContent || '');
    const [isFullScreen, setIsFullScreen] = useState(false);
    const { t } = useI18n();

    const origLen = task.content?.length || 0;
    const currLen = text.length;

    const editorContent = (
        <div className={`control-editor-overlay ${isFullScreen ? 'full-screen' : ''}`} onClick={(e) => e.stopPropagation()}>
            <div className="control-editor-content">
                <div className="control-editor-header">
                    <h3>{t('queue.control_title') || 'ПЕРЕВІРКА ТЕКСТУ'}</h3>
                    <button
                        className="control-expand-btn"
                        onClick={() => setIsFullScreen(!isFullScreen)}
                        title={isFullScreen ? "Зменшити" : "Розгорнути"}
                    >
                        {isFullScreen ? (
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M8 3v5H3M21 8h-5V3M3 16h5v5M16 21v-5h5" /></svg>
                        ) : (
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M15 3h6v6M9 21H3v-6M21 3l-7 7M3 21l7-7" /></svg>
                        )}
                    </button>
                </div>

                <div className="control-stats">
                    <div className="stat-item">
                        <span className="stat-label">Оригінал:</span>
                        <span className="stat-value">{origLen}</span>
                    </div>
                    <div className="stat-item">
                        <span className="stat-label">Переклад:</span>
                        <span className={`stat-value ${currLen > origLen * 1.2 ? 'warning' : ''}`}>{currLen}</span>
                    </div>
                </div>

                <textarea
                    className="control-textarea premium-scrollbar"
                    value={text}
                    onChange={(e) => setText(e.target.value)}
                    autoFocus
                />
                <div className="control-actions">
                    <button className="control-ok-btn" onClick={() => onConfirm(task.id, text)}>
                        OK
                    </button>
                </div>
            </div>
        </div>
    );

    if (isFullScreen) {
        return createPortal(editorContent, document.body);
    }

    return editorContent;
};

const renderStatusLines = (message: string, isFinished: boolean) => {
    if (!message) return null;
    return message.split('\n').map((line, idx) => {
        const match = line.match(/^(\w+):\s*(\d+)\/(\d+)$/i);
        if (match) {
            const [, , currentStr, totalStr] = match;
            const current = parseInt(currentStr, 10);
            const total = parseInt(totalStr, 10);
            const isWarning = isFinished && total > 0 && current < total;
            return (
                <div key={idx} style={{ color: isWarning ? '#ffa500' : 'inherit' }}>
                    {line}
                </div>
            );
        }
        return <div key={idx}>{line}</div>;
    });
};

export const Queue = ({ setCurrentPath }: QueueProps) => {
    const { t } = useI18n();
    const { tasks, removeTask, clearQueue, startQueue, isProcessing, resumeTask } = useQueue();
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
        const settings = task.settings || {};

        const isMainStageEnabled = task.type === 'translate' ? settings.translateEnabled !== false : settings.rewriteEnabled !== false;
        const isVoiceEnabled = settings.voiceoverEnabled === true;
        const isSubtitleEnabled = settings.subtitleEnabled === true;
        const isImageEnabled = settings.imageEnabled === true;
        const isMontageEnabled = settings.montageEnabled === true;

        const cleanStageLabel = (label: string) => label.replace(/^[\d\.A-Z]+\s*/, '');

        const mainLabel = isMainStageEnabled
            ? (task.type === 'translate' ? t('text.translate') : t('text.rewrite'))
            : t('text.original');

        const displayMainLabel = cleanStageLabel(mainLabel);

        return (
            <div key={task.id} className={`task-card-wrapper ${isExpanded ? 'expanded' : ''}`}>
                <div
                    className={`task-card animate-sidebar-item ${isExpanded ? 'active' : ''} ${task.isAwaitingControl ? 'awaiting-control' : ''}`}
                    onClick={() => !task.isAwaitingControl && toggleExpand(task.id)}
                >
                    {task.isAwaitingControl && (
                        <ControlEditor task={task} onConfirm={resumeTask} />
                    )}
                    <div className="task-card-header">
                        <span className={`task-type-badge ${task.type}`}>
                            {displayMainLabel}
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
                        {/* Текст */}
                        <div className={`task-stage-item status-${task.textStatus}`}>
                            <div className="stage-left">
                                <LightbulbIcon />
                                <span>{displayMainLabel}</span>
                            </div>
                            <span className="stage-status-text badge-status">
                                {task.textStatus === 'completed' ? `${task.resultLength || 0} chars` :
                                    task.textStatus === 'running' ? (t('queue.status_running') || 'Processing...') :
                                        task.textStatus === 'waiting' ? (t('queue.status_waiting') || 'В черзі') :
                                            task.textStatus === 'failed' ? t('queue.status_failed') : t('queue.status_pending')}
                            </span>
                        </div>

                        {/* Етап 2: Озвучка (якщо увімкнено) */}
                        {isVoiceEnabled && (
                            <div className={`task-stage-item status-${task.voiceStatus}`} style={{ marginTop: '4px' }}>
                                <div className="stage-left">
                                    <VoiceIcon />
                                    <span>{cleanStageLabel(t('text.voiceover'))}</span>
                                </div>
                                <span className="stage-status-text badge-status">
                                    {task.voiceStatus === 'completed' ? (task.voiceDuration || t('queue.voice_saved') || 'MP3 saved') :
                                        task.voiceStatus === 'running' ? (t('queue.status_running') || 'Synthesizing...') :
                                            task.voiceStatus === 'waiting' ? (t('queue.status_waiting') || 'Waiting') :
                                                task.voiceStatus === 'failed' ? t('queue.status_failed') : t('queue.status_pending')}
                                </span>
                            </div>
                        )}

                        {/* Картинки */}
                        {isImageEnabled && (
                            <div className={`task-stage-item status-${task.imageStatus}`} style={{ marginTop: '4px' }}>
                                <div className="stage-left">
                                    <ImageIcon />
                                    <span>{cleanStageLabel(t('pipeline.stage.image'))}</span>
                                </div>
                                <div className="stage-status-text badge-status" style={{ whiteSpace: 'pre-wrap', textAlign: 'right', fontSize: '11px', lineHeight: '1.4' }}>
                                    {task.imageStatus === 'completed' ? (
                                        task.imagesMessage ? renderStatusLines(task.imagesMessage, true) : (t('queue.image_saved') || 'Images Gen.')
                                    ) : task.imageStatus === 'running' ? (
                                        task.imagesMessage ? renderStatusLines(task.imagesMessage, false) : (t('queue.status_running') || 'Generating...')
                                    ) : task.imageStatus === 'waiting' ? (
                                        t('queue.status_waiting') || 'Waiting'
                                    ) : task.imageStatus === 'failed' ? (
                                        task.imagesMessage ? renderStatusLines(task.imagesMessage, true) : t('queue.status_failed')
                                    ) : t('queue.status_pending')}
                                </div>
                            </div>
                        )}

                        {/* Субтитри */}
                        {isSubtitleEnabled && (
                            <div className={`task-stage-item status-${task.subtitleStatus}`} style={{ marginTop: '4px' }}>
                                <div className="stage-left">
                                    <SubtitleIcon />
                                    <span>{cleanStageLabel(t('pipeline.stage.subtitle'))}</span>
                                </div>
                                <span className="stage-status-text badge-status">
                                    {task.subtitleStatus === 'completed' ? (t('queue.subtitle_saved') || 'SRT збережено') :
                                        task.subtitleStatus === 'running' ? (t('queue.status_running') || 'Transcribing...') :
                                            task.subtitleStatus === 'waiting' ? (t('queue.status_waiting') || 'Waiting') :
                                                task.subtitleStatus === 'failed' ? t('queue.status_failed') : t('queue.status_pending')}
                                </span>
                            </div>
                        )}

                        {/* Етап 5: Монтаж (якщо увімкнено) */}
                        {isMontageEnabled && (
                            <div className={`task-stage-item status-${task.montageStatus}`} style={{ marginTop: '4px' }}>
                                <div className="stage-left">
                                    <MontageIcon />
                                    <span>{cleanStageLabel(t('pipeline.stage.montage'))}</span>
                                </div>
                                <div className="stage-status-text badge-status" style={{ whiteSpace: 'pre-wrap', textAlign: 'right', fontSize: '11px', lineHeight: '1.4' }}>
                                    {task.montageStatus === 'completed' ? t('queue.status_completed') :
                                        task.montageStatus === 'running' ? (
                                            task.montageMsg ? renderStatusLines(task.montageMsg, false) : t('queue.status_running')
                                        ) : task.montageStatus === 'waiting' ? t('queue.status_waiting') :
                                            task.montageStatus === 'failed' ? (
                                                task.montageMsg ? renderStatusLines(task.montageMsg, true) : t('queue.status_failed')
                                            ) : t('queue.status_pending')}
                                </div>
                            </div>
                        )}
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
            <div className="queue-header">
                <div className="queue-title">ЧЕРГА ЗАВДАНЬ</div>
                <div style={{ display: 'flex', gap: '12px', alignItems: 'center' }}>
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
