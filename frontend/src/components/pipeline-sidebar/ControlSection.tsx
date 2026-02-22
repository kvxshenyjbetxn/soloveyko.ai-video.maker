import React from 'react';
import { useI18n } from '../../contexts/I18nContext';

interface ControlSectionProps {
    settings: any;
    handleChange: (field: string, value: any) => void;
}

const ControlIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <line x1="4" y1="21" x2="4" y2="14" />
        <line x1="4" y1="10" x2="4" y2="3" />
        <line x1="12" y1="21" x2="12" y2="12" />
        <line x1="12" y1="8" x2="12" y2="3" />
        <line x1="20" y1="21" x2="20" y2="16" />
        <line x1="20" y1="12" x2="20" y2="3" />
        <line x1="2" y1="14" x2="6" y2="14" />
        <line x1="10" y1="8" x2="14" y2="8" />
        <line x1="18" y1="16" x2="22" y2="16" />
    </svg>
);

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
                    <div style={{
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        width: '28px',
                        height: '28px',
                        borderRadius: '8px',
                        background: 'rgba(var(--accent-rgb), 0.1)',
                        color: 'var(--accent-color)',
                        transition: 'all 0.3s'
                    }}>
                        <ControlIcon />
                    </div>
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

                    <div className="settings-control">
                        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                            <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.image_control') || 'Контроль зображень'}</label>
                            <label className="stage-switch small">
                                <input
                                    type="checkbox"
                                    checked={settings.imageControlEnabled}
                                    onChange={(e) => handleChange('imageControlEnabled', e.target.checked)}
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
