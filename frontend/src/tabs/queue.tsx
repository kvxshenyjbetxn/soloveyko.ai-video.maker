import React, { useState, useEffect, useMemo, useCallback } from 'react';
import { createPortal } from 'react-dom';
import { useI18n } from '../contexts/I18nContext';
import './queue.css';
import { useQueue, QueueTask } from '../contexts/QueueContext';
import { useLogger } from '../contexts/LoggerContext';
import { ConfirmModal } from '../components/ConfirmModal';
import { ExistingFilesModal } from '../components/ExistingFilesModal';
import { MontageEditor } from '../components/MontageEditor';
import { VirtualLogList } from '../components/VirtualLogList';
// @ts-ignore
import { GetOpenRouterSavedModels, GetPipelineSettings, OpenPath, ResolveTaskDir, GetAvailableWorkers } from '../../wailsjs/go/main/App';

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

const FolderIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>
);

const ControlEditor = ({ task, onConfirm }: { task: QueueTask, onConfirm: (id: string, text: string) => void }) => {
    const { regenerateTask, cancelTask } = useQueue();
    const [text, setText] = useState(task.controlContent || '');
    const [isFullScreen, setIsFullScreen] = useState(false);
    const [showSettings, setShowSettings] = useState(false);

    const [prompt, setPrompt] = useState(task.settings?.translatePrompt || task.settings?.rewritePrompt || '');
    const [model, setModel] = useState(task.settings?.translateModel || task.settings?.rewriteModel || '');
    const [temperature, setTemperature] = useState(task.settings?.temperature || 0.7);
    const [maxTokens, setMaxTokens] = useState(task.settings?.maxTokens || 2000);
    const [savedModels, setSavedModels] = useState<string[]>([]);

    useEffect(() => {
        const fetchModels = async () => {
            try {
                const models = await GetOpenRouterSavedModels();
                if (models) setSavedModels(models);
            } catch (err) {
                console.error("Failed to fetch OpenRouter models:", err);
            }
        };
        fetchModels();
    }, []);

    const { t } = useI18n();

    const origLen = task.originalLength || task.content?.length || 0;
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

                {showSettings && (
                    <div className="control-settings-menu animate-fade-in">
                        <div className="settings-grid">
                            <div className="settings-field">
                                <label>{t('queue.prompt_label')}</label>
                                <textarea
                                    value={prompt}
                                    onChange={(e) => setPrompt(e.target.value)}
                                    className="premium-scrollbar"
                                />
                            </div>
                            <div className="settings-row">
                                <div className="settings-field">
                                    <label>{t('queue.model_label')}</label>
                                    {savedModels.length > 0 ? (
                                        <select value={model} onChange={(e) => setModel(e.target.value)}>
                                            {savedModels.map(m => (
                                                <option key={m} value={m}>{m}</option>
                                            ))}
                                        </select>
                                    ) : (
                                        <input type="text" value={model} onChange={(e) => setModel(e.target.value)} />
                                    )}
                                </div>
                                <div className="settings-field">
                                    <label>{t('queue.temp_label')}</label>
                                    <input type="number" step="0.1" value={temperature} onChange={(e) => setTemperature(parseFloat(e.target.value))} />
                                </div>
                                <div className="settings-field">
                                    <label>{t('queue.max_tokens_label')}</label>
                                    <input type="number" value={maxTokens} onChange={(e) => setMaxTokens(parseInt(e.target.value, 10))} />
                                </div>
                            </div>
                        </div>
                        <button
                            className="apply-regenerate-btn"
                            onClick={() => {
                                const newSettings = { ...task.settings };
                                if (task.type === 'translate') {
                                    newSettings.translatePrompt = prompt;
                                    newSettings.translateModel = model;
                                } else {
                                    newSettings.rewritePrompt = prompt;
                                    newSettings.rewriteModel = model;
                                }
                                newSettings.temperature = temperature;
                                newSettings.maxTokens = maxTokens;
                                regenerateTask(task.id, text, newSettings);
                            }}
                        >
                            {t('queue.apply_and_regenerate')}
                        </button>
                    </div>
                )}

                <div className="control-actions">
                    <div style={{ flex: 1 }} />
                    <button className="control-cancel-btn" onClick={() => cancelTask(task.id)} title={t('common.cancel')}>
                        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10" /><line x1="15" y1="9" x2="9" y2="15" /><line x1="9" y1="9" x2="15" y2="15" /></svg>
                        {isFullScreen && <span>{t('common.cancel')}</span>}
                    </button>
                    <button className="control-settings-btn" onClick={() => {
                        if (!isFullScreen) {
                            setIsFullScreen(true);
                            setShowSettings(true);
                        } else {
                            setShowSettings(!showSettings);
                        }
                    }} title={t('queue.edit_settings')}>
                        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" /></svg>
                    </button>
                    <button className="control-regen-btn" onClick={() => regenerateTask(task.id, text)} title={t('queue.regenerate')}>
                        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="23 4 23 10 17 10" /><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" /></svg>
                        {isFullScreen && <span>{t('queue.regenerate')}</span>}
                    </button>
                    <button className="control-ok-btn" onClick={() => onConfirm(task.id, text)}>OK</button>
                </div>
            </div>
        </div>
    );

    if (isFullScreen) return createPortal(editorContent, document.body);
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

const MagicWandIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="m21.64 3.64-1.28-1.28a1.21 1.21 0 0 0-1.72 0L2.36 18.64a1.21 1.21 0 0 0 0 1.72l1.28 1.28a1.21 1.21 0 0 0 1.72 0L21.64 5.36a1.21 1.21 0 0 0 0-1.72Z"></path><path d="m14 7 3 3"></path><path d="M5 6v.01"></path><path d="M19 14v.01"></path><path d="M10 2v.01"></path><path d="M7 21v.01"></path><path d="M14 22v.01"></path></svg>
);

const TaskItem = React.memo(({ task, isExpanded, onToggle, onRemove, onOpenFolder, onOpenMontageEditor, isProcessing, t, resumeTask, logs }: any) => {
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
    const taskLogs = useMemo(() => logs.filter((l: any) => l.taskId === task.id), [logs, task.id]);

    return (
        <div className={`task-card-wrapper ${isExpanded ? 'expanded' : ''}`}>
            <div
                className={`task-card animate-sidebar-item ${isExpanded ? 'active' : ''} ${task.isAwaitingControl ? 'awaiting-control' : ''} ${task.isAwaitingMontageControl ? 'awaiting-montage-control' : ''}`}
                onClick={() => !task.isAwaitingControl && onToggle(task.id)}
                style={{ position: 'relative', overflow: 'hidden' }}
            >
                {task.workerName && (
                    <div className="remote-execution-overlay" style={{
                        position: 'absolute',
                        top: 0,
                        left: 0,
                        right: 0,
                        bottom: 0,
                        backgroundColor: 'rgba(0, 0, 0, 0.5)',
                        backdropFilter: 'blur(4px) grayscale(0.2)',
                        borderRadius: '12px',
                        zIndex: 10,
                        pointerEvents: 'none',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        border: '2px dashed var(--accent-color-transparent)'
                    }}>
                        <div style={{
                            backgroundColor: 'var(--bg-primary)',
                            padding: '12px 24px',
                            borderRadius: '14px',
                            border: '1.5px solid var(--accent-color)',
                            boxShadow: '0 8px 30px rgba(0,0,0,0.5), 0 0 20px var(--accent-color-transparent)',
                            display: 'flex',
                            flexDirection: 'column',
                            alignItems: 'center',
                            gap: '6px',
                            transform: 'rotate(-7deg)',
                            maxWidth: '85%'
                        }}>
                            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="var(--accent-color)" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><rect x="2" y="2" width="20" height="8" rx="2" ry="2"/><rect x="2" y="14" width="20" height="8" rx="2" ry="2"/><line x1="6" y1="6" x2="6.01" y2="6"/><line x1="6" y1="18" x2="6.01" y2="18"/></svg>
                                <span style={{ 
                                    color: 'var(--accent-color)', 
                                    fontSize: '10px', 
                                    fontWeight: 800, 
                                    letterSpacing: '1.5px',
                                    textTransform: 'uppercase',
                                    whiteSpace: 'nowrap'
                                }}>
                                    {t('common.remote_worker_title')}
                                </span>
                            </div>
                            <span style={{ 
                                color: 'white', 
                                fontSize: '14px', 
                                fontWeight: 700,
                                textAlign: 'center'
                            }}>
                                {task.workerName}
                            </span>
                        </div>
                    </div>
                )}
                {task.isAwaitingControl && (
                    <ControlEditor task={task} onConfirm={resumeTask} />
                )}
                <div className="task-card-header">
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                        <span className={`task-type-badge ${task.type}`}>
                            {displayMainLabel}
                        </span>
                    </div>
                    <div className="task-card-header-actions">
                        {settings.montageControlEnabled && (
                            <button
                                className={`open-folder-task-btn ${task.isAwaitingMontageControl ? 'pulse-btn active' : ''}`}
                                onClick={(e) => {
                                    e.stopPropagation();
                                    if (task.isAwaitingMontageControl && onOpenMontageEditor) {
                                        onOpenMontageEditor(task);
                                    }
                                }}
                                title={t('pipeline.montage_control') || 'Montage Control'}
                                style={{ 
                                    marginRight: '4px',
                                    opacity: task.isAwaitingMontageControl ? 1 : 0.5,
                                    cursor: task.isAwaitingMontageControl ? 'pointer' : 'not-allowed',
                                    color: task.isAwaitingMontageControl ? 'var(--accent-color)' : 'inherit',
                                    borderColor: task.isAwaitingMontageControl ? 'var(--accent-color)' : 'transparent'
                                }}
                            >
                                <MagicWandIcon />
                            </button>
                        )}
                        <button
                            className="open-folder-task-btn"
                            onClick={(e) => {
                                e.stopPropagation();
                                onOpenFolder(task);
                            }}
                            title={t('common.open_folder') || 'Open folder'}
                        >
                            <FolderIcon />
                        </button>
                        <button
                            className="remove-task-btn"
                            disabled={isProcessing}
                            onClick={(e) => {
                                e.stopPropagation();
                                onRemove(task.id);
                            }}
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
                        </button>
                    </div>
                </div>

                <div className="task-card-name-container">
                    <div className="task-card-folder-name" title={task.folderName}>{task.folderName}</div>
                    {task.subName && (
                        <div className="task-card-sub-name" title={task.subName}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><polyline points="9 18 15 12 9 6"></polyline></svg>
                            {task.subName}
                        </div>
                    )}
                </div>

                <div className="task-stages-list">
                    <div className={`task-stage-item status-${task.textStatus}`}>
                        <div className="stage-left"><LightbulbIcon /><span>{displayMainLabel}</span></div>
                        <span className="stage-status-text badge-status">
                            {task.textStatus === 'completed' ? (t('queue.chars', { count: task.resultLength || 0 }) || `${task.resultLength || 0} chars`) :
                                task.textStatus === 'running' ? (t('queue.status_running') || 'Processing...') :
                                    task.textStatus === 'waiting' ? (t('queue.status_waiting') || 'В черзі') :
                                        task.textStatus === 'failed' ? t('queue.status_failed') : t('queue.status_pending')}
                        </span>
                    </div>

                    {isVoiceEnabled && (
                        <div className={`task-stage-item status-${task.voiceStatus}`} style={{ marginTop: '4px' }}>
                            <div className="stage-left"><VoiceIcon /><span>{cleanStageLabel(t('text.voiceover'))}</span></div>
                            <span className="stage-status-text badge-status">
                                {task.voiceStatus === 'completed' ? (task.voiceDuration || t('queue.voice_saved') || 'MP3 saved') :
                                    task.voiceStatus === 'running' ? (t('queue.status_running') || 'Synthesizing...') :
                                        task.voiceStatus === 'waiting' ? (t('queue.status_waiting') || 'Waiting') :
                                            task.voiceStatus === 'failed' ? t('queue.status_failed') : t('queue.status_pending')}
                            </span>
                        </div>
                    )}

                    {isImageEnabled && (
                        <div className={`task-stage-item status-${task.imageStatus}`} style={{ marginTop: '4px' }}>
                            <div className="stage-left"><ImageIcon /><span>{cleanStageLabel(t('pipeline.stage.image'))}</span></div>
                            <div className="stage-status-text badge-status" style={{ whiteSpace: 'pre-wrap', textAlign: 'right', fontSize: '10px' }}>
                                {task.imageStatus === 'completed' ? (
                                    task.imagesMessage ? renderStatusLines(task.imagesMessage, true) : (t('queue.status_completed') || 'Completed')
                                ) : task.imageStatus === 'running' ? (
                                    task.imagesMessage ? renderStatusLines(task.imagesMessage, false) : (t('queue.status_running') || 'Generating...')
                                ) : task.imageStatus === 'waiting' ? (t('queue.status_waiting') || 'Waiting') :
                                    task.imageStatus === 'failed' ? (
                                        task.imagesMessage ? renderStatusLines(task.imagesMessage, true) : t('queue.status_failed')
                                    ) : t('queue.status_pending')}
                            </div>
                        </div>
                    )}

                    {isSubtitleEnabled && (
                        <div className={`task-stage-item status-${task.subtitleStatus}`} style={{ marginTop: '4px' }}>
                            <div className="stage-left"><SubtitleIcon /><span>{cleanStageLabel(t('pipeline.stage.subtitle'))}</span></div>
                            <span className="stage-status-text badge-status">
                                {task.subtitleStatus === 'completed' ? (t('queue.subtitle_saved') || 'SRT збережено') :
                                    task.subtitleStatus === 'running' ? (t('queue.status_running') || 'Transcribing...') :
                                        task.subtitleStatus === 'waiting' ? (t('queue.status_waiting') || 'Waiting') :
                                            task.subtitleStatus === 'failed' ? t('queue.status_failed') : t('queue.status_pending')}
                            </span>
                        </div>
                    )}

                    {isMontageEnabled && (
                        <div className={`task-stage-item status-${task.montageStatus}`} style={{ marginTop: '4px' }}>
                            <div className="stage-left"><MontageIcon /><span>{cleanStageLabel(t('pipeline.stage.montage'))}</span></div>
                            <div className="stage-status-text badge-status" style={{ whiteSpace: 'pre-wrap', textAlign: 'right', fontSize: '10px' }}>
                                {task.montageStatus === 'completed' ? (task.montageMsg || t('queue.status_completed')) :
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



                <div className="task-card-footer">{new Date(task.timestamp).toLocaleTimeString()}</div>
            </div>

            <div className="task-inline-log" onClick={(e) => e.stopPropagation()}>
                <div className="log-header"><span className="log-title">{t('tabs.logs')}</span></div>
                <div className="log-content premium-scrollbar" style={{ overflow: 'hidden' }}>
                    {taskLogs.length === 0 ? (
                        <div className="log-empty">
                            <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z" /><polyline points="14 2 14 8 20 8" /><line x1="16" y1="13" x2="8" y2="13" /><line x1="16" y1="17" x2="8" y2="17" /><line x1="10" y1="9" x2="8" y2="9" /></svg>
                            <span>{t('logsTab.empty')}</span>
                        </div>
                    ) : (
                        <VirtualLogList
                            logs={taskLogs}
                            rowHeight={28}
                            renderRow={(log: any) => (
                                <div
                                    key={log.id}
                                    className={`task-log-entry level-${log.level.toLowerCase()}`}
                                    title={log.message}
                                    onClick={(e) => {
                                        e.stopPropagation();
                                        navigator.clipboard.writeText(log.message);
                                        const target = e.currentTarget as HTMLElement;
                                        const originalBg = target.style.backgroundColor;
                                        target.style.backgroundColor = 'rgba(255, 255, 255, 0.15)';
                                        setTimeout(() => { target.style.backgroundColor = originalBg; }, 200);
                                    }}
                                    style={{
                                        height: '28px',
                                        display: 'flex',
                                        alignItems: 'center',
                                        gap: '8px',
                                        padding: '0 8px',
                                        boxSizing: 'border-box',
                                        borderBottom: '1px solid rgba(255,255,255,0.02)',
                                        cursor: 'copy'
                                    }}
                                >
                                    <span className="task-log-time" style={{ minWidth: '65px', fontSize: '10px', flexShrink: 0 }}>{log.timestamp.toLocaleTimeString()}</span>
                                    <span className="task-log-message" style={{
                                        wordBreak: 'break-word',
                                        whiteSpace: 'pre-wrap',
                                        fontSize: '11px',
                                        lineHeight: '1.2',
                                        maxHeight: '24px',
                                        overflow: 'hidden',
                                        display: '-webkit-box',
                                        WebkitLineClamp: 2,
                                        WebkitBoxOrient: 'vertical'
                                    }}>{log.message}</span>
                                </div>
                            )}
                        />
                    )}
                </div>
            </div>
        </div>
    );
});

export const Queue = ({ setCurrentPath }: QueueProps) => {
    const { t } = useI18n();
    const { tasks, removeTask, clearQueue, startQueue, startRemoteQueue, isProcessing, resumeTask, resumeWithExistingFiles, resumeMontageControl } = useQueue();
    const { logs } = useLogger();
    const [expandedTaskIds, setExpandedTaskIds] = useState<string[]>([]);
    const [confirmModal, setConfirmModal] = useState<{ isOpen: boolean; title: string; message?: string; onConfirm: () => void; }>({ isOpen: false, title: '', onConfirm: () => { } });
    const [activeMontageTask, setActiveMontageTask] = useState<QueueTask | null>(null);

    const [workers, setWorkers] = useState<any[]>([]);
    const [selectedWorkerId, setSelectedWorkerId] = useState<string>("");
    const [isWorkerDropdownOpen, setIsWorkerDropdownOpen] = useState(false);

    useEffect(() => {
        const fetchWorkers = async () => {
            try {
                // @ts-ignore
                const data = await GetAvailableWorkers();
                if (data) setWorkers(data);
            } catch (e) {
                console.error("Failed to fetch workers:", e);
            }
        };
        fetchWorkers();
        const interval = setInterval(fetchWorkers, 30000);
        return () => clearInterval(interval);
    }, []);

    // Close dropdown on click outside
    useEffect(() => {
        if (!isWorkerDropdownOpen) return;
        const handleClick = () => setIsWorkerDropdownOpen(false);
        window.addEventListener('click', handleClick);
        return () => window.removeEventListener('click', handleClick);
    }, [isWorkerDropdownOpen]);

    const handleStartQueue = () => {
        if (selectedWorkerId) {
            const workerName = selectedWorker?.hostname || selectedWorker?.name || "Remote Worker";
            startRemoteQueue(selectedWorkerId, workerName);
        } else {
            startQueue();
        }
    };

    useEffect(() => {
        if (tasks.length === 0 && setCurrentPath) {
            const timer = setTimeout(() => setCurrentPath('text.translate'), 300);
            return () => clearTimeout(timer);
        }
    }, [tasks.length, setCurrentPath]);

    const handleClearQueue = useCallback(() => {
        if (isProcessing) return;
        setConfirmModal({
            isOpen: true, title: t('queue.clear_all'), message: t('queue.delete_all_confirm'),
            onConfirm: () => { clearQueue(); if (setCurrentPath) setCurrentPath('text.translate'); setConfirmModal(prev => ({ ...prev, isOpen: false })); }
        });
    }, [isProcessing, clearQueue, setCurrentPath, t]);

    const handleRemoveTask = useCallback((id: string) => {
        if (isProcessing) return;
        setConfirmModal({
            isOpen: true, title: t('common.delete'), message: t('queue.delete_confirm'),
            onConfirm: () => { removeTask(id); setConfirmModal(prev => ({ ...prev, isOpen: false })); }
        });
    }, [isProcessing, removeTask, t]);

    const toggleExpand = useCallback((id: string) => {
        setExpandedTaskIds(prev => prev.includes(id) ? prev.filter(tid => tid !== id) : [...prev, id]);
    }, []);

    const handleOpenFolder = useCallback(async (task: QueueTask) => {
        try {
            const path = await ResolveTaskDir(task.folderName, task.type, task.subName, task.settings || {});
            if (path) {
                await OpenPath(path);
            }
        } catch (err) {
            console.error("Failed to open task folder:", err);
        }
    }, []);

    const handleOpenMontageEditor = useCallback((task: QueueTask) => {
        setActiveMontageTask(task);
    }, []);

    const handleMontageConfirm = useCallback((taskId: string, resultData: string) => {
        resumeMontageControl(taskId, resultData);
        setActiveMontageTask(null);
    }, [resumeMontageControl]);

    const handleMontageCancel = useCallback((taskId: string) => {
        setActiveMontageTask(null);
    }, []);

    const selectedWorker = workers.find(w => w.hardware_id === selectedWorkerId);

    return (
        <div className="content-wrapper animate-fade">
            <div className="queue-header">
                <div className="queue-title">ЧЕРГА ЗАВДАНЬ</div>
                <div style={{ display: 'flex', gap: '12px', alignItems: 'center', position: 'relative' }}>
                    {tasks.length > 0 && !isProcessing && (
                        <div className="custom-worker-selector" onClick={(e) => { e.stopPropagation(); setIsWorkerDropdownOpen(!isWorkerDropdownOpen); }} style={{
                            display: 'flex',
                            alignItems: 'center',
                            backgroundColor: 'rgba(255, 255, 255, 0.03)',
                            border: '1px solid rgba(255, 255, 255, 0.08)',
                            borderRadius: '8px',
                            padding: '0 12px',
                            gap: '10px',
                            height: '36px',
                            cursor: 'pointer',
                            transition: 'all 0.2s cubic-bezier(0.4, 0, 0.2, 1)',
                            userSelect: 'none',
                            minWidth: '180px',
                            boxShadow: isWorkerDropdownOpen ? '0 0 0 2px var(--accent-color-transparent)' : 'none',
                            borderColor: isWorkerDropdownOpen ? 'var(--accent-color)' : 'rgba(255, 255, 255, 0.08)'
                        }}>
                            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flex: 1 }}>
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke={selectedWorkerId ? 'var(--accent-color)' : 'currentColor'} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" style={{ opacity: selectedWorkerId ? 1 : 0.6 }}><rect x="2" y="2" width="20" height="8" rx="2" ry="2"/><rect x="2" y="14" width="20" height="8" rx="2" ry="2"/><line x1="6" y1="6" x2="6.01" y2="6"/><line x1="6" y1="18" x2="6.01" y2="18"/></svg>
                                <span style={{ fontSize: '13px', fontWeight: 500, color: 'rgba(255,255,255,0.9)' }}>
                                    {selectedWorkerId ? (selectedWorker?.hostname || selectedWorker?.name || t('other.remote_control')) : t('common.local_processing')}
                                </span>
                            </div>
                            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" style={{ 
                                opacity: 0.4, 
                                transition: 'transform 0.3s ease',
                                transform: isWorkerDropdownOpen ? 'rotate(180deg)' : 'rotate(0deg)'
                            }}><path d="m6 9 6 6 6-6"/></svg>

                            {isWorkerDropdownOpen && (
                                <div className="worker-dropdown-menu animate-slide-down" style={{
                                    position: 'absolute',
                                    top: 'calc(100% + 8px)',
                                    right: 0,
                                    width: '100%',
                                    minWidth: '220px',
                                    backgroundColor: 'var(--bg-secondary)',
                                    backgroundImage: 'linear-gradient(rgba(255,255,255,0.03), rgba(255,255,255,0))',
                                    border: '1px solid var(--border-color)',
                                    borderRadius: '10px',
                                    padding: '6px',
                                    zIndex: 1000,
                                    boxShadow: '0 10px 25px -5px rgba(0,0,0,0.5), 0 8px 10px -6px rgba(0,0,0,0.5)',
                                    backdropFilter: 'blur(10px)'
                                }}>
                                    <div 
                                        className={`worker-dropdown-item ${!selectedWorkerId ? 'active' : ''}`}
                                        onClick={() => setSelectedWorkerId("")}
                                        style={{
                                            padding: '8px 10px',
                                            borderRadius: '6px',
                                            fontSize: '12px',
                                            display: 'flex',
                                            alignItems: 'center',
                                            gap: '10px',
                                            color: !selectedWorkerId ? 'white' : 'var(--text-secondary)',
                                            backgroundColor: !selectedWorkerId ? 'rgba(255,255,255,0.08)' : 'transparent',
                                            transition: 'all 0.15s ease'
                                        }}
                                    >
                                        <div style={{ width: '8px', height: '8px', borderRadius: '50%', backgroundColor: !selectedWorkerId ? 'var(--accent-color)' : 'rgba(255,255,255,0.2)' }} />
                                        <div style={{ flex: 1 }}>{t('common.local_processing')}</div>
                                        {!selectedWorkerId && <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="var(--accent-color)" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><polyline points="20 6 9 17 4 12"/></svg>}
                                    </div>

                                    {workers.length > 0 && <div style={{ height: '1px', backgroundColor: 'var(--border-color)', margin: '4px 6px' }} />}

                                    {workers.map(w => (
                                        <div 
                                            key={w.hardware_id}
                                            className={`worker-dropdown-item ${selectedWorkerId === w.hardware_id ? 'active' : ''}`}
                                            onClick={() => setSelectedWorkerId(w.hardware_id)}
                                            style={{
                                                padding: '8px 10px',
                                                borderRadius: '6px',
                                                fontSize: '12px',
                                                display: 'flex',
                                                alignItems: 'center',
                                                gap: '10px',
                                                color: selectedWorkerId === w.hardware_id ? 'white' : 'var(--text-secondary)',
                                                backgroundColor: selectedWorkerId === w.hardware_id ? 'rgba(255,255,255,0.08)' : 'transparent',
                                                transition: 'all 0.15s ease',
                                                marginTop: '2px'
                                            }}
                                        >
                                            <div style={{ width: '8px', height: '8px', borderRadius: '50%', backgroundColor: w.status === 'ready' ? '#4caf50' : '#ff9800' }} />
                                            <div style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
                                                <span style={{ fontWeight: 500 }}>{w.hostname || w.name}</span>
                                                <span style={{ fontSize: '10px', opacity: 0.5 }}>ID: {(w.hardware_id || '').substring(0, 8)}</span>
                                            </div>
                                            {selectedWorkerId === w.hardware_id && <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="var(--accent-color)" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><polyline points="20 6 9 17 4 12"/></svg>}
                                        </div>
                                    ))}

                                    {workers.length === 0 && (
                                        <div style={{ padding: '12px', textAlign: 'center', fontSize: '11px', opacity: 0.4 }}>
                                            {t('other.no_workers')}
                                        </div>
                                    )}
                                </div>
                            )}
                        </div>
                    )}
                    {tasks.length > 0 && (
                        <button className="clear-queue-btn" onClick={handleClearQueue} disabled={isProcessing}>{t('queue.clear_all') || 'Clear All'}</button>
                    )}
                    {tasks.length > 0 && (
                        <button className={`start-queue-btn ${isProcessing ? 'processing' : ''}`} onClick={handleStartQueue} disabled={isProcessing}>
                            {isProcessing ? (<><div className="spinner-small" /><span>{t('queue.processing')}</span></>) : (<><svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z" /></svg><span>{t('queue.start')}</span></>)}
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
                        {tasks.map(task => (
                            <TaskItem
                                key={task.id}
                                task={task}
                                isExpanded={expandedTaskIds.includes(task.id)}
                                onToggle={toggleExpand}
                                onRemove={handleRemoveTask}
                                onOpenFolder={handleOpenFolder}
                                onOpenMontageEditor={handleOpenMontageEditor}
                                isProcessing={isProcessing}
                                t={t}
                                resumeTask={resumeTask}
                                logs={logs}
                            />
                        ))}
                    </div>
                )}
            </div>

            <ConfirmModal isOpen={confirmModal.isOpen} onClose={() => setConfirmModal(prev => ({ ...prev, isOpen: false }))} onConfirm={confirmModal.onConfirm} title={confirmModal.title} message={confirmModal.message} />
            {tasks.some(t => t.isAwaitingExistingFilesCheck) && (
                <ExistingFilesModal
                    isOpen={true}
                    data={tasks.find(t => t.isAwaitingExistingFilesCheck)?.existingFilesData}
                    onConfirm={(skip) => resumeWithExistingFiles(tasks.find(t => t.isAwaitingExistingFilesCheck)!.id, skip)}
                    onCancel={() => resumeWithExistingFiles(tasks.find(t => t.isAwaitingExistingFilesCheck)!.id, [])}
                />
            )}
            {activeMontageTask && (
                <MontageEditor 
                    task={activeMontageTask} 
                    onConfirm={handleMontageConfirm} 
                    onCancel={handleMontageCancel} 
                />
            )}
        </div>
    );
};
