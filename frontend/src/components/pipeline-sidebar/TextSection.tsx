import React from 'react';
import { useI18n } from '../../contexts/I18nContext';

interface TextSectionProps {
    type: 'translate' | 'rewrite';
    settings: any;
    handleChange: (field: string, value: any) => void;
    models: string[];
    renderValueOrInput: (field: string, value: number, isFloat: boolean) => React.ReactNode;
    setCurrentPath?: (path: string) => void;
}

export const TextSection: React.FC<TextSectionProps> = ({
    type, settings, handleChange, models, renderValueOrInput, setCurrentPath
}) => {
    const { t } = useI18n();
    const isTranslate = type === 'translate';
    const isRewrite = type === 'rewrite';

    const isEnabled = isTranslate ? settings.translateEnabled : settings.rewriteEnabled;
    const isCollapsed = isTranslate ? settings.translateCollapsed : settings.rewriteCollapsed;

    const modelValue = isTranslate ? settings.translateModel : (isRewrite ? settings.rewriteModel : '');
    const tempValue = (isTranslate ? settings.translateTemperature : settings.rewriteTemperature) ?? 0;
    const tokensValue = (isTranslate ? settings.translateMaxTokens : settings.rewriteMaxTokens) ?? 0;
    const promptValue = isTranslate ? settings.translatePrompt : settings.rewritePrompt;

    const toggleCollapse = () => {
        const field = isTranslate ? 'translateCollapsed' : 'rewriteCollapsed';
        handleChange(field, !isCollapsed);
    };

    const handleToggleEnable = (e: React.ChangeEvent<HTMLInputElement>) => {
        const val = e.target.checked;
        const field = isTranslate ? 'translateEnabled' : 'rewriteEnabled';
        const collapsedField = isTranslate ? 'translateCollapsed' : 'rewriteCollapsed';
        handleChange(field, val);
        if (!val) {
            handleChange(collapsedField, true);
        }
    };

    return (
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
                        <span className="stage-status-text">
                            {isEnabled ? t('pipeline.stage.enabled') : t('pipeline.stage.disabled')}
                        </span>
                    </div>
                </div>
                <label className="stage-switch" onClick={(e) => e.stopPropagation()}>
                    <input
                        type="checkbox"
                        checked={isEnabled}
                        onChange={handleToggleEnable}
                    />
                    <span className="stage-slider"></span>
                </label>
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
                                    if (setCurrentPath) setCurrentPath('settings.api.openrouter');
                                    return;
                                }
                                let field = isTranslate ? 'translateModel' : 'rewriteModel';
                                handleChange(field, val);
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
                                onChange={(e) => {
                                    let field = isTranslate ? 'translateTemperature' : 'rewriteTemperature';
                                    handleChange(field, parseFloat(e.target.value));
                                }}
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
                                onChange={(e) => {
                                    let field = isTranslate ? 'translateMaxTokens' : 'rewriteMaxTokens';
                                    handleChange(field, parseInt(e.target.value));
                                }}
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
                            onChange={(e) => {
                                let field = isTranslate ? 'translatePrompt' : 'rewritePrompt';
                                handleChange(field, e.target.value);
                            }}
                            placeholder={t(`pipeline.${type}.prompt_placeholder`)}
                        />
                    </div>
                </div>
            </div>
        </div>
    );
};
