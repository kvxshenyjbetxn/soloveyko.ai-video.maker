import React from 'react';
import { useI18n } from '../../contexts/I18nContext';
import './CustomStagesSection.css';

interface CustomStage {
    id: string;
    name: string;
    prompt: string;
    dataSource: string;
    model: string;
    temperature: number;
    maxTokens: number;
    enabled: boolean;
}

interface CustomStagesSectionProps {
    settings: any;
    handleChange: (field: string, value: any) => void;
    models: string[];
    isCollapsed?: boolean;
    onToggleCollapse?: (collapsed: boolean) => void;
}

const LayersIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <polygon points="12 2 2 7 12 12 22 7 12 2" />
        <polyline points="2 17 12 22 22 17" />
        <polyline points="2 12 12 17 22 12" />
    </svg>
);

const TrashIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M3 6h18" /><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" /><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
    </svg>
);

const CustomStageItem: React.FC<{ 
    stage: CustomStage, 
    onUpdate: (id: string, field: keyof CustomStage, value: any) => void,
    onDelete: (id: string) => void,
    models: string[],
    t: any
}> = React.memo(({ stage, onUpdate, onDelete, models, t }) => {
    const [localName, setLocalName] = React.useState(stage.name);
    const [localPrompt, setLocalPrompt] = React.useState(stage.prompt);

    React.useEffect(() => {
        setLocalName(stage.name);
    }, [stage.name]);

    React.useEffect(() => {
        setLocalPrompt(stage.prompt);
    }, [stage.prompt]);

    return (
        <div className="custom-stage-item">
            <div className="custom-stage-header">
                <input
                    type="text"
                    className="custom-stage-name-input"
                    value={localName}
                    onChange={(e) => setLocalName(e.target.value)}
                    onBlur={() => { if (localName !== stage.name) onUpdate(stage.id, 'name', localName); }}
                    placeholder={t('pipeline.custom_stages.stage_name')}
                    onClick={(e) => e.stopPropagation()}
                />
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <label className="stage-switch" onClick={(e) => e.stopPropagation()}>
                        <input
                            type="checkbox"
                            checked={stage.enabled}
                            onChange={(e) => onUpdate(stage.id, 'enabled', e.target.checked)}
                        />
                        <span className="stage-slider"></span>
                    </label>
                    <button
                        className="delete-stage-btn"
                        onClick={(e) => { e.stopPropagation(); onDelete(stage.id); }}
                    >
                        <TrashIcon />
                    </button>
                </div>
            </div>

            <div className="settings-control">
                <label className="settings-label">{t('pipeline.custom_stages.data_source')}</label>
                <select
                    className="settings-select"
                    value={stage.dataSource}
                    onChange={(e) => onUpdate(stage.id, 'dataSource', e.target.value)}
                >
                    <option value="text">{t('pipeline.custom_stages.source_text')}</option>
                    <option value="taskName">{t('pipeline.custom_stages.source_task_name')}</option>
                </select>
            </div>

            <div className="settings-control">
                <label className="settings-label">{t('pipeline.model')}</label>
                <select
                    className="settings-select"
                    value={stage.model || ''}
                    onChange={(e) => onUpdate(stage.id, 'model', e.target.value)}
                >
                    <option value="">{t('pipeline.custom_stages.default_model')}</option>
                    {models.map(m => <option key={m} value={m}>{m}</option>)}
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
                        value={stage.temperature || 0}
                        style={{ '--range-progress': `${((stage.temperature || 0) / 2) * 100}%` } as React.CSSProperties}
                        onChange={(e) => onUpdate(stage.id, 'temperature', parseFloat(e.target.value))}
                    />
                    <span className="settings-slider-value">
                        {(!stage.temperature || stage.temperature === 0)
                            ? t('pipeline.custom_stages.default_temp')
                            : (Number(stage.temperature) || 0).toFixed(1)}
                    </span>
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
                        value={stage.maxTokens || 0}
                        style={{ '--range-progress': `${((stage.maxTokens || 0) / 128000) * 100}%` } as React.CSSProperties}
                        onChange={(e) => onUpdate(stage.id, 'maxTokens', parseInt(e.target.value))}
                    />
                    <span className="settings-slider-value">{stage.maxTokens === 0 ? t('pipeline.max_tokens_unlimited') : stage.maxTokens}</span>
                </div>
            </div>

            <div className="settings-control">
                <label className="settings-label">{t('pipeline.custom_stages.prompt')}</label>
                <textarea
                    className="settings-textarea"
                    style={{ height: '80px' }}
                    value={localPrompt}
                    onChange={(e) => setLocalPrompt(e.target.value)}
                    onBlur={() => { if (localPrompt !== stage.prompt) onUpdate(stage.id, 'prompt', localPrompt); }}
                    placeholder={t('pipeline.custom_stages.prompt_placeholder')}
                />
            </div>
        </div>
    );
});

