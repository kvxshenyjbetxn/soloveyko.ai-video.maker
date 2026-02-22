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

const TRANSITION_EFFECTS = [
    "fade", "wipeleft", "wiperight", "wipeup", "wipedown",
    "slideleft", "slideright", "slideup", "slidedown", "circlecrop",
    "rectcrop", "distance", "fadeblack", "fadewhite", "radial",
    "smoothleft", "smoothright", "smoothup", "smoothdown",
    "circleopen", "circleclose", "vertopen", "vertclose",
    "horzopen", "horzclose", "dissolve", "pixelize", "diagtl",
    "diagtr", "diagbl", "diagbr"
];

const ENCODING_PRESETS = [
    "ultrafast", "superfast", "veryfast", "faster", "fast", "medium", "slow", "slower", "veryslow"
];

const RESOLUTIONS = ["720p", "1080p", "2k"];
const FPS_OPTIONS = [24, 30, 60];

export const MontageSection: React.FC<MontageSectionProps> = ({
    settings, handleChange, setSettings
}) => {
    const { t } = useI18n();

    // Utility for slider fill
    const getProgress = (val: number, min: number, max: number) => {
        return ((val - min) / (max - min)) * 100;
    };

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

                    {/* Resolution & FPS Row */}
                    <div className="settings-row">
                        <div className="settings-control" style={{ flex: 1 }}>
                            <label className="settings-label">
                                {t('pipeline.montage.resolution')}
                            </label>
                            <select
                                className="settings-select"
                                value={settings.montageResolution || '1080p'}
                                onChange={(e) => handleChange('montageResolution', e.target.value)}
                            >
                                {RESOLUTIONS.map(res => (
                                    <option key={res} value={res}>{res}</option>
                                ))}
                            </select>
                        </div>
                        <div className="settings-control" style={{ flex: 1 }}>
                            <label className="settings-label">
                                {t('pipeline.montage.fps')}
                            </label>
                            <select
                                className="settings-select"
                                value={settings.montageFPS || 30}
                                onChange={(e) => handleChange('montageFPS', parseInt(e.target.value))}
                            >
                                {FPS_OPTIONS.map(fps => (
                                    <option key={fps} value={fps}>{fps} FPS</option>
                                ))}
                            </select>
                        </div>
                    </div>

                    {/* Sway (Rocking) */}
                    <div className="settings-control">
                        <label className="settings-label">{t('pipeline.montage.sway')}</label>
                        <div className="settings-slider-container">
                            <input
                                type="range"
                                min="0"
                                max="3"
                                step="0.1"
                                className="settings-slider"
                                value={settings.montageSwayFactor || 1.0}
                                onChange={(e) => handleChange('montageSwayFactor', parseFloat(e.target.value))}
                                style={{ '--range-progress': `${getProgress(settings.montageSwayFactor || 1.0, 0, 3)}%` } as React.CSSProperties}
                            />
                            <span className="settings-slider-value">{(settings.montageSwayFactor || 1.0).toFixed(1)}x</span>
                        </div>
                    </div>

                    {/* Zoom Intensity */}
                    <div className="settings-control">
                        <label className="settings-label">{t('pipeline.montage.zoom')}</label>
                        <div className="settings-slider-container">
                            <input
                                type="range"
                                min="0"
                                max="3"
                                step="0.1"
                                className="settings-slider"
                                value={settings.montageZoomFactor || 1.0}
                                onChange={(e) => handleChange('montageZoomFactor', parseFloat(e.target.value))}
                                style={{ '--range-progress': `${getProgress(settings.montageZoomFactor || 1.0, 0, 3)}%` } as React.CSSProperties}
                            />
                            <span className="settings-slider-value">{(settings.montageZoomFactor || 1.0).toFixed(1)}x</span>
                        </div>
                    </div>

                    {/* Internal Upscale Factor */}
                    <div className="settings-control">
                        <label className="settings-label">{t('pipeline.montage.internal_upscale')}</label>
                        <div className="settings-slider-container">
                            <input
                                type="range"
                                min="1.0"
                                max="3.0"
                                step="0.1"
                                className="settings-slider"
                                value={settings.montageUpscaleFactor || 2.0}
                                onChange={(e) => handleChange('montageUpscaleFactor', parseFloat(e.target.value))}
                                style={{ '--range-progress': `${getProgress(settings.montageUpscaleFactor || 2.0, 1.0, 3.0)}%` } as React.CSSProperties}
                            />
                            <span className="settings-slider-value">{(settings.montageUpscaleFactor || 2.0).toFixed(1)}x</span>
                        </div>
                    </div>

                    {/* Transition Duration */}
                    <div className="settings-control">
                        <label className="settings-label">{t('pipeline.montage.transitions')}</label>
                        <div className="settings-slider-container">
                            <input
                                type="range"
                                min="0.1"
                                max="2"
                                step="0.05"
                                className="settings-slider"
                                value={settings.montageTransitionDuration || 0.5}
                                onChange={(e) => handleChange('montageTransitionDuration', parseFloat(e.target.value))}
                                style={{ '--range-progress': `${getProgress(settings.montageTransitionDuration || 0.5, 0.1, 2)}%` } as React.CSSProperties}
                            />
                            <span className="settings-slider-value">{(settings.montageTransitionDuration || 0.5).toFixed(2)}s</span>
                        </div>
                    </div>

                    {/* Transition Effect Selection */}
                    <div className="settings-control">
                        <label className="settings-label">
                            {t('pipeline.montage.transition_effect')}
                        </label>
                        <select
                            className="settings-select"
                            value={settings.montageTransitionEffect || 'fade'}
                            onChange={(e) => handleChange('montageTransitionEffect', e.target.value)}
                        >
                            {TRANSITION_EFFECTS.map(effect => (
                                <option key={effect} value={effect}>{effect}</option>
                            ))}
                        </select>
                    </div>

                    {/* Encoding Preset */}
                    <div className="settings-control">
                        <label className="settings-label">
                            {t('pipeline.montage.encoding_preset')}
                        </label>
                        <select
                            className="settings-select"
                            value={settings.montageEncodingPreset || 'medium'}
                            onChange={(e) => handleChange('montageEncodingPreset', e.target.value)}
                        >
                            {ENCODING_PRESETS.map(preset => (
                                <option key={preset} value={preset}>{preset}</option>
                            ))}
                        </select>
                    </div>

                    {/* Bitrate */}
                    <div className="settings-control">
                        <label className="settings-label">{t('pipeline.montage.bitrate')}</label>
                        <div className="settings-slider-container">
                            <input
                                type="range" min="1" max="50" step="1" className="settings-slider"
                                value={settings.montageBitrate || 15}
                                onChange={(e) => handleChange('montageBitrate', parseInt(e.target.value))}
                                style={{ '--range-progress': `${getProgress(settings.montageBitrate || 15, 1, 50)}%` } as React.CSSProperties}
                            />
                            <span className="settings-slider-value">{settings.montageBitrate || 15} Mbps</span>
                        </div>
                    </div>

                </div>
            </div>
        </div>
    );
};
