import React from 'react';
import { useI18n } from '../../contexts/I18nContext';

interface PathSectionProps {
    type: string;
    settings: any;
    handleChange: (field: string, value: any) => void;
    handleSelectPath: () => void;
}

export const PathSection: React.FC<PathSectionProps> = ({ type, settings, handleChange, handleSelectPath }) => {
    const { t } = useI18n();
    const isTranslate = type === 'translate';
    const isRewrite = type === 'rewrite';
    const isVoiceover = type === 'voiceover';

    const outputPath = isTranslate ? settings?.translateOutputPath : (isRewrite ? settings?.rewriteOutputPath : (isVoiceover ? settings?.voiceoverOutputPath : settings?.imageOutputPath)) || '';

    return (
        <div className={`pipeline-stage-container ${settings.pathCollapsed ? 'is-collapsed' : ''}`}>
            <div
                className="pipeline-stage-header"
                onClick={() => handleChange('pathCollapsed', !settings.pathCollapsed)}
                style={{ cursor: 'pointer' }}
            >
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flex: 1 }}>
                    <svg
                        className={`stage-chevron ${settings.pathCollapsed ? 'rotated' : ''}`}
                        xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"
                    >
                        <path d="m6 9 6 6 6-6" />
                    </svg>
                    <div style={{ display: 'flex', flexDirection: 'column' }}>
                        <span className="pipeline-stage-title">{t('pipeline.stage.path')}</span>
                    </div>
                </div>
            </div>

            <div className={`stage-settings-content ${settings.pathCollapsed ? 'collapsed' : ''}`}>
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
    );
};
