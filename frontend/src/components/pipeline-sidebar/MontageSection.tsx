import React from 'react';
import { useI18n } from '../../contexts/I18nContext';

interface MontageSectionProps {
    settings: any;
    handleChange: (field: string, value: any) => void;
    setSettings: React.Dispatch<React.SetStateAction<any>>;
    setCurrentPath?: (path: string) => void;
    isCollapsed?: boolean;
    onToggleCollapse?: (collapsed: boolean) => void;
}

const MontageIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
    </svg>
);

const TRANSITION_EFFECTS = [
    "fade_fast", "fade", "wipeleft", "wiperight", "wipeup", "wipedown",
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
    settings, handleChange, setSettings, setCurrentPath,
    isCollapsed: externalIsCollapsed, onToggleCollapse
}) => {
    const { t } = useI18n();

    const internalIsCollapsed = settings.montageCollapsed;
    const isCollapsed = externalIsCollapsed !== undefined ? externalIsCollapsed : internalIsCollapsed;

    const toggleCollapse = () => {
        if (onToggleCollapse) {
            onToggleCollapse(!isCollapsed);
        } else {
            handleChange('montageCollapsed', !isCollapsed);
        }
    };

    // Utility for slider fill
    const getProgress = (val: number, min: number, max: number) => {
        return ((val - min) / (max - min)) * 100;
    };

    return (
        <div className={`pipeline-stage-container ${isCollapsed ? 'is-collapsed' : ''}`}>
            <div
                className="pipeline-stage-header"
                onClick={toggleCollapse}
            >
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
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }} onClick={(e) => e.stopPropagation()}>
                    <button
                        className="templates-settings-link"
                        onClick={() => setCurrentPath?.('settings.performance')}
                        title={t('pipeline.performance_settings') || 'Performance Settings'}
                        style={{ margin: 0, padding: '4px' }}
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            <circle cx="12" cy="12" r="3" />
                            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
                        </svg>
                    </button>
                    <label className="stage-switch">
                        <input
                            type="checkbox"
                            checked={settings.montageEnabled}
                            onChange={(e) => {
                                const val = e.target.checked;
                                setSettings((prev: any) => ({
                                    ...prev,
                                    montageEnabled: val
                                }));
                            }}
                        />
                        <span className="stage-slider"></span>
                    </label>
                </div>
            </div>

            <div className={`stage-settings-content ${isCollapsed ? 'collapsed' : ''}`}>
                <div className="settings-group">
                    <div className="settings-group-title" style={{ marginBottom: '16px' }}>
                        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12 2v20M2 12h20"/><rect x="2" y="2" width="20" height="20" rx="2.18" ry="2.18"/></svg>
                        {t('pipeline.montage.group_overlays')}
                    </div>

                    {/* Intro Video Setting */}
                    <div className="settings-control">
                        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                            <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.montage.intro_video_enabled')}</label>
                            <label className="stage-switch small">
                                <input
                                    type="checkbox"
                                    checked={settings.montageIntroVideoEnabled || false}
                                    onChange={(e) => handleChange('montageIntroVideoEnabled', e.target.checked)}
                                />
                                <span className="stage-slider"></span>
                            </label>
                        </div>
                    </div>

                    {settings.montageIntroVideoEnabled && (
                        <div className="settings-control">
                            <div
                                onClick={async () => {
                                    try {
                                        const path = await (window as any).go.main.App.SelectVideo();
                                        if (path) {
                                            handleChange('montageIntroVideoPath', path);
                                        }
                                    } catch (err) {
                                        console.error(err);
                                    }
                                }}
                                style={{
                                    width: '100%',
                                    padding: '12px',
                                    borderRadius: '10px',
                                    border: settings.montageIntroVideoPath ? '1px solid var(--accent-color)' : '1px dashed var(--bg-tertiary)',
                                    backgroundColor: settings.montageIntroVideoPath ? 'rgba(var(--accent-rgb), 0.05)' : 'var(--bg-secondary)',
                                    display: 'flex',
                                    flexDirection: 'column',
                                    alignItems: 'center',
                                    justifyContent: 'center',
                                    gap: '6px',
                                    cursor: 'pointer',
                                    transition: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
                                    position: 'relative',
                                    overflow: 'hidden',
                                    minHeight: '80px'
                                }}
                            >
                                <div style={{
                                    fontSize: '20px',
                                    opacity: settings.montageIntroVideoPath ? 1 : 0.5,
                                    filter: settings.montageIntroVideoPath ? 'drop-shadow(0 0 8px var(--accent-color))' : 'none',
                                }}>
                                    {settings.montageIntroVideoPath ? '🎬' : '📁'}
                                </div>
                                <div style={{
                                    fontSize: '11px',
                                    fontWeight: '600',
                                    color: settings.montageIntroVideoPath ? 'var(--text-primary)' : 'var(--text-secondary)',
                                    textAlign: 'center'
                                }}>
                                    {settings.montageIntroVideoPath
                                        ? t('pipeline.montage.intro_video_change')
                                        : t('pipeline.montage.intro_video_select')}
                                </div>
                                {settings.montageIntroVideoPath && (
                                    <div style={{
                                        fontSize: '9px',
                                        color: 'var(--text-tertiary)',
                                        maxWidth: '100%',
                                        overflow: 'hidden',
                                        textOverflow: 'ellipsis',
                                        whiteSpace: 'nowrap',
                                        opacity: 0.8
                                    }}>
                                        {settings.montageIntroVideoPath.split(/[\\/]/).pop()}
                                    </div>
                                )}
                            </div>
                        </div>
                    )}

                    <div style={{ margin: '12px 0', borderTop: '1px solid var(--border-color)', opacity: 0.5 }} />

                    {/* Watermark Setting */}
                    <div className="settings-control">
                        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                            <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.montage.watermark_enabled')}</label>
                            <label className="stage-switch small">
                                <input
                                    type="checkbox"
                                    checked={settings.montageWatermarkEnabled || false}
                                    onChange={(e) => handleChange('montageWatermarkEnabled', e.target.checked)}
                                />
                                <span className="stage-slider"></span>
                            </label>
                        </div>
                    </div>

                    {settings.montageWatermarkEnabled && (
                        <>
                            {/* ... existing watermark controls ... */}
                            <div className="settings-control">
                                <div
                                    onClick={async () => {
                                        try {
                                            const path = await (window as any).go.main.App.SelectImage();
                                            if (path) {
                                                handleChange('montageWatermarkPath', path);
                                            }
                                        } catch (err) {
                                            console.error(err);
                                        }
                                    }}
                                    style={{
                                        width: '100%',
                                        padding: '12px',
                                        borderRadius: '10px',
                                        border: settings.montageWatermarkPath ? '1px solid var(--accent-color)' : '1px dashed var(--bg-tertiary)',
                                        backgroundColor: settings.montageWatermarkPath ? 'rgba(var(--accent-rgb), 0.05)' : 'var(--bg-secondary)',
                                        display: 'flex',
                                        flexDirection: 'column',
                                        alignItems: 'center',
                                        justifyContent: 'center',
                                        gap: '6px',
                                        cursor: 'pointer',
                                        transition: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
                                        position: 'relative',
                                        overflow: 'hidden',
                                        minHeight: '80px'
                                    }}
                                >
                                    <div style={{
                                        fontSize: '20px',
                                        opacity: settings.montageWatermarkPath ? 1 : 0.5,
                                        filter: settings.montageWatermarkPath ? 'drop-shadow(0 0 8px var(--accent-color))' : 'none',
                                    }}>
                                        {settings.montageWatermarkPath ? '🖼️' : '📁'}
                                    </div>
                                    <div style={{
                                        fontSize: '11px',
                                        fontWeight: '600',
                                        color: settings.montageWatermarkPath ? 'var(--text-primary)' : 'var(--text-secondary)',
                                        textAlign: 'center'
                                    }}>
                                        {settings.montageWatermarkPath
                                            ? t('pipeline.montage.watermark_change')
                                            : t('pipeline.montage.watermark_select')}
                                    </div>
                                    {settings.montageWatermarkPath && (
                                        <div style={{
                                            fontSize: '9px',
                                            color: 'var(--text-tertiary)',
                                            maxWidth: '100%',
                                            overflow: 'hidden',
                                            textOverflow: 'ellipsis',
                                            whiteSpace: 'nowrap',
                                            opacity: 0.8
                                        }}>
                                            {settings.montageWatermarkPath.split(/[\\/]/).pop()}
                                        </div>
                                    )}
                                </div>
                            </div>
                            {/* ... remaining watermark controls ... */}
                            <div className="settings-control">
                                <label className="settings-label">{t('pipeline.montage.watermark_position')}</label>
                                <select
                                    className="settings-select"
                                    value={settings.montageWatermarkPosition || 'bottom-right'}
                                    onChange={(e) => handleChange('montageWatermarkPosition', e.target.value)}
                                >
                                    <option value="top-left">{t('pipeline.montage.pos_top_left')}</option>
                                    <option value="top-center">{t('pipeline.montage.pos_top_center')}</option>
                                    <option value="top-right">{t('pipeline.montage.pos_top_right')}</option>
                                    <option value="bottom-left">{t('pipeline.montage.pos_bottom_left')}</option>
                                    <option value="bottom-center">{t('pipeline.montage.pos_bottom_center')}</option>
                                    <option value="bottom-right">{t('pipeline.montage.pos_bottom_right')}</option>
                                    <option value="center">{t('pipeline.montage.pos_center')}</option>
                                </select>
                            </div>

                            <div className="settings-control">
                                <label className="settings-label">{t('pipeline.montage.watermark_opacity')}</label>
                                <div className="settings-slider-container">
                                    <input
                                        type="range" min="0.1" max="1.0" step="0.05" className="settings-slider"
                                        value={settings.montageWatermarkOpacity || 0.8}
                                        onChange={(e) => handleChange('montageWatermarkOpacity', parseFloat(e.target.value))}
                                        style={{ '--range-progress': `${getProgress(settings.montageWatermarkOpacity || 0.8, 0.1, 1.0)}%` } as React.CSSProperties}
                                    />
                                    <span className="settings-slider-value">{(Number(settings.montageWatermarkOpacity) || 0.8).toFixed(2)}</span>
                                </div>
                            </div>

                            <div className="settings-control">
                                <label className="settings-label">{t('pipeline.montage.watermark_scale')}</label>
                                <div className="settings-slider-container">
                                    <input
                                        type="range" min="5" max="50" step="1" className="settings-slider"
                                        value={settings.montageWatermarkSize || 15}
                                        onChange={(e) => handleChange('montageWatermarkSize', parseInt(e.target.value))}
                                        style={{ '--range-progress': `${getProgress(settings.montageWatermarkSize || 15, 5, 50)}%` } as React.CSSProperties}
                                    />
                                    <span className="settings-slider-value">{settings.montageWatermarkSize || 15}%</span>
                                </div>
                            </div>

                            {settings.montageIntroVideoEnabled && (
                                <div className="settings-control">
                                    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                                        <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.montage.watermark_on_intro')}</label>
                                        <label className="stage-switch small">
                                            <input
                                                type="checkbox"
                                                checked={settings.montageWatermarkOnIntro || false}
                                                onChange={(e) => handleChange('montageWatermarkOnIntro', e.target.checked)}
                                            />
                                            <span className="stage-slider"></span>
                                        </label>
                                    </div>
                                </div>
                            )}
                        </>
                    )}

                    <div style={{ margin: '12px 0', borderTop: '1px solid var(--border-color)', opacity: 0.5 }} />

                    {/* Overlay Effects Setting */}
                    <div className="settings-control">
                        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                            <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.montage.overlay_enabled')}</label>
                            <label className="stage-switch small">
                                <input
                                    type="checkbox"
                                    checked={settings.montageOverlayEnabled || false}
                                    onChange={(e) => handleChange('montageOverlayEnabled', e.target.checked)}
                                />
                                <span className="stage-slider"></span>
                            </label>
                        </div>
                    </div>

                    {settings.montageOverlayEnabled && (
                        <>
                            <div className="settings-control">
                                <div
                                    onClick={async () => {
                                        try {
                                            const path = await (window as any).go.main.App.SelectVideo();
                                            if (path) {
                                                handleChange('montageOverlayPath', path);
                                            }
                                        } catch (err) {
                                            console.error(err);
                                        }
                                    }}
                                    style={{
                                        width: '100%',
                                        padding: '12px',
                                        borderRadius: '10px',
                                        border: settings.montageOverlayPath ? '1px solid var(--accent-color)' : '1px dashed var(--bg-tertiary)',
                                        backgroundColor: settings.montageOverlayPath ? 'rgba(var(--accent-rgb), 0.05)' : 'var(--bg-secondary)',
                                        display: 'flex',
                                        flexDirection: 'column',
                                        alignItems: 'center',
                                        justifyContent: 'center',
                                        gap: '6px',
                                        cursor: 'pointer',
                                        transition: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
                                        position: 'relative',
                                        overflow: 'hidden',
                                        minHeight: '80px'
                                    }}
                                >
                                    <div style={{
                                        fontSize: '20px',
                                        opacity: settings.montageOverlayPath ? 1 : 0.5,
                                        filter: settings.montageOverlayPath ? 'drop-shadow(0 0 8px var(--accent-color))' : 'none',
                                    }}>
                                        {settings.montageOverlayPath ? '✨' : '📁'}
                                    </div>
                                    <div style={{
                                        fontSize: '11px',
                                        fontWeight: '600',
                                        color: settings.montageOverlayPath ? 'var(--text-primary)' : 'var(--text-secondary)',
                                        textAlign: 'center'
                                    }}>
                                        {settings.montageOverlayPath
                                            ? t('pipeline.montage.overlay_change')
                                            : t('pipeline.montage.overlay_select')}
                                    </div>
                                    {settings.montageOverlayPath && (
                                        <div style={{
                                            fontSize: '9px',
                                            color: 'var(--text-tertiary)',
                                            maxWidth: '100%',
                                            overflow: 'hidden',
                                            textOverflow: 'ellipsis',
                                            whiteSpace: 'nowrap',
                                            opacity: 0.8
                                        }}>
                                            {settings.montageOverlayPath.split(/[\\/]/).pop()}
                                        </div>
                                    )}
                                </div>
                            </div>

                            {settings.montageIntroVideoEnabled && (
                                <div className="settings-control">
                                    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                                        <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.montage.overlay_on_intro')}</label>
                                        <label className="stage-switch small">
                                            <input
                                                type="checkbox"
                                                checked={settings.montageOverlayOnIntro || false}
                                                onChange={(e) => handleChange('montageOverlayOnIntro', e.target.checked)}
                                            />
                                            <span className="stage-slider"></span>
                                        </label>
                                    </div>
                                </div>
                            )}
                        </>
                    )}

                    <div style={{ margin: '12px 0', borderTop: '1px solid var(--border-color)', opacity: 0.5 }} />

                    {/* Overlay Triggers Setting */}
                    <div className="settings-control">
                        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                            <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.montage.overlay_triggers_enabled')}</label>
                            <label className="stage-switch small">
                                <input
                                    type="checkbox"
                                    checked={settings.montageOverlayTriggersEnabled || false}
                                    onChange={(e) => handleChange('montageOverlayTriggersEnabled', e.target.checked)}
                                />
                                <span className="stage-slider"></span>
                            </label>
                        </div>
                    </div>

                    {settings.montageOverlayTriggersEnabled && (
                        <div className="overlay-triggers-container" style={{ display: 'flex', flexDirection: 'column', gap: '12px', marginTop: '8px' }}>
                            {(settings.montageOverlayTriggers || []).map((trigger: any, index: number) => (
                                <div key={index} className="trigger-item" style={{
                                    padding: '12px',
                                    borderRadius: '10px',
                                    backgroundColor: 'var(--bg-secondary)',
                                    border: '1px solid var(--border-color)',
                                    display: 'flex',
                                    flexDirection: 'column',
                                    gap: '10px',
                                    position: 'relative'
                                }}>
                                    <button
                                        onClick={() => {
                                            const newTriggers = [...settings.montageOverlayTriggers];
                                            newTriggers.splice(index, 1);
                                            handleChange('montageOverlayTriggers', newTriggers);
                                        }}
                                        style={{
                                            position: 'absolute',
                                            top: '8px',
                                            right: '8px',
                                            background: 'none',
                                            border: 'none',
                                            color: 'var(--text-tertiary)',
                                            cursor: 'pointer',
                                            padding: '4px',
                                            borderRadius: '4px',
                                            display: 'flex',
                                            alignItems: 'center',
                                            justifyContent: 'center',
                                            transition: 'all 0.2s'
                                        }}
                                        className="trigger-remove-btn"
                                    >
                                        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M18 6 6 18M6 6l12 12" /></svg>
                                    </button>

                                    <div className="settings-control" style={{ marginBottom: 0 }}>
                                        <label className="settings-label" style={{ fontSize: '11px' }}>{t('pipeline.montage.overlay_triggers_phrase')}</label>
                                        <input
                                            type="text"
                                            className="settings-input"
                                            value={trigger.phrase || ''}
                                            onChange={(e) => {
                                                const newTriggers = [...settings.montageOverlayTriggers];
                                                newTriggers[index] = { ...trigger, phrase: e.target.value };
                                                handleChange('montageOverlayTriggers', newTriggers);
                                            }}
                                            placeholder="..."
                                        />
                                    </div>

                                    <div className="settings-control" style={{ marginBottom: 0 }}>
                                        <label className="settings-label" style={{ fontSize: '11px' }}>{t('pipeline.montage.overlay_triggers_path')}</label>
                                        <div style={{ display: 'flex', gap: '6px' }}>
                                            <input
                                                type="text"
                                                className="settings-input"
                                                value={trigger.path || ''}
                                                readOnly
                                                style={{ flex: 1, cursor: 'default' }}
                                            />
                                            <button
                                                className="secondary-button small"
                                                onClick={async () => {
                                                    try {
                                                        const path = await (window as any).go.main.App.SelectVideo();
                                                        if (path) {
                                                            const newTriggers = [...settings.montageOverlayTriggers];
                                                            newTriggers[index] = { ...trigger, path: path };
                                                            handleChange('montageOverlayTriggers', newTriggers);
                                                        }
                                                    } catch (err) {
                                                        console.error(err);
                                                    }
                                                }}
                                                style={{ padding: '0 10px', borderRadius: '8px', minWidth: '40px' }}
                                            >
                                                📁
                                            </button>
                                        </div>
                                    </div>

                                    <div className="settings-row" style={{ gap: '10px' }}>
                                        <div className="settings-control" style={{ flex: 1, marginBottom: 0 }}>
                                            <label className="settings-label" style={{ fontSize: '11px' }}>{t('pipeline.montage.overlay_triggers_x')}</label>
                                            <input
                                                type="number"
                                                className="settings-input"
                                                value={trigger.x || 0}
                                                onChange={(e) => {
                                                    const newTriggers = [...settings.montageOverlayTriggers];
                                                    newTriggers[index] = { ...trigger, x: parseInt(e.target.value) || 0 };
                                                    handleChange('montageOverlayTriggers', newTriggers);
                                                }}
                                            />
                                        </div>
                                        <div className="settings-control" style={{ flex: 1, marginBottom: 0 }}>
                                            <label className="settings-label" style={{ fontSize: '11px' }}>{t('pipeline.montage.overlay_triggers_y')}</label>
                                            <input
                                                type="number"
                                                className="settings-input"
                                                value={trigger.y || 0}
                                                onChange={(e) => {
                                                    const newTriggers = [...settings.montageOverlayTriggers];
                                                    newTriggers[index] = { ...trigger, y: parseInt(e.target.value) || 0 };
                                                    handleChange('montageOverlayTriggers', newTriggers);
                                                }}
                                            />
                                        </div>
                                    </div>
                                </div>
                            ))}

                            <button
                                className="secondary-button"
                                onClick={() => {
                                    const newTriggers = [...(settings.montageOverlayTriggers || []), { phrase: '', path: '', x: 0, y: 0 }];
                                    handleChange('montageOverlayTriggers', newTriggers);
                                }}
                                style={{
                                    width: '100%',
                                    padding: '10px',
                                    borderRadius: '10px',
                                    fontSize: '12px',
                                    display: 'flex',
                                    alignItems: 'center',
                                    justifyContent: 'center',
                                    gap: '8px',
                                    border: '1px solid var(--border-color)',
                                    backgroundColor: 'var(--bg-secondary)',
                                    color: 'var(--text-primary)',
                                    fontWeight: 500,
                                    transition: 'all 0.2s',
                                    marginTop: '4px'
                                }}
                                onMouseOver={(e) => {
                                    e.currentTarget.style.backgroundColor = 'var(--bg-tertiary)';
                                    e.currentTarget.style.borderColor = 'var(--accent-primary)';
                                }}
                                onMouseOut={(e) => {
                                    e.currentTarget.style.backgroundColor = 'var(--bg-secondary)';
                                    e.currentTarget.style.borderColor = 'var(--border-color)';
                                }}
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--accent-primary)" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><path d="M5 12h14M12 5v14" /></svg>
                                {t('pipeline.montage.overlay_triggers_add')}
                            </button>
                        </div>
                    )}

                    <div className="settings-group-title" style={{ marginTop: '20px', marginBottom: '16px', borderTop: '1px solid var(--border-color)', paddingTop: '16px' }}>
                        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12 2v10"/><path d="M18.4 4.6a9 9 0 1 1-12.8 0"/><path d="m12 12 9 9"/></svg>
                        {t('pipeline.montage.group_quality')}
                    </div>

                    {/* Resolution & FPS Row */}
                    <div className="settings-row">
                        <div className="settings-control" style={{ flex: 1, marginBottom: 0 }}>
                            <label className="settings-label" style={{ fontSize: '11px' }}>
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
                        <div className="settings-control" style={{ flex: 1, marginBottom: 0 }}>
                            <label className="settings-label" style={{ fontSize: '11px' }}>
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

                    <div className="settings-control" style={{ marginTop: '12px' }}>
                        <label className="settings-label" style={{ fontSize: '11px' }}>{t('pipeline.montage.sway')}</label>
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
                            <span className="settings-slider-value">{(Number(settings.montageSwayFactor) || 1.0).toFixed(1)}x</span>
                        </div>
                    </div>

                    <div className="settings-control">
                        <label className="settings-label" style={{ fontSize: '11px' }}>{t('pipeline.montage.zoom')}</label>
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
                            <span className="settings-slider-value">{(Number(settings.montageZoomFactor) || 1.0).toFixed(1)}x</span>
                        </div>
                    </div>

                    <div className="settings-control">
                        <label className="settings-label" style={{ fontSize: '11px' }}>{t('pipeline.montage.internal_upscale')}</label>
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
                            <span className="settings-slider-value">{(Number(settings.montageUpscaleFactor) || 2.0).toFixed(1)}x</span>
                        </div>
                    </div>

                    <div className="settings-control">
                        <label className="settings-label" style={{ fontSize: '11px' }}>{t('pipeline.montage.transitions')}</label>
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
                            <span className="settings-slider-value">{(Number(settings.montageTransitionDuration) || 0.5).toFixed(2)}s</span>
                        </div>
                    </div>

                    <div className="settings-control">
                        <label className="settings-label" style={{ fontSize: '11px' }}>
                            {t('pipeline.montage.transition_effect')}
                        </label>
                        <select
                            className="settings-select"
                            value={settings.montageTransitionEffect || 'fade'}
                            onChange={(e) => handleChange('montageTransitionEffect', e.target.value)}
                        >
                            {TRANSITION_EFFECTS.map(effect => (
                                <option key={effect} value={effect}>
                                    {effect === "fade_fast"
                                        ? `fade (${t('pipeline.montage.transition_fast')})`
                                        : `${effect} (${t('pipeline.montage.transition_slow')})`}
                                </option>
                            ))}
                        </select>
                    </div>

                    <div className="settings-control">
                        <label className="settings-label" style={{ fontSize: '11px' }}>
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

                    <div className="settings-control" style={{ marginBottom: 0 }}>
                        <label className="settings-label" style={{ fontSize: '11px' }}>{t('pipeline.montage.bitrate')}</label>
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
