import React from 'react';
import { useI18n } from '../../contexts/I18nContext';

interface ControlSectionProps {
    settings: any;
    handleChange: (field: string, value: any) => void;
}

export const ControlSection: React.FC<ControlSectionProps> = ({ settings, handleChange }) => {
    const { t } = useI18n();

    return (
        <div className={`pipeline-stage-container ${settings.controlCollapsed ? 'is-collapsed' : ''}`}>
            <div
                className="pipeline-stage-header"
                onClick={() => handleChange('controlCollapsed', !settings.controlCollapsed)}
            >
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flex: 1 }}>
                    <svg
                        className={`stage-chevron ${settings.controlCollapsed ? 'rotated' : ''}`}
                        xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"
                    >
                        <path d="m6 9 6 6 6-6" />
                    </svg>
                    <div style={{ display: 'flex', flexDirection: 'column' }}>
                        <span className="pipeline-stage-title">{t('pipeline.control') || 'Контроль'}</span>
                    </div>
                </div>
            </div>
            <div className={`stage-settings-content ${settings.controlCollapsed ? 'collapsed' : ''}`}>
                <div className="settings-group">
                    <div className="settings-control">
                        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                            <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.translate_control') || 'Контроль перекладу'}</label>
                            <label className="stage-switch small">
                                <input
                                    type="checkbox"
                                    checked={settings.translateControlEnabled}
                                    onChange={(e) => handleChange('translateControlEnabled', e.target.checked)}
                                />
                                <span className="stage-slider"></span>
                            </label>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
};
