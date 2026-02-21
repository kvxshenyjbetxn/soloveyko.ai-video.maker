import React from 'react';
import { useI18n } from '../../contexts/I18nContext';

interface MontageSectionProps {
    settings: any;
    handleChange: (field: string, value: any) => void;
    setSettings: React.Dispatch<React.SetStateAction<any>>;
}

const MontageIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
    </svg>
);

export const MontageSection: React.FC<MontageSectionProps> = ({
    settings, handleChange, setSettings
}) => {
    const { t } = useI18n();

    return (
        <div className={`pipeline-stage-container ${settings.montageCollapsed || !settings.montageEnabled ? 'is-collapsed' : ''}`}>
            <div
                className="pipeline-stage-header"
                onClick={() => handleChange('montageCollapsed', !settings.montageCollapsed)}
            >
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flex: 1 }}>
                    <svg
                        className={`stage-chevron ${settings.montageCollapsed || !settings.montageEnabled ? 'rotated' : ''}`}
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
                        background: settings.montageEnabled ? 'rgba(var(--accent-rgb), 0.1)' : 'var(--bg-tertiary)',
                        color: settings.montageEnabled ? 'var(--accent-color)' : 'var(--text-tertiary)',
                        transition: 'all 0.3s'
                    }}>
                        <MontageIcon />
                    </div>
                    <div style={{ display: 'flex', flexDirection: 'column' }}>
                        <span className="pipeline-stage-title">{t('pipeline.montage.title')}</span>
                        <span className="stage-status-text">
                            {settings.montageEnabled ? t('pipeline.stage.enabled') : t('pipeline.stage.disabled_simple')}
                        </span>
                    </div>
                </div>
                <label className="stage-switch" onClick={(e) => e.stopPropagation()}>
                    <input
                        type="checkbox"
                        checked={settings.montageEnabled}
                        onChange={(e) => {
                            const val = e.target.checked;
                            setSettings((prev: any) => ({
                                ...prev,
                                montageEnabled: val,
                                montageCollapsed: !val ? true : prev.montageCollapsed
                            }));
                        }}
                    />
                    <span className="stage-slider"></span>
                </label>
            </div>

            <div className={`stage-settings-content ${settings.montageCollapsed || !settings.montageEnabled ? 'collapsed' : ''}`}>
                <div className="settings-group">
                    <div className="empty-section-placeholder" style={{
                        display: 'flex',
                        flexDirection: 'column',
                        alignItems: 'center',
                        justifyContent: 'center',
                        padding: '30px 20px',
                        textAlign: 'center',
                        color: 'var(--text-tertiary)',
                        gap: '12px'
                    }}>
                        <div style={{ opacity: 0.3 }}>
                            <MontageIcon />
                        </div>
                        <p style={{ fontSize: '12px', lineHeight: '1.5' }}>
                            {t('pipeline.montage.empty_description')}
                        </p>
                    </div>
                </div>
            </div>
        </div>
    );
};
