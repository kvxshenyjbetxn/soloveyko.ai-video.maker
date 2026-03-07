import React, { useState, useEffect } from 'react';
import { useI18n } from '../../contexts/I18nContext';
import { EventsOn, EventsOff } from '../../../wailsjs/runtime/runtime';
// @ts-ignore
import {
    CheckSubtitleModel,
    DownloadSubtitleModel,
    IsAmdWhisperInstalled,
    InstallAmdWhisper,
    GetAmdWhisperModels
} from '../../../wailsjs/go/main/App';
import { ConfirmModal } from '../ConfirmModal';
import SearchableSelect from '../SearchableSelect';

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
    const [amdInstalled, setAmdInstalled] = useState(false);
    const [amdModels, setAmdModels] = useState<string[]>([]);
    const [installingAmd, setInstallingAmd] = useState(false);
    const [showAmdConfirm, setShowAmdConfirm] = useState(false);

    const models = ['tiny', 'base', 'small', 'medium', 'large-v1', 'large-v2', 'large-v3'];
    const services = [
        { id: 'standard', name: t('pipeline.subtitle.services.standard') },
        { id: 'amd', name: t('pipeline.subtitle.services.amd') },
        { id: 'assemblyai', name: 'AssemblyAI' },
        { id: 'whisperx', name: 'WhisperX' }
    ];

    useEffect(() => {
        if (settings.subtitleModel && (settings.subtitleService === 'standard' || settings.subtitleService === 'amd')) {
            checkModel(settings.subtitleModel);
        }

        if (settings.subtitleService === 'amd') {
            checkAmdStatus();
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

    const checkAmdStatus = async () => {
        try {
            const installed = await IsAmdWhisperInstalled();
            setAmdInstalled(installed);
            if (installed) {
                const modelsList = await GetAmdWhisperModels();
                setAmdModels(modelsList);
                if (modelsList.length > 0 && !settings.subtitleModel) {
                    handleChange('subtitleModel', modelsList[0]);
                }
            }
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

    const handleInstallAmd = async () => {
        setShowAmdConfirm(true);
    };

    const confirmAmdInstall = async () => {
        setShowAmdConfirm(false);
        setInstallingAmd(true);
        try {
            await InstallAmdWhisper();
            await checkAmdStatus();
        } catch (e) {
            console.error(e);
        } finally {
            setInstallingAmd(false);
        }
    };

    const handleServiceChange = (service: string) => {
        handleChange('subtitleService', service);
        if (service === 'amd') {
            checkAmdStatus();
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

                    {(settings.subtitleService === 'standard' || settings.subtitleService === 'amd' || settings.subtitleService === 'whisperx') && (
                        <>
                            <div className="settings-control">
                                <label className="settings-label">{t('pipeline.model')} ({settings.subtitleService === 'whisperx' ? 'WhisperX' : 'Whisper'})</label>
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

                            {settings.subtitleService === 'whisperx' && (
                                <div className="settings-control" style={{ marginBottom: '16px' }}>
                                    <label className="settings-label" style={{ fontSize: '11px' }}>Мова (авто, якщо порожньо)</label>
                                    <input
                                        type="text"
                                        className="settings-input"
                                        style={{ height: '32px' }}
                                        placeholder="en, uk, ru, ..."
                                        value={settings.subtitleWhisperxLanguage || ''}
                                        onChange={(e) => handleChange('subtitleWhisperxLanguage', e.target.value.toLowerCase().slice(0, 2))}
                                    />
                                </div>
                            )}
                        </>
                    )}

                    {(settings.subtitleService === 'standard' || settings.subtitleService === 'amd') && (
                        <>
                            {!modelExists && !downloading && (
                                <div className="settings-control">
                                    <div style={{ padding: '12px', backgroundColor: 'rgba(255, 170, 0, 0.05)', border: '1px solid rgba(255, 170, 0, 0.2)', borderRadius: '10px', fontSize: '11px', color: '#ffaa00', display: 'flex', flexDirection: 'column', gap: '10px' }}>
                                        <div style={{ lineHeight: '1.4' }}>Модель <b>{settings.subtitleModel}</b> не завантажена. Її необхідно завантажити для роботи транскрибації.</div>
                                        <button
                                            onClick={handleDownload}
                                            className="premium-button"
                                            style={{ background: 'linear-gradient(135deg, #ffaa00 0%, #ff8800 100%)', boxShadow: '0 4px 15px rgba(255, 136, 0, 0.2)', height: '32px' }}
                                        >
                                            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
                                                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                                                <polyline points="7 10 12 15 17 10" />
                                                <line x1="12" x2="12" y1="15" y2="3" />
                                            </svg>
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

                    {settings.subtitleService === 'amd' && (
                        <>
                            {!amdInstalled ? (
                                <div className="settings-control" style={{ marginTop: '8px' }}>
                                    <div className="premium-button-glow">
                                        <button
                                            className="premium-button"
                                            onClick={handleInstallAmd}
                                            disabled={installingAmd}
                                        >
                                            {installingAmd ? (
                                                <div className="spinner-tiny animate-spin" />
                                            ) : (
                                                <>
                                                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
                                                        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                                                        <polyline points="7 10 12 15 17 10" />
                                                        <line x1="12" x2="12" y1="15" y2="3" />
                                                    </svg>
                                                    {t('pipeline.subtitle.amd.install')}
                                                </>
                                            )}
                                        </button>
                                    </div>
                                </div>
                            ) : (
                                <>
                                    <div className="settings-control">
                                        <label className="settings-label">{t('pipeline.subtitle.amd.language')}</label>
                                        <input
                                            type="text"
                                            className="settings-input"
                                            placeholder="uk"
                                            value={settings.subtitleAmdLanguage || ''}
                                            onChange={(e) => handleChange('subtitleAmdLanguage', e.target.value.toLowerCase().slice(0, 2))}
                                        />
                                    </div>
                                </>
                            )}
                        </>
                    )}

                    {/* --- ГРУПА: КАРАОКЕ ТА WHISPERX/ASSEMBLY-AI --- */}
                    {(settings.subtitleService === 'whisperx' || settings.subtitleService === 'assemblyai') && (
                        <div style={{
                            marginBottom: '16px',
                            padding: '16px',
                            background: 'rgba(var(--accent-rgb), 0.04)',
                            borderRadius: '12px',
                            border: '1px solid rgba(var(--accent-rgb), 0.2)',
                            display: 'flex',
                            flexDirection: 'column',
                            gap: '12px'
                        }}>
                            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                                <label className="settings-label" style={{ marginBottom: 0, display: 'flex', alignItems: 'center', gap: '8px' }}>
                                    <svg width="16" height="16" viewBox="0 0 24 24" fill="var(--accent-primary)"><path d="M12 2L15.09 8.26L22 9.27L17 14.14L18.18 21.02L12 17.77L5.82 21.02L7 14.14L2 9.27L8.91 8.26L12 2Z" /></svg>
                                    <span style={{ fontSize: '14px', fontWeight: 700, color: 'var(--accent-primary)' }}>Ефект Караоке</span>
                                </label>
                                <label className="stage-switch small">
                                    <input
                                        type="checkbox"
                                        checked={settings.subtitleKaraokeEffect || false}
                                        onChange={(e) => handleChange('subtitleKaraokeEffect', e.target.checked)}
                                    />
                                    <span className="stage-slider"></span>
                                </label>
                            </div>

                            {settings.subtitleKaraokeEffect && (
                                <div style={{ display: 'flex', flexDirection: 'column', gap: '12px', paddingTop: '12px', borderTop: '1px solid rgba(var(--accent-rgb), 0.1)' }}>

                                    <div className="settings-control" style={{ marginBottom: 0 }}>
                                        <label className="settings-label" style={{ fontSize: '11px' }}>Колір активного слова</label>
                                        <div style={{ display: 'flex', gap: '6px', alignItems: 'center' }}>
                                            <div style={{ width: '28px', height: '28px', borderRadius: '6px', backgroundColor: settings.subtitleKaraokeColor || '#ffd700', border: '1px solid var(--border-color)', position: 'relative', overflow: 'hidden', cursor: 'pointer' }}>
                                                <input type="color" value={settings.subtitleKaraokeColor || '#ffd700'} onChange={(e) => handleChange('subtitleKaraokeColor', e.target.value)} style={{ position: 'absolute', top: '-5px', left: '-5px', width: '40px', height: '40px', opacity: 0 }} />
                                            </div>
                                            <input type="text" className="settings-input" style={{ fontFamily: 'monospace', fontSize: '10px', textTransform: 'uppercase', height: '28px' }} value={settings.subtitleKaraokeColor || '#ffd700'} onChange={(e) => handleChange('subtitleKaraokeColor', e.target.value)} />
                                        </div>
                                    </div>

                                    <div className="settings-control" style={{ marginBottom: 0 }}>
                                        <label className="settings-label" style={{ fontSize: '11px' }}>Scale-up (Збільшення)</label>
                                        <div className="settings-slider-container">
                                            <input
                                                type="range"
                                                min="1"
                                                max="2"
                                                step="0.05"
                                                className="settings-slider"
                                                value={settings.subtitleKaraokeScale ?? 1.1}
                                                style={{ '--range-progress': `${((settings.subtitleKaraokeScale ?? 1.1) - 1) / (2 - 1) * 100}%` } as React.CSSProperties}
                                                onChange={(e) => handleChange('subtitleKaraokeScale', parseFloat(e.target.value))}
                                            />
                                            <span className="settings-slider-value">x{settings.subtitleKaraokeScale ?? 1.1}</span>
                                        </div>
                                    </div>

                                    <div className="settings-control" style={{ marginBottom: 0 }}>
                                        <label className="settings-label" style={{ fontSize: '11px' }}>Швидкість анімації (ms)</label>
                                        <div className="settings-slider-container">
                                            <input
                                                type="range"
                                                min="0"
                                                max="500"
                                                step="10"
                                                className="settings-slider"
                                                value={settings.subtitleKaraokeSpeed ?? 100}
                                                style={{ '--range-progress': `${((settings.subtitleKaraokeSpeed ?? 100) - 0) / (500 - 0) * 100}%` } as React.CSSProperties}
                                                onChange={(e) => handleChange('subtitleKaraokeSpeed', parseInt(e.target.value))}
                                            />
                                            <span className="settings-slider-value">{settings.subtitleKaraokeSpeed ?? 100}ms</span>
                                        </div>
                                    </div>

                                    <div className="settings-control" style={{ marginBottom: 0 }}>
                                        <label className="settings-label" style={{ fontSize: '11px' }}>Тип підсвічування</label>
                                        <select className="settings-select" style={{ height: '28px', fontSize: '11px' }} value={settings.subtitleKaraokeMode || 'highlight'} onChange={(e) => handleChange('subtitleKaraokeMode', e.target.value)}>
                                            <option value="fill">Заливка (поступовий перехід)</option>
                                            <option value="highlight">Акцент (тільки поточне слово)</option>
                                        </select>
                                    </div>

                                    <div className="settings-control" style={{ marginBottom: 0 }}>
                                        <label className="settings-label" style={{ fontSize: '11px' }}>Слів у рядку (Динаміка)</label>
                                        <div className="settings-slider-container">
                                            <input
                                                type="range"
                                                min="1"
                                                max="20"
                                                step="1"
                                                className="settings-slider"
                                                value={settings.subtitleMaxWords || 10}
                                                style={{ '--range-progress': `${((settings.subtitleMaxWords || 10) - 1) / (20 - 1) * 100}%` } as React.CSSProperties}
                                                onChange={(e) => handleChange('subtitleMaxWords', parseInt(e.target.value))}
                                            />
                                            <span className="settings-slider-value">{settings.subtitleMaxWords || 10}</span>
                                        </div>
                                    </div>

                                    <div style={{ fontSize: '10px', color: 'rgba(var(--accent-rgb), 0.7)', lineHeight: '1.4', fontStyle: 'italic' }}>
                                        {t('pipeline.subtitle.karaoke_desc') || "Створює анімовані субтитри з точним підсвічуванням кожного слова під час мовлення."}
                                    </div>
                                </div>
                            )}
                        </div>
                    )}

                    {(settings.subtitleService === 'standard' || settings.subtitleService === 'amd') && (
                        <>
                            {settings.subtitleService === 'standard' && (
                                <div className="settings-control">
                                    <label className="settings-label">{t('pipeline.subtitle.max_len')}</label>
                                    <div className="settings-slider-container">
                                        <input
                                            type="range"
                                            min="10"
                                            max="150"
                                            step="1"
                                            className="settings-slider"
                                            value={settings.subtitleMaxLen || 40}
                                            style={{ '--range-progress': `${((settings.subtitleMaxLen || 40) - 10) / (150 - 10) * 100}%` } as React.CSSProperties}
                                            onChange={(e) => handleChange('subtitleMaxLen', parseInt(e.target.value))}
                                        />
                                        <span className="settings-slider-value">{settings.subtitleMaxLen || 40}</span>
                                    </div>
                                    <div style={{ fontSize: '10px', color: '#888', marginTop: '4px' }}>
                                        {t('pipeline.subtitle.max_len_desc')}
                                    </div>
                                </div>
                            )}

                            {settings.subtitleService === 'amd' && (
                                <div className="settings-control">
                                    <label className="settings-label">{t('pipeline.subtitle.max_words')}</label>
                                    <div className="settings-slider-container">
                                        <input
                                            type="range"
                                            min="1"
                                            max="30"
                                            step="1"
                                            className="settings-slider"
                                            value={settings.subtitleMaxWords || 10}
                                            style={{ '--range-progress': `${((settings.subtitleMaxWords || 10) - 1) / (30 - 1) * 100}%` } as React.CSSProperties}
                                            onChange={(e) => handleChange('subtitleMaxWords', parseInt(e.target.value))}
                                        />
                                        <span className="settings-slider-value">{settings.subtitleMaxWords || 10}</span>
                                    </div>
                                    <div style={{ fontSize: '10px', color: '#888', marginTop: '4px' }}>
                                        {t('pipeline.subtitle.max_words_desc')}
                                    </div>
                                </div>
                            )}
                        </>
                    )}

                    {/* --- ГРУПА: СТИЛІЗАЦІЯ ТЕКСТУ --- */}
                    <div style={{ marginTop: '16px', paddingTop: '16px', borderTop: '1px solid var(--border-color)', display: 'flex', flexDirection: 'column', gap: '16px' }}>
                        <div style={{ fontSize: '12px', fontWeight: '600', color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
                            Стилізація та Оформлення
                        </div>

                        {/* Шрифти */}
                        <div className="settings-control">
                            <label className="settings-label">{t('pipeline.subtitle.font')}</label>
                            <SearchableSelect
                                options={[
                                    'Arial', 'Montserrat', 'Inter', 'Roboto', 'Open Sans', 'Verdana', 'Tahoma',
                                    'Impact', 'Georgia', 'Times New Roman', 'Arial Black',
                                    'Helvetica', 'Segoe UI', 'Ubuntu', 'Noto Sans', 'Oswald', 'Raleway',
                                    'Poppins', 'Lato', 'Quicksand', 'Nunito', 'Bebas Neue'
                                ].sort().map(f => ({ value: f, label: f }))}
                                value={settings.subtitleFont || 'Arial'}
                                onChange={(val) => handleChange('subtitleFont', val)}
                                placeholder={t('pipeline.subtitle.font_placeholder')}
                                searchPlaceholder={t('pipeline.subtitle.font_search')}
                            />
                        </div>

                        {/* Основний Колір та Розмір */}
                        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px' }}>
                            <div className="settings-control" style={{ marginBottom: 0 }}>
                                <label className="settings-label">Колір тексту</label>
                                <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                                    <div style={{ width: '28px', height: '28px', borderRadius: '6px', backgroundColor: settings.subtitleColor || '#ffffff', border: '2px solid var(--border-color)', position: 'relative', overflow: 'hidden', cursor: 'pointer' }}>
                                        <input type="color" value={settings.subtitleColor || '#ffffff'} onChange={(e) => handleChange('subtitleColor', e.target.value)} style={{ position: 'absolute', top: '-5px', left: '-5px', width: '50px', height: '50px', cursor: 'pointer', opacity: 0 }} />
                                    </div>
                                    <input type="text" className="settings-input" style={{ fontFamily: 'monospace', fontSize: '10px', textTransform: 'uppercase', height: '28px' }} value={settings.subtitleColor || '#ffffff'} onChange={(e) => handleChange('subtitleColor', e.target.value)} />
                                </div>
                            </div>
                            <div className="settings-control" style={{ marginBottom: 0 }}>
                                <label className="settings-label">Розмір</label>
                                <div className="settings-slider-container">
                                    <input
                                        type="range"
                                        min="8"
                                        max="120"
                                        step="1"
                                        className="settings-slider"
                                        value={settings.subtitleSize || 24}
                                        style={{ '--range-progress': `${((settings.subtitleSize || 24) - 8) / (120 - 8) * 100}%` } as React.CSSProperties}
                                        onChange={(e) => handleChange('subtitleSize', parseInt(e.target.value))}
                                    />
                                    <span className="settings-slider-value">{settings.subtitleSize || 24}</span>
                                </div>
                            </div>
                        </div>

                        {/* Обводка та Тінь */}
                        <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                            <div className="settings-control" style={{ marginBottom: 0 }}>
                                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '4px' }}>
                                    <label className="settings-label">Обводка</label>
                                    <div style={{ width: '24px', height: '24px', borderRadius: '4px', backgroundColor: settings.subtitleOutlineColor || '#000000', border: '1px solid var(--border-color)', position: 'relative', overflow: 'hidden', cursor: 'pointer' }}>
                                        <input type="color" value={settings.subtitleOutlineColor || '#000000'} onChange={(e) => handleChange('subtitleOutlineColor', e.target.value)} style={{ position: 'absolute', top: '-5px', left: '-5px', width: '40px', height: '40px', opacity: 0 }} />
                                    </div>
                                </div>
                                <div className="settings-slider-container">
                                    <input
                                        type="range"
                                        min="0"
                                        max="10"
                                        step="0.5"
                                        className="settings-slider"
                                        value={settings.subtitleOutlineWidth ?? 2}
                                        style={{ '--range-progress': `${((settings.subtitleOutlineWidth ?? 2) - 0) / (10 - 0) * 100}%` } as React.CSSProperties}
                                        onChange={(e) => handleChange('subtitleOutlineWidth', parseFloat(e.target.value))}
                                    />
                                    <span className="settings-slider-value">{settings.subtitleOutlineWidth ?? 2}</span>
                                </div>
                            </div>

                            <div className="settings-control" style={{ marginBottom: 0 }}>
                                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '4px' }}>
                                    <label className="settings-label">Тінь</label>
                                    <div style={{ width: '24px', height: '24px', borderRadius: '4px', backgroundColor: settings.subtitleShadowColor || '#000000', border: '1px solid var(--border-color)', position: 'relative', overflow: 'hidden', cursor: 'pointer' }}>
                                        <input type="color" value={settings.subtitleShadowColor || '#000000'} onChange={(e) => handleChange('subtitleShadowColor', e.target.value)} style={{ position: 'absolute', top: '-5px', left: '-5px', width: '40px', height: '40px', opacity: 0 }} />
                                    </div>
                                </div>
                                <div className="settings-slider-container">
                                    <input
                                        type="range"
                                        min="0"
                                        max="10"
                                        step="0.5"
                                        className="settings-slider"
                                        value={settings.subtitleShadowWidth ?? 1}
                                        style={{ '--range-progress': `${((settings.subtitleShadowWidth ?? 1) - 0) / (10 - 0) * 100}%` } as React.CSSProperties}
                                        onChange={(e) => handleChange('subtitleShadowWidth', parseFloat(e.target.value))}
                                    />
                                    <span className="settings-slider-value">{settings.subtitleShadowWidth ?? 1}</span>
                                </div>
                            </div>
                        </div>

                        {/* Glow (Blur) та Регістр */}
                        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px' }}>
                            <div className="settings-control" style={{ marginBottom: 0 }}>
                                <label className="settings-label">Blur (Сяйво тіні)</label>
                                <div className="settings-slider-container">
                                    <input
                                        type="range"
                                        min="0"
                                        max="20"
                                        step="1"
                                        className="settings-slider"
                                        value={settings.subtitleBlur ?? 0}
                                        style={{ '--range-progress': `${((settings.subtitleBlur ?? 0) - 0) / (20 - 0) * 100}%` } as React.CSSProperties}
                                        onChange={(e) => handleChange('subtitleBlur', parseFloat(e.target.value))}
                                    />
                                    <span className="settings-slider-value">{settings.subtitleBlur ?? 0}</span>
                                </div>
                            </div>
                            <div className="settings-control" style={{ marginBottom: 0 }}>
                                <label className="settings-label">Всі великі (UPPERCASE)</label>
                                <label className="stage-switch small" style={{ marginTop: '4px' }}>
                                    <input type="checkbox" checked={settings.subtitleUppercase || false} onChange={(e) => handleChange('subtitleUppercase', e.target.checked)} />
                                    <span className="stage-slider"></span>
                                </label>
                            </div>
                        </div>


                    </div>

                    {/* --- ГРУПА: РОЗТАШУВАННЯ --- */}
                    <div style={{ marginTop: '16px', paddingTop: '16px', borderTop: '1px solid var(--border-color)', display: 'flex', flexDirection: 'column', gap: '16px' }}>
                        <div style={{ fontSize: '12px', fontWeight: '600', color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
                            Позиція та Відступи
                        </div>

                        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px' }}>
                            <div className="settings-control" style={{ marginBottom: 0 }}>
                                <label className="settings-label">Вертикальна позиція</label>
                                <select className="settings-select" style={{ height: '28px', fontSize: '11px' }} value={settings.subtitlePosition || 'bottom'} onChange={(e) => handleChange('subtitlePosition', e.target.value)}>
                                    <option value="bottom">Знизу</option>
                                    <option value="middle">Посередині</option>
                                    <option value="top">Зверху</option>
                                </select>
                            </div>
                            <div className="settings-control" style={{ marginBottom: 0 }}>
                                <label className="settings-label">Відступ (Margin V)</label>
                                <div className="settings-slider-container">
                                    <input
                                        type="range"
                                        min="0"
                                        max="600"
                                        step="1"
                                        className="settings-slider"
                                        value={settings.subtitleMarginV ?? 80}
                                        style={{ '--range-progress': `${((settings.subtitleMarginV ?? 80) - 0) / (600 - 0) * 100}%` } as React.CSSProperties}
                                        onChange={(e) => handleChange('subtitleMarginV', parseInt(e.target.value))}
                                    />
                                    <span className="settings-slider-value">{settings.subtitleMarginV ?? 80}</span>
                                </div>
                            </div>
                        </div>
                    </div>

                    {/* --- ГРУПА: АНІМАЦІЇ ТА ЕФЕКТИ (ЗАГАЛЬНІ) --- */}
                    <div style={{ marginTop: '16px', paddingTop: '16px', borderTop: '1px solid var(--border-color)', display: 'flex', flexDirection: 'column', gap: '16px' }}>
                        <div style={{ fontSize: '12px', fontWeight: '600', color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
                            Анімація (Загальна)
                        </div>

                        <div className="settings-control">
                            <label className="settings-label">Основна анімація появи</label>
                            <select className="settings-select" style={{ height: '28px', fontSize: '11px' }} value={settings.subtitleAnimation || 'none'} onChange={(e) => handleChange('subtitleAnimation', e.target.value)}>
                                <option value="none">Без анімації</option>
                                <option value="slide-up">Slide Up (Плавний підйом)</option>
                            </select>
                        </div>

                        {/* Fade (FadeIn/FadeOut) */}
                        <div className="settings-control">
                            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '8px' }}>
                                <label className="settings-label" style={{ marginBottom: 0 }}>Плавна поява/зникнення (Fade)</label>
                                <label className="stage-switch small">
                                    <input type="checkbox" checked={settings.subtitleFadeEnabled !== false} onChange={(e) => handleChange('subtitleFadeEnabled', e.target.checked)} />
                                    <span className="stage-slider"></span>
                                </label>
                            </div>
                            {settings.subtitleFadeEnabled !== false && (
                                <div style={{ display: 'flex', flexDirection: 'column', gap: '12px', padding: '10px', background: 'rgba(var(--accent-rgb), 0.03)', borderRadius: '8px' }}>
                                    <div className="settings-control" style={{ marginBottom: 0 }}>
                                        <label className="settings-label" style={{ fontSize: '10px' }}>Поява (ms)</label>
                                        <div className="settings-slider-container">
                                            <input
                                                type="range"
                                                min="0"
                                                max="2000"
                                                step="50"
                                                className="settings-slider"
                                                value={settings.subtitleFadeIn || 300}
                                                style={{ '--range-progress': `${((settings.subtitleFadeIn || 300) - 0) / (2000 - 0) * 100}%` } as React.CSSProperties}
                                                onChange={(e) => handleChange('subtitleFadeIn', parseInt(e.target.value))}
                                            />
                                            <span className="settings-slider-value">{settings.subtitleFadeIn || 300}</span>
                                        </div>
                                    </div>
                                    <div className="settings-control" style={{ marginBottom: 0 }}>
                                        <label className="settings-label" style={{ fontSize: '10px' }}>Зникнення (ms)</label>
                                        <div className="settings-slider-container">
                                            <input
                                                type="range"
                                                min="0"
                                                max="2000"
                                                step="50"
                                                className="settings-slider"
                                                value={settings.subtitleFadeOut || 300}
                                                style={{ '--range-progress': `${((settings.subtitleFadeOut || 300) - 0) / (2000 - 0) * 100}%` } as React.CSSProperties}
                                                onChange={(e) => handleChange('subtitleFadeOut', parseInt(e.target.value))}
                                            />
                                            <span className="settings-slider-value">{settings.subtitleFadeOut || 300}</span>
                                        </div>
                                    </div>
                                </div>
                            )}
                        </div>
                    </div>
                </div>
            </div>

            <ConfirmModal
                isOpen={showAmdConfirm}
                onClose={() => setShowAmdConfirm(false)}
                onConfirm={confirmAmdInstall}
                title="AMD Whisper"
                message={t('pipeline.subtitle.amd.warning')}
                confirmText={t('common.add')}
                cancelText={t('common.cancel')}
                isDanger={false}
                type="warning"
            />
        </div>
    );
};

