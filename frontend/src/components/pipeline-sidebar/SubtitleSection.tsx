import React, { useState, useEffect } from 'react';
import { useI18n } from '../../contexts/I18nContext';
import { EventsOn, EventsOff } from '../../../wailsjs/runtime/runtime';
// @ts-ignore
import { CheckSubtitleModel, DownloadSubtitleModel } from '../../../wailsjs/go/main/App';

interface SubtitleSectionProps {
    settings: any;
    handleChange: (field: string, value: any) => void;
    setSettings: React.Dispatch<React.SetStateAction<any>>;
}

const SubtitleIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <rect x="3" y="11" width="18" height="10" rx="2" />
        <path d="M7 15h4M15 15h2M7 18h10" />
        <path d="M12 2v3M5 5l1.5 1.5M19 5l-1.5 1.5" />
    </svg>
);

export const SubtitleSection: React.FC<SubtitleSectionProps> = ({
    settings, handleChange, setSettings
}) => {
    const { t } = useI18n();
    const [downloading, setDownloading] = useState(false);
    const [downloadProgress, setDownloadProgress] = useState(0);
    const [downloadStatus, setDownloadStatus] = useState('');
    const [modelExists, setModelExists] = useState(true);

    const models = ['tiny', 'base', 'small', 'medium', 'large-v1', 'large-v2', 'large-v3'];
    const services = [
        { id: 'standard', name: 'Стандарт (Local Whisper)' },
        { id: 'amd', name: 'AMD (в розробці)' },
        { id: 'assemblyai', name: 'AssemblyAI (в розробці)' }
    ];

    useEffect(() => {
        if (settings.subtitleService === 'standard' && settings.subtitleModel) {
            checkModel(settings.subtitleModel);
        }
    }, [settings.subtitleService, settings.subtitleModel]);

    useEffect(() => {
        const unsubscribe = EventsOn('download_progress', (data: any) => {
            setDownloadStatus(data.status);
            setDownloadProgress(data.percent);
            if (data.percent >= 100 && data.status.includes('Транскрибація') === false) {
                setTimeout(() => {
                    setDownloading(false);
                    setModelExists(true);
                }, 1000);
            }
        });
        return () => {
            EventsOff('download_progress');
            if (unsubscribe) unsubscribe();
        };
    }, []);

    const checkModel = async (model: string) => {
        try {
            const exists = await CheckSubtitleModel(model);
            setModelExists(exists);
        } catch (e) {
            console.error(e);
        }
    };

    const handleDownload = async () => {
        if (!settings.subtitleModel) return;
        setDownloading(true);
        setDownloadProgress(0);
        setDownloadStatus(`Початок завантаження моделі ${settings.subtitleModel}...`);
        try {
            await DownloadSubtitleModel(settings.subtitleModel);
        } catch (e) {
            console.error(e);
            setDownloading(false);
        }
    };

    return (
        <div className={`pipeline-stage-container ${settings.subtitleCollapsed || !settings.subtitleEnabled ? 'is-collapsed' : ''}`}>
            <div
                className="pipeline-stage-header"
                onClick={() => handleChange('subtitleCollapsed', !settings.subtitleCollapsed)}
            >
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flex: 1 }}>
                    <svg
                        className={`stage-chevron ${settings.subtitleCollapsed || !settings.subtitleEnabled ? 'rotated' : ''}`}
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
                        background: settings.subtitleEnabled ? 'rgba(var(--accent-rgb), 0.1)' : 'var(--bg-tertiary)',
                        color: settings.subtitleEnabled ? 'var(--accent-color)' : 'var(--text-tertiary)',
                        transition: 'all 0.3s'
                    }}>
                        <SubtitleIcon />
                    </div>
                    <div style={{ display: 'flex', flexDirection: 'column' }}>
                        <span className="pipeline-stage-title">{t('pipeline.stage.subtitle')}</span>
                        <span className="stage-status-text">
                            {settings.subtitleEnabled ? t('pipeline.stage.enabled') : t('pipeline.stage.disabled_simple')}
                        </span>
                    </div>
                </div>
                <label className="stage-switch" onClick={(e) => e.stopPropagation()}>
                    <input
                        type="checkbox"
                        checked={settings.subtitleEnabled}
                        onChange={(e) => {
                            const val = e.target.checked;
                            setSettings((prev: any) => ({
                                ...prev,
                                subtitleEnabled: val,
                                subtitleCollapsed: !val ? true : prev.subtitleCollapsed
                            }));
                        }}
                    />
                    <span className="stage-slider"></span>
                </label>
            </div>

            <div className={`stage-settings-content ${settings.subtitleCollapsed || !settings.subtitleEnabled ? 'collapsed' : ''}`}>
                <div className="settings-group">
                    <div className="settings-control">
                        <label className="settings-label">Сервіс транскрибації</label>
                        <select
                            className="settings-select"
                            value={settings.subtitleService || 'standard'}
                            onChange={(e) => handleChange('subtitleService', e.target.value)}
                        >
                            {services.map(s => (
                                <option key={s.id} value={s.id}>{s.name}</option>
                            ))}
                        </select>
                    </div>

                    {settings.subtitleService === 'standard' && (
                        <>
                            <div className="settings-control">
                                <label className="settings-label">Модель розпізнавання (Whisper)</label>
                                <select
                                    className="settings-select"
                                    value={settings.subtitleModel || 'base'}
                                    onChange={(e) => handleChange('subtitleModel', e.target.value)}
                                >
                                    {models.map(m => (
                                        <option key={m} value={m}>{m} ({m === 'tiny' || m === 'base' ? 'швидко' : m === 'small' || m === 'medium' ? 'баланс' : 'повільно/якісно'})</option>
                                    ))}
                                </select>
                            </div>

                            {!modelExists && !downloading && (
                                <div className="settings-control">
                                    <div style={{ padding: '10px', backgroundColor: 'rgba(255, 170, 0, 0.1)', border: '1px solid rgba(255, 170, 0, 0.3)', borderRadius: '6px', fontSize: '12px', color: '#ffaa00' }}>
                                        <div style={{ marginBottom: '8px' }}>Модель <b>{settings.subtitleModel}</b> не завантажена. Її необхідно завантажити для роботи транскрибації.</div>
                                        <button
                                            onClick={handleDownload}
                                            style={{ background: '#ffaa00', color: '#000', border: 'none', padding: '6px 12px', borderRadius: '4px', cursor: 'pointer', fontWeight: 'bold' }}>
                                            Завантажити зараз
                                        </button>
                                    </div>
                                </div>
                            )}

                            {downloading && (
                                <div className="settings-control">
                                    <div style={{ padding: '10px', backgroundColor: 'rgba(0, 153, 255, 0.1)', border: '1px solid rgba(0, 153, 255, 0.3)', borderRadius: '6px' }}>
                                        <div style={{ fontSize: '11px', color: '#888', marginBottom: '4px' }}>{downloadStatus}</div>
                                        <div style={{ width: '100%', height: '8px', backgroundColor: '#222', borderRadius: '4px', overflow: 'hidden' }}>
                                            <div style={{ width: `${downloadProgress}%`, height: '100%', backgroundColor: '#0099ff', transition: 'width 0.2s', borderRadius: '4px' }}></div>
                                        </div>
                                        <div style={{ fontSize: '11px', color: '#0099ff', marginTop: '4px', textAlign: 'right' }}>{downloadProgress.toFixed(1)}%</div>
                                    </div>
                                </div>
                            )}
                        </>
                    )}
                </div>
            </div>
        </div>
    );
};