export const CustomStagesSection: React.FC<CustomStagesSectionProps> = React.memo(({ settings, handleChange, models, isCollapsed: externalIsCollapsed, onToggleCollapse }) => {
    const { t } = useI18n();
    const stages = settings.customStages || [];
    const internalIsCollapsed = settings.customStagesCollapsed;
    
    const isCollapsed = externalIsCollapsed !== undefined ? externalIsCollapsed : internalIsCollapsed;

    const toggleCollapse = () => {
        if (onToggleCollapse) {
            onToggleCollapse(!isCollapsed);
        } else {
            handleChange('customStagesCollapsed', !isCollapsed);
        }
    };

    const handleAddStage = () => {
        const newStage: CustomStage = {
            id: Math.random().toString(36).substr(2, 9),
            name: `Stage ${stages.length + 1}`,
            prompt: "Summarize: {{content}}",
            dataSource: 'text',
            model: '',
            temperature: 0,
            maxTokens: 0,
            enabled: true
        };
        handleChange('customStages', [...stages, newStage]);
        if (isCollapsed) {
            if (onToggleCollapse) onToggleCollapse(false);
            else handleChange('customStagesCollapsed', false);
        }
    };

    const handleUpdateStage = React.useCallback((id: string, field: keyof CustomStage, value: any) => {
        const newStages = stages.map((s: CustomStage) => s.id === id ? { ...s, [field]: value } : s);
        handleChange('customStages', newStages);
    }, [stages, handleChange]);

    const handleDeleteStage = React.useCallback((id: string) => {
        if (window.confirm(t('pipeline.custom_stages.delete_confirm'))) {
            const newStages = stages.filter((s: CustomStage) => s.id !== id);
            handleChange('customStages', newStages);
        }
    }, [stages, handleChange, t]);

    return (
        <div className={`pipeline-stage-container ${isCollapsed ? 'is-collapsed' : ''}`}>
            <div className="pipeline-stage-header" onClick={toggleCollapse}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flex: 1 }}>
                    <svg
                        className={`stage-chevron ${isCollapsed ? 'rotated' : ''}`}
                        xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"
                    >
                        <path d="m6 9 6 6 6-6" />
                    </svg>
                    <div style={{
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        width: '28px',
                        height: '28px',
                        borderRadius: '8px',
                        background: settings.customStagesEnabled ? 'rgba(var(--accent-rgb), 0.1)' : 'var(--bg-tertiary)',
                        color: settings.customStagesEnabled ? 'var(--accent-color)' : 'var(--text-tertiary)',
                        transition: 'all 0.3s'
                    }}>
                        <LayersIcon />
                    </div>
                    <div style={{ display: 'flex', flexDirection: 'column' }}>
                        <span className="pipeline-stage-title">{t('pipeline.custom_stages.title')}</span>
                        <span className="stage-status-text">
                            {settings.customStagesEnabled ? `${stages.length} ${t('common.ready')}` : t('common.disabled')}
                        </span>
                    </div>
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <label className="stage-switch" onClick={(e) => e.stopPropagation()}>
                        <input
                            type="checkbox"
                            checked={settings.customStagesEnabled}
                            onChange={(e) => {
                                const val = e.target.checked;
                                handleChange('customStagesEnabled', val);
                                if (!val) {
                                    const disabledStages = stages.map((s: CustomStage) => ({ ...s, enabled: false }));
                                    handleChange('customStages', disabledStages);
                                }
                            }}
                        />
                        <span className="stage-slider"></span>
                    </label>
                </div>
            </div>

            <div className={`stage-settings-content ${isCollapsed ? 'collapsed' : ''}`}>
                <div className="custom-stages-list">
                    {stages.map((stage: CustomStage) => (
                        <CustomStageItem 
                            key={stage.id} 
                            stage={stage} 
                            onUpdate={handleUpdateStage} 
                            onDelete={handleDeleteStage}
                            models={models}
                            t={t}
                        />
                    ))}
                </div>
                <button className="add-stage-btn" onClick={handleAddStage}>
                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>
                    {t('pipeline.custom_stages.add_stage')}
                </button>
            </div>
        </div>
    );
});
