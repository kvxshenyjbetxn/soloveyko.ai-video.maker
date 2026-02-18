import { useState, useEffect, useRef, useCallback } from 'react';
import './PipelineSidebar.css';
import { useI18n } from '../contexts/I18nContext';
import { useQueue } from '../contexts/QueueContext';
import { useServices } from '../contexts/ServiceContext';
// @ts-ignore
import { GetPipelineSettings, SavePipelineSettings, GetOpenRouterSavedModels, SelectDirectory, GetDefaultVideosPath } from '../../wailsjs/go/main/App';

import { TaskNameModal } from './TaskNameModal';

interface PipelineSidebarProps {
    type: 'translate' | 'rewrite';
    isOpen: boolean;
    onToggle: () => void;
    content: string;
    setCurrentPath?: (path: string) => void;
}

export const PipelineSidebar: React.FC<PipelineSidebarProps> = ({ type, isOpen, onToggle, content, setCurrentPath }) => {
    const { t } = useI18n();
    const { addTask } = useQueue();
    const { openRouterKeys } = useServices();
    const [settings, setSettings] = useState<any>(null);
    const [models, setModels] = useState<string[]>([]);
    const [isResizing, setIsResizing] = useState(false);
    const [editingField, setEditingField] = useState<string | null>(null);
    const [isModalOpen, setIsModalOpen] = useState(false);

    const sidebarRef = useRef<HTMLDivElement>(null);
    const lastSavedRef = useRef<string>("");

    useEffect(() => {
        const init = async () => {
            try {
                const orModels = await GetOpenRouterSavedModels();
                const s = await GetPipelineSettings();

                const modelList = orModels || [];
                setModels(modelList);

                let updated = false;
                if (modelList.length > 0) {
                    if (s.translateModel === "") {
                        s.translateModel = modelList[0];
                        updated = true;
                    }
                    if (s.rewriteModel === "") {
                        s.rewriteModel = modelList[0];
                        updated = true;
                    }
                }

                if (openRouterKeys.length > 0) {
                    if (!s.translateOpenRouterKeyID) {
                        s.translateOpenRouterKeyID = openRouterKeys[0].id;
                        updated = true;
                    }
                    if (!s.rewriteOpenRouterKeyID) {
                        s.rewriteOpenRouterKeyID = openRouterKeys[0].id;
                        updated = true;
                    }
                }

                if (!s.rewriteEnabled) {
                    s.rewriteEnabled = true;
                    updated = true;
                }

                if (!s.outputPath) {
                    try {
                        const def = await GetDefaultVideosPath();
                        if (def) {
                            s.outputPath = def;
                            updated = true;
                        }
                    } catch (e) {
                        console.error("Failed to get default path:", e);
                    }
                }

                // Завжди згортаємо блок API та Шлях при ініціалізації
                s.apiCollapsed = true;
                s.pathCollapsed = true;
                updated = true;

                if (updated) {
                    await SavePipelineSettings(s);
                }

                setSettings(s);
                lastSavedRef.current = JSON.stringify(s);
            } catch (err) {
                console.error("Failed to initialize sidebar:", err);
            }
        };

        init();
    }, [type]);

    useEffect(() => {
        document.documentElement.style.setProperty('--sidebar-toggle-width', '36px');
        return () => {
            document.documentElement.style.setProperty('--sidebar-toggle-width', '0px');
        };
    }, []);

    useEffect(() => {
        const width = isOpen ? (settings?.sidebarWidth || 320) : 0;
        document.documentElement.style.setProperty('--pipeline-sidebar-width', `${width}px`);
        return () => {
            document.documentElement.style.setProperty('--pipeline-sidebar-width', '0px');
        };
    }, [settings?.sidebarWidth, isOpen]);

    const startResizing = useCallback((e: React.MouseEvent) => {
        setIsResizing(true);
        e.preventDefault();
    }, []);

    const stopResizing = useCallback(() => {
        setIsResizing(false);
    }, []);

    const resize = useCallback((e: MouseEvent) => {
        if (isResizing && sidebarRef.current) {
            const newWidth = window.innerWidth - e.pageX;
            if (newWidth >= 250 && newWidth <= 600) {
                setSettings((prev: any) => ({ ...prev, sidebarWidth: newWidth }));
            }
        }
    }, [isResizing]);

    useEffect(() => {
        if (!settings) return;

        const currentString = JSON.stringify(settings);
        if (currentString !== lastSavedRef.current) {
            const timer = setTimeout(() => {
                SavePipelineSettings(settings);
                lastSavedRef.current = currentString;
            }, 500);

            return () => clearTimeout(timer);
        }
    }, [settings]);

    useEffect(() => {
        if (isResizing) {
            window.addEventListener('mousemove', resize);
            window.addEventListener('mouseup', stopResizing);
        } else {
            window.removeEventListener('mousemove', resize);
            window.removeEventListener('mouseup', stopResizing);
        }

        return () => {
            window.removeEventListener('mousemove', resize);
            window.removeEventListener('mouseup', stopResizing);
        };
    }, [isResizing, resize, stopResizing]);

    const handleChange = (field: string, value: any) => {
        setSettings((prev: any) => ({
            ...prev,
            [field]: value
        }));
    };

    if (!settings) return null;

    const isTranslate = type === 'translate';
    const isEnabled = isTranslate ? settings.translateEnabled : settings.rewriteEnabled;
    const isCollapsed = isTranslate ? settings.translateCollapsed : settings.rewriteCollapsed;
    const isApiCollapsed = settings.apiCollapsed;
    const isPathCollapsed = settings.pathCollapsed;

    const modelValue = isTranslate ? settings.translateModel : settings.rewriteModel;
    const tempValue = isTranslate ? settings.translateTemperature : settings.rewriteTemperature;
    const tokensValue = isTranslate ? settings.translateMaxTokens : settings.rewriteMaxTokens;
    const promptValue = isTranslate ? settings.translatePrompt : settings.rewritePrompt;
    const selectedApiKeyID = isTranslate ? settings.translateOpenRouterKeyID : settings.rewriteOpenRouterKeyID;
    const outputPath = settings.outputPath || '';


    const toggleCollapse = () => {
        const field = isTranslate ? 'translateCollapsed' : 'rewriteCollapsed';
        handleChange(field, !isCollapsed);
    };

    const handleToggleEnable = (e: React.ChangeEvent<HTMLInputElement>) => {
        const val = e.target.checked;
        const newSettings = { ...settings, translateEnabled: val };
        if (!val) {
            newSettings.translateCollapsed = true;
        }
        setSettings(newSettings);
    };

    const handleSelectPath = async () => {
        try {
            const path = await SelectDirectory();
            if (path) {
                handleChange('outputPath', path);
            }
        } catch (err) {
            console.error("Failed to select path:", err);
        }
    };

    const renderValueOrInput = (field: string, value: number, isFloat: boolean) => {
        if (editingField === field) {
            return (
                <input
                    autoFocus
                    className="settings-value-input"
                    type="number"
                    defaultValue={value}
                    step={isFloat ? "0.1" : "500"}
                    onBlur={(e) => {
                        setEditingField(null);
                        let val = parseFloat(e.target.value);
                        if (isNaN(val)) val = value;
                        handleChange(field, val);
                    }}
                    onKeyDown={(e) => {
                        if (e.key === 'Enter') {
                            (e.target as HTMLInputElement).blur();
                        }
                    }}
                />
            );
        }

        let displayValue: string | number = isFloat ? value.toFixed(1) : value;
        if (!isFloat && value === 0 && field.includes('MaxTokens')) {
            displayValue = t('pipeline.max_tokens_unlimited');
        }

        return (
            <span
                className="settings-slider-value"
                onClick={(e) => {
                    e.stopPropagation();
                    setEditingField(field);
                }}
                style={!isFloat && value === 0 ? { minWidth: '80px', fontSize: '10px' } : {}}
            >
                {displayValue}
            </span>
        );
    };

    return (
        <aside
            className="pipeline-sidebar"
            ref={sidebarRef}
            style={{ width: `${isOpen ? (settings.sidebarWidth || 320) : 0}px` }}
        >
            <div
                className={`sidebar-resizer ${isResizing ? 'is-resizing' : ''}`}
                onMouseDown={startResizing}
            />

            <div className="sidebar-clipper">
                <div className="pipeline-sidebar-header">
                    <div className="pipeline-sidebar-title">{t(`pipeline.${type}.title`)}</div>
                </div>

                <div className="pipeline-sidebar-content">
                    {/* Pipeline Name Section */}
                    <div className="settings-group" style={{ marginBottom: '8px', borderBottom: '1px solid var(--border-color)', paddingBottom: '12px' }}>
                        <div className="settings-control">
                            <label className="settings-label">{t('pipeline.name')}</label>
                            <input
                                className="settings-input"
                                value={(isTranslate ? settings.translatePipelineName : settings.rewritePipelineName) || ''}
                                onChange={(e) => handleChange(isTranslate ? 'translatePipelineName' : 'rewritePipelineName', e.target.value)}
                                placeholder="Назва пайплайну..."
                            />
                        </div>
                    </div>

                    {/* API Settings Section */}
                    <div className={`pipeline-stage-container ${isApiCollapsed ? 'is-collapsed' : ''}`}>
                        <div
                            className="pipeline-stage-header"
                            onClick={() => handleChange('apiCollapsed', !isApiCollapsed)}
                        >
                            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flex: 1 }}>
                                <svg
                                    className={`stage-chevron ${isApiCollapsed ? 'rotated' : ''}`}
                                    xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"
                                >
                                    <path d="m6 9 6 6 6-6" />
                                </svg>
                                <div style={{ display: 'flex', flexDirection: 'column' }}>
                                    <span className="pipeline-stage-title">{t('pipeline.group.api')}</span>
                                </div>
                            </div>
                        </div>

                        <div className={`stage-settings-content ${isApiCollapsed ? 'collapsed' : ''}`}>
                            <div className="settings-group">
                                <div className="settings-control">
                                    <label className="settings-label">OpenRouter Key</label>
                                    <select
                                        className="settings-select"
                                        value={selectedApiKeyID}
                                        onChange={(e) => {
                                            const val = e.target.value;
                                            if (val === "MANAGE_KEYS") {
                                                if (setCurrentPath) {
                                                    setCurrentPath('settings.api.openrouter');
                                                }
                                                return;
                                            }
                                            handleChange(isTranslate ? 'translateOpenRouterKeyID' : 'rewriteOpenRouterKeyID', val);
                                        }}
                                    >
                                        {openRouterKeys.length === 0 ? (
                                            <option value="">{t('api.openrouterSettings.noKeys')}</option>
                                        ) : (
                                            openRouterKeys.map(k => (
                                                <option key={k.id} value={k.id}>{k.name}</option>
                                            ))
                                        )}
                                        <option value="MANAGE_KEYS" style={{ color: 'var(--accent-primary)', fontWeight: 'bold' }}>
                                            ⚙️ {t('tabs.settings')}
                                        </option>
                                    </select>
                                </div>
                            </div>
                        </div>
                    </div>

                    {/* Save Path Section */}
                    <div className={`pipeline-stage-container ${isPathCollapsed ? 'is-collapsed' : ''}`}>
                        <div
                            className="pipeline-stage-header"
                            onClick={() => handleChange('pathCollapsed', !isPathCollapsed)}
                        >
                            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flex: 1 }}>
                                <svg
                                    className={`stage-chevron ${isPathCollapsed ? 'rotated' : ''}`}
                                    xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"
                                >
                                    <path d="m6 9 6 6 6-6" />
                                </svg>
                                <div style={{ display: 'flex', flexDirection: 'column' }}>
                                    <span className="pipeline-stage-title">{t('pipeline.stage.path')}</span>
                                </div>
                            </div>
                        </div>

                        <div className={`stage-settings-content ${isPathCollapsed ? 'collapsed' : ''}`}>
                            <div className="settings-group">
                                <div className="settings-control">
                                    <label className="settings-label">{t('pipeline.group.path')}</label>
                                    <div className="settings-row">
                                        <input
                                            className="settings-input"
                                            style={{ flex: 1, textOverflow: 'ellipsis' }}
                                            value={outputPath}
                                            readOnly
                                            placeholder="Виберіть папку..."
                                        />
                                        <button
                                            className="settings-button"
                                            onClick={handleSelectPath}
                                        >
                                            Обзор
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div className={`pipeline-stage-container ${isCollapsed || !isEnabled ? 'is-collapsed' : ''}`}>
                        <div
                            className="pipeline-stage-header"
                            onClick={toggleCollapse}
                        >
                            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flex: 1 }}>
                                <svg
                                    className={`stage-chevron ${isCollapsed || !isEnabled ? 'rotated' : ''}`}
                                    xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"
                                >
                                    <path d="m6 9 6 6 6-6" />
                                </svg>
                                <div style={{ display: 'flex', flexDirection: 'column' }}>
                                    <span className="pipeline-stage-title">{t(`pipeline.stage.${type}`)}</span>
                                    {isTranslate && (
                                        <span className="stage-status-text">
                                            {isEnabled ? t('pipeline.stage.enabled') : t('pipeline.stage.disabled')}
                                        </span>
                                    )}
                                </div>
                            </div>

                            {isTranslate && (
                                <label className="stage-switch" onClick={(e) => e.stopPropagation()}>
                                    <input
                                        type="checkbox"
                                        checked={isEnabled}
                                        onChange={handleToggleEnable}
                                    />
                                    <span className="stage-slider"></span>
                                </label>
                            )}
                        </div>

                        <div className={`stage-settings-content ${isCollapsed || !isEnabled ? 'collapsed' : ''}`}>
                            <div className="settings-group">
                                <div className="settings-group-title">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12 2a10 10 0 1 0 10 10H12V2z" /><path d="M12 12 2.1 12a10.05 10.05 0 0 1 9.9-10v10z" /><path d="m9 16.5 3-3" /></svg>
                                    {t('pipeline.group.ai')}
                                </div>

                                <div className="settings-control">
                                    <label className="settings-label">{t('pipeline.model')}</label>
                                    <select
                                        className="settings-select"
                                        value={modelValue}
                                        onChange={(e) => {
                                            const val = e.target.value;
                                            if (val === "ADD_NEW_MODEL") {
                                                if (setCurrentPath) {
                                                    setCurrentPath('settings.api.openrouter');
                                                }
                                                return;
                                            }
                                            handleChange(isTranslate ? 'translateModel' : 'rewriteModel', val);
                                        }}
                                    >
                                        {models.map(m => <option key={m} value={m}>{m}</option>)}
                                        <option value="ADD_NEW_MODEL" style={{ color: 'var(--accent-primary)', fontWeight: 'bold' }}>
                                            + {t('pipeline.add_model')}
                                        </option>
                                    </select>
                                </div>

                                <div className="settings-control">
                                    <label className="settings-label">{t('pipeline.temperature')}</label>
                                    <div className="settings-slider-container">
                                        <input
                                            type="range"
                                            className="settings-slider"
                                            min="0"
                                            max="2"
                                            step="0.1"
                                            value={tempValue}
                                            style={{ '--range-progress': `${(tempValue / 2) * 100}%` } as React.CSSProperties}
                                            onChange={(e) => handleChange(isTranslate ? 'translateTemperature' : 'rewriteTemperature', parseFloat(e.target.value))}
                                        />
                                        {renderValueOrInput(isTranslate ? 'translateTemperature' : 'rewriteTemperature', tempValue, true)}
                                    </div>
                                </div>

                                <div className="settings-control">
                                    <label className="settings-label">{t('pipeline.max_tokens')}</label>
                                    <div className="settings-slider-container">
                                        <input
                                            type="range"
                                            className="settings-slider"
                                            min="0"
                                            max="128000"
                                            step="500"
                                            value={tokensValue}
                                            style={{ '--range-progress': `${(tokensValue / 128000) * 100}%` } as React.CSSProperties}
                                            onChange={(e) => handleChange(isTranslate ? 'translateMaxTokens' : 'rewriteMaxTokens', parseInt(e.target.value))}
                                        />
                                        {renderValueOrInput(isTranslate ? 'translateMaxTokens' : 'rewriteMaxTokens', tokensValue, false)}
                                    </div>
                                </div>
                            </div>

                            <div className="settings-group">
                                <div className="settings-group-title">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" /></svg>
                                    {t('pipeline.group.prompt')}
                                </div>
                                <div className="settings-control">
                                    <label className="settings-label">{t('pipeline.system_prompt')}</label>
                                    <textarea
                                        className="settings-textarea"
                                        value={promptValue}
                                        onChange={(e) => handleChange(isTranslate ? 'translatePrompt' : 'rewritePrompt', e.target.value)}
                                        placeholder={t(`pipeline.${type}.prompt_placeholder`)}
                                    />
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <div className="pipeline-sidebar-footer">
                    <button
                        className="add-to-queue-btn"
                        onClick={() => {
                            if (content.trim()) {
                                setIsModalOpen(true);
                            }
                        }}
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                            <line x1="12" y1="5" x2="12" y2="19"></line>
                            <line x1="5" y1="12" x2="19" y2="12"></line>
                        </svg>
                        {t('pipeline.add_to_queue')}
                    </button>
                </div>
            </div>

            <TaskNameModal
                isOpen={isModalOpen}
                onClose={() => setIsModalOpen(false)}
                onConfirm={(name) => addTask(type, content, settings, name)}
            />

            <button
                className={`sidebar-floating-toggle ${isOpen ? 'is-open' : ''}`}
                onClick={onToggle}
                title={isOpen ? t('pipeline.hide_settings') : t('pipeline.show_settings')}
            >
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" /></svg>
            </button>
        </aside>
    );
};
