import React from 'react';
import { useI18n } from '../../contexts/I18nContext';

interface ImageSectionProps {
    settings: any;
    handleChange: (field: string, value: any) => void;
    setSettings: React.Dispatch<React.SetStateAction<any>>;
    fetchPollinationsModels: () => void;
    pollinationsModels: string[];
    loadingPollinationsModels: boolean;
    estimatedChunks: number;
    content: string;
    models: string[];
    renderValueOrInput: (field: string, value: number, isFloat: boolean) => React.ReactNode;
    setCurrentPath?: (path: string) => void;
    elevenLabsImageKeys?: any[];
    isCollapsed?: boolean;
    onToggleCollapse?: (collapsed: boolean) => void;
}

const ImageIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
        <circle cx="8.5" cy="8.5" r="1.5" />
        <polyline points="21 15 16 10 5 21" />
    </svg>
);

export const ImageSection: React.FC<ImageSectionProps> = ({
    settings, handleChange, setSettings, fetchPollinationsModels, pollinationsModels, loadingPollinationsModels, estimatedChunks, content, models, renderValueOrInput, setCurrentPath, elevenLabsImageKeys,
    isCollapsed: externalIsCollapsed, onToggleCollapse
}) => {
    const { t } = useI18n();
    const [previewUrl, setPreviewUrl] = React.useState<string | null>(null);

    const internalIsCollapsed = settings.imageCollapsed;
    const isCollapsed = externalIsCollapsed !== undefined ? externalIsCollapsed : internalIsCollapsed;

    const toggleCollapse = () => {
        if (onToggleCollapse) {
            onToggleCollapse(!isCollapsed);
        } else {
            handleChange('imageCollapsed', !isCollapsed);
        }
    };

    React.useEffect(() => {
        if (settings.imageGooglerReferenceImage) {
            // Load preview
            const loadPreview = async () => {
                try {
                    const b64 = await (window as any).go.main.App.GetImageAsBase64(settings.imageGooglerReferenceImage);
                    if (b64) setPreviewUrl(b64);
                } catch (err) {
                    console.error("Failed to load preview:", err);
                    setPreviewUrl(null);
                }
            };
            loadPreview();
        } else {
            setPreviewUrl(null);
        }
    }, [settings.imageGooglerReferenceImage]);

    return (
        <div className={`pipeline-stage-container ${isCollapsed ? 'is-collapsed' : ''}`} >
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
                        background: settings.imageEnabled ? 'rgba(var(--accent-rgb), 0.1)' : 'var(--bg-tertiary)',
                        color: settings.imageEnabled ? 'var(--accent-color)' : 'var(--text-tertiary)',
                        transition: 'all 0.3s'
                    }}>
                        <ImageIcon />
                    </div>
                    <div style={{ display: 'flex', flexDirection: 'column' }}>
                        <span className="pipeline-stage-title">{t('pipeline.stage.image')}</span>
                        <span className="stage-status-text">
                            {settings.imageEnabled ? t('pipeline.stage.enabled') : t('pipeline.stage.disabled_simple')}
                        </span>
                    </div>
                </div>
                <label className="stage-switch" onClick={(e) => e.stopPropagation()}>
                    <input
                        type="checkbox"
                        checked={settings.imageEnabled}
                        onChange={(e) => {
                            const val = e.target.checked;
                            setSettings((prev: any) => ({
                                ...prev,
                                imageEnabled: val
                            }));
                        }}
                    />
                    <span className="stage-slider"></span>
                </label>
            </div>

            <div className={`stage-settings-content ${isCollapsed ? 'collapsed' : ''}`}>
                <div className="settings-group">

                    <div className="settings-group-title" style={{ marginBottom: '16px' }}>
                        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="2" y="7" width="20" height="15" rx="2" ry="2"/><polyline points="17 2 12 7 7 2"/></svg>
                        {t('pipeline.image.video_sequence_mode')}
                    </div>

                    <div className="settings-control" style={{ marginBottom: '16px', paddingBottom: '16px', borderBottom: '1px solid var(--border-color)' }}>
                        <div style={{ display: 'flex', background: 'var(--bg-tertiary)', borderRadius: '8px', padding: '4px', gap: '4px' }}>
                            <button
                                title={t('pipeline.image.video_distribution_sequential_desc')}
                                className={`method-toggle-btn ${(settings.imageVideoDistribution || 'sequential') === 'sequential' ? 'active' : ''}`}
                                onClick={() => handleChange('imageVideoDistribution', 'sequential')}
                                style={{
                                    flex: 1, padding: '6px', borderRadius: '6px', border: 'none',
                                    background: (settings.imageVideoDistribution || 'sequential') === 'sequential' ? 'var(--bg-primary)' : 'transparent',
                                    color: (settings.imageVideoDistribution || 'sequential') === 'sequential' ? 'var(--text-primary)' : 'var(--text-secondary)',
                                    cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '6px',
                                    fontSize: '12px', fontWeight: (settings.imageVideoDistribution || 'sequential') === 'sequential' ? 500 : 400,
                                    boxShadow: (settings.imageVideoDistribution || 'sequential') === 'sequential' ? '0 2px 4px rgba(0,0,0,0.1)' : 'none',
                                    transition: 'all 0.2s'
                                }}
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/></svg>
                                {t('pipeline.image.video_distribution_sequential')}
                            </button>
                            <button
                                title={t('pipeline.image.video_distribution_random_desc')}
                                className={`method-toggle-btn ${settings.imageVideoDistribution === 'random' ? 'active' : ''}`}
                                onClick={() => handleChange('imageVideoDistribution', 'random')}
                                style={{
                                    flex: 1, padding: '6px', borderRadius: '6px', border: 'none',
                                    background: settings.imageVideoDistribution === 'random' ? 'var(--bg-primary)' : 'transparent',
                                    color: settings.imageVideoDistribution === 'random' ? 'var(--text-primary)' : 'var(--text-secondary)',
                                    cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '6px',
                                    fontSize: '12px', fontWeight: settings.imageVideoDistribution === 'random' ? 500 : 400,
                                    boxShadow: settings.imageVideoDistribution === 'random' ? '0 2px 4px rgba(0,0,0,0.1)' : 'none',
                                    transition: 'all 0.2s'
                                }}
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="16 3 21 3 21 8"/><line x1="4" y1="20" x2="21" y2="3"/><polyline points="21 16 21 21 16 21"/><line x1="15" y1="15" x2="21" y2="21"/></svg>
                                {t('pipeline.image.video_distribution_random')}
                            </button>
                            <button
                                title={t('pipeline.image.video_distribution_subtitle_desc')}
                                className={`method-toggle-btn ${settings.imageVideoDistribution === 'subtitle_duration' ? 'active' : ''}`}
                                onClick={() => handleChange('imageVideoDistribution', 'subtitle_duration')}
                                style={{
                                    flex: 1, padding: '6px', borderRadius: '6px', border: 'none',
                                    background: settings.imageVideoDistribution === 'subtitle_duration' ? 'var(--bg-primary)' : 'transparent',
                                    color: settings.imageVideoDistribution === 'subtitle_duration' ? 'var(--text-primary)' : 'var(--text-secondary)',
                                    cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '6px',
                                    fontSize: '12px', fontWeight: settings.imageVideoDistribution === 'subtitle_duration' ? 500 : 400,
                                    boxShadow: settings.imageVideoDistribution === 'subtitle_duration' ? '0 2px 4px rgba(0,0,0,0.1)' : 'none',
                                    transition: 'all 0.2s'
                                }}
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/></svg>
                                {t('pipeline.image.video_distribution_subtitle')}
                            </button>
                        </div>
                        <div style={{ fontSize: '11px', color: 'var(--text-secondary)', marginTop: '8px', lineHeight: '1.4' }}>
                            {settings.imageVideoDistribution === 'random'
                                ? t('pipeline.image.video_distribution_random_desc')
                                : settings.imageVideoDistribution === 'subtitle_duration'
                                ? t('pipeline.image.video_distribution_subtitle_desc')
                                : t('pipeline.image.video_distribution_sequential_desc')}
                        </div>
                        {settings.imageVideoDistribution === 'subtitle_duration' && (
                            <div style={{ marginTop: '12px' }}>
                                <label className="settings-label" style={{ fontSize: '11px' }}>
                                    {t('pipeline.image.video_subtitle_threshold')}
                                </label>
                                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginTop: '6px' }}>
                                    <input
                                        type="range"
                                        className="settings-slider"
                                        min="1"
                                        max="15"
                                        step="0.5"
                                        value={settings.imageVideoSubtitleThreshold ?? 3}
                                        style={{ '--range-progress': `${((settings.imageVideoSubtitleThreshold ?? 3) - 1) / 14 * 100}%`, flex: 1 } as React.CSSProperties}
                                        onChange={(e) => handleChange('imageVideoSubtitleThreshold', parseFloat(e.target.value))}
                                    />
                                    <span style={{ fontSize: '12px', minWidth: '32px', textAlign: 'right', fontWeight: 500 }}>
                                        {(settings.imageVideoSubtitleThreshold ?? 3).toFixed(1)}s
                                    </span>
                                </div>
                            </div>
                        )}
                        {settings.imageVideoDistribution === 'random' && (
                            <div style={{ marginTop: '12px' }}>
                                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                                    <label className="settings-label" style={{ marginBottom: 0 }} title={t('pipeline.image.video_start_count_desc')}>
                                        {t('pipeline.image.video_start_count')}
                                    </label>
                                    <label className="stage-switch small">
                                        <input
                                            type="checkbox"
                                            checked={(settings.imageVideoStartCount ?? 0) > 0}
                                            onChange={(e) => handleChange('imageVideoStartCount', e.target.checked ? 1 : 0)}
                                        />
                                        <span className="stage-slider"></span>
                                    </label>
                                </div>
                                {(settings.imageVideoStartCount ?? 0) > 0 && (
                                    <input
                                        type="number"
                                        min="1"
                                        max="99"
                                        value={settings.imageVideoStartCount ?? 1}
                                        onChange={(e) => handleChange('imageVideoStartCount', Math.max(1, parseInt(e.target.value) || 1))}
                                        style={{
                                            marginTop: '8px', width: '100%', padding: '6px 10px',
                                            borderRadius: '6px', border: '1px solid var(--border-color)',
                                            background: 'var(--bg-tertiary)', color: 'var(--text-primary)',
                                            fontSize: '13px', boxSizing: 'border-box'
                                        }}
                                    />
                                )}
                            </div>
                        )}
                    </div>

                    <div className="settings-group-title" style={{ marginBottom: '16px' }}>
                        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M21 2v6h-6"/><path d="M3 12a9 9 0 0 1 15-6.7L21 8"/><path d="M3 22v-6h6"/><path d="M21 12a9 9 0 0 1-15 6.7L3 16"/></svg>
                        {t('pipeline.image.sync_mode')}
                    </div>

                    <div className="settings-control" style={{ marginBottom: '16px', paddingBottom: '16px', borderBottom: '1px solid var(--border-color)' }}>
                        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                            <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.image.sync_enabled')}</label>
                            <label className="stage-switch small">
                                <input
                                    type="checkbox"
                                    checked={settings.imageSyncEnabled || false}
                                    onChange={(e) => handleChange('imageSyncEnabled', e.target.checked)}
                                />
                                <span className="stage-slider"></span>
                            </label>
                        </div>


                    </div>

                    <div className="settings-group-title" style={{ marginTop: '20px', marginBottom: '16px' }}>
                        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
                        {t('pipeline.image.generation_method')}
                    </div>

                    <div className="settings-control">
                        <div style={{ display: 'flex', background: 'var(--bg-tertiary)', borderRadius: '8px', padding: '4px', gap: '4px' }}>
                            <button
                                className={`method-toggle-btn ${settings.imageGenerationMethod === 'lines' ? 'active' : ''}`}
                                onClick={() => handleChange('imageGenerationMethod', 'lines')}
                                style={{
                                    flex: 1, padding: '6px', borderRadius: '6px', border: 'none',
                                    background: settings.imageGenerationMethod === 'lines' ? 'var(--bg-primary)' : 'transparent',
                                    color: settings.imageGenerationMethod === 'lines' ? 'var(--text-primary)' : 'var(--text-secondary)',
                                    cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '6px',
                                    fontSize: '12px', fontWeight: settings.imageGenerationMethod === 'lines' ? 500 : 400,
                                    boxShadow: settings.imageGenerationMethod === 'lines' ? '0 2px 4px rgba(0,0,0,0.1)' : 'none',
                                    transition: 'all 0.2s'
                                }}
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><line x1="8" y1="6" x2="21" y2="6"></line><line x1="8" y1="12" x2="21" y2="12"></line><line x1="8" y1="18" x2="21" y2="18"></line><line x1="3" y1="6" x2="3.01" y2="6"></line><line x1="3" y1="12" x2="3.01" y2="12"></line><line x1="3" y1="18" x2="3.01" y2="18"></line></svg>
                                {t('pipeline.image.lines') || 'Строки'}
                            </button>
                            <button
                                className={`method-toggle-btn ${settings.imageGenerationMethod !== 'lines' ? 'active' : ''}`}
                                onClick={() => handleChange('imageGenerationMethod', 'sentences')}
                                style={{
                                    flex: 1, padding: '6px', borderRadius: '6px', border: 'none',
                                    background: settings.imageGenerationMethod !== 'lines' ? 'var(--bg-primary)' : 'transparent',
                                    color: settings.imageGenerationMethod !== 'lines' ? 'var(--text-primary)' : 'var(--text-secondary)',
                                    cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '6px',
                                    fontSize: '12px', fontWeight: settings.imageGenerationMethod !== 'lines' ? 500 : 400,
                                    boxShadow: settings.imageGenerationMethod !== 'lines' ? '0 2px 4px rgba(0,0,0,0.1)' : 'none',
                                    transition: 'all 0.2s'
                                }}
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="4 7 4 4 20 4 20 7"></polyline><line x1="9" y1="20" x2="15" y2="20"></line><line x1="12" y1="4" x2="12" y2="20"></line></svg>
                                {t('pipeline.image.sentences') || 'Предложения'}
                            </button>
                        </div>
                    </div>

                    {settings.imageGenerationMethod === 'sentences' && (
                        <div className="settings-control">
                            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                                <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.image.group_limit') || 'Группировать по лимиту символов'}</label>
                                <label className="stage-switch small">
                                    <input
                                        type="checkbox"
                                        checked={settings.imageGroupSentences}
                                        onChange={(e) => handleChange('imageGroupSentences', e.target.checked)}
                                    />
                                    <span className="stage-slider"></span>
                                </label>
                            </div>


                            {settings.imageGroupSentences && (
                                <div className="settings-slider-container" style={{ marginTop: '8px' }}>
                                    <span style={{ fontSize: '11px', color: 'var(--text-secondary)' }}>{t('pipeline.image.symbol_limit') || 'Ліміт символів:'} {settings.imageSentenceLimit ?? 1000}</span>
                                    <input
                                        type="range"
                                        className="settings-slider"
                                        min="50"
                                        max="5000"
                                        step="50"
                                        value={settings.imageSentenceLimit ?? 1000}
                                        style={{ '--range-progress': `${((settings.imageSentenceLimit ?? 1000) - 50) / 4950 * 100}%`, marginTop: '8px', width: '100%' } as React.CSSProperties}
                                        onChange={(e) => handleChange('imageSentenceLimit', parseInt(e.target.value))}
                                    />
                                </div>
                            )}
                        </div>
                    )}

                    {(settings.imageGenerationMethod === 'lines' || (settings.imageGenerationMethod === 'sentences' && settings.imageGroupSentences)) && (
                        <div className="settings-control" style={{ marginTop: '12px' }}>
                            <label className="settings-label">{t('pipeline.image.initial_sentences') || 'Динамічний початок (речень)'}</label>

                            <div className="settings-slider-container">
                                <input
                                    type="range"
                                    className="settings-slider"
                                    min="0"
                                    max="100"
                                    step="1"
                                    value={settings.imageInitialSentenceCount ?? 0}
                                    style={{ '--range-progress': `${((settings.imageInitialSentenceCount ?? 0) / 100) * 100}%` } as React.CSSProperties}
                                    onChange={(e) => handleChange('imageInitialSentenceCount', parseInt(e.target.value))}
                                />
                                <span style={{ fontSize: '12px', minWidth: '24px', textAlign: 'right', fontWeight: 500 }}>{settings.imageInitialSentenceCount ?? 0}</span>
                            </div>
                        </div>
                    )}

                    <div className="settings-group-title" style={{ marginTop: '20px', marginBottom: '16px', borderTop: '1px solid var(--border-color)', paddingTop: '16px' }}>
                        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12 20h.01"/><path d="M12 16h.01"/><path d="M12 12h.01"/><path d="M12 8h.01"/><path d="M12 4h.01"/></svg>
                        {t('pipeline.image.mode')}
                    </div>

                    <div className="settings-control">
                        <div style={{ display: 'flex', background: 'var(--bg-tertiary)', borderRadius: '8px', padding: '4px', gap: '4px', marginBottom: '12px' }}>
                            <button
                                className={`method-toggle-btn ${(settings.imageMode || 'normal') === 'normal' ? 'active' : ''}`}
                                onClick={() => handleChange('imageMode', 'normal')}
                                style={{
                                    flex: 1, padding: '6px', borderRadius: '6px', border: 'none',
                                    background: (settings.imageMode || 'normal') === 'normal' ? 'var(--bg-primary)' : 'transparent',
                                    color: (settings.imageMode || 'normal') === 'normal' ? 'var(--text-primary)' : 'var(--text-secondary)',
                                    cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '6px',
                                    fontSize: '12px', fontWeight: (settings.imageMode || 'normal') === 'normal' ? 500 : 400,
                                    boxShadow: (settings.imageMode || 'normal') === 'normal' ? '0 2px 4px rgba(0,0,0,0.1)' : 'none',
                                    transition: 'all 0.2s'
                                }}
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10"/><line x1="8" y1="12" x2="16" y2="12"/></svg>
                                {t('pipeline.image.mode_normal') || 'Звичайний'}
                            </button>
                            <button
                                className={`method-toggle-btn ${settings.imageMode === 'memory' ? 'active' : ''}`}
                                onClick={() => handleChange('imageMode', 'memory')}
                                style={{
                                    flex: 1, padding: '6px', borderRadius: '6px', border: 'none',
                                    background: settings.imageMode === 'memory' ? 'var(--bg-primary)' : 'transparent',
                                    color: settings.imageMode === 'memory' ? 'var(--text-primary)' : 'var(--text-secondary)',
                                    cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '6px',
                                    fontSize: '12px', fontWeight: settings.imageMode === 'memory' ? 500 : 400,
                                    boxShadow: settings.imageMode === 'memory' ? '0 2px 4px rgba(0,0,0,0.1)' : 'none',
                                    transition: 'all 0.2s'
                                }}
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="16"/><line x1="8" y1="12" x2="16" y2="12"/></svg>
                                {t('pipeline.image.mode_memory') || 'Пам\'ять'}
                            </button>
                        </div>

                        <div className="settings-control">
                            {settings.imageMode === 'memory' && (
                                <div style={{ display: 'flex', background: 'var(--bg-tertiary)', borderRadius: '8px', padding: '4px', gap: '4px', marginBottom: '12px' }}>
                                    <button
                                        className={`method-toggle-btn ${(settings.imageMemoryType || 'primitive') === 'primitive' ? 'active' : ''}`}
                                        onClick={() => handleChange('imageMemoryType', 'primitive')}
                                        style={{
                                            flex: 1, padding: '6px', borderRadius: '6px', border: 'none',
                                            background: (settings.imageMemoryType || 'primitive') === 'primitive' ? 'var(--bg-primary)' : 'transparent',
                                            color: (settings.imageMemoryType || 'primitive') === 'primitive' ? 'var(--text-primary)' : 'var(--text-secondary)',
                                            cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '6px',
                                            fontSize: '12px', fontWeight: (settings.imageMemoryType || 'primitive') === 'primitive' ? 500 : 400,
                                            boxShadow: (settings.imageMemoryType || 'primitive') === 'primitive' ? '0 2px 4px rgba(0,0,0,0.1)' : 'none',
                                            transition: 'all 0.2s'
                                        }}
                                    >
                                        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M13 2 3 14h9l-1 8 10-12h-9l1-8z"/></svg>
                                        {t('pipeline.image.memory_type_primitive') || 'Коротка'}
                                    </button>
                                    <button
                                        className={`method-toggle-btn ${settings.imageMemoryType === 'story' ? 'active' : ''}`}
                                        onClick={() => handleChange('imageMemoryType', 'story')}
                                        style={{
                                            flex: 1, padding: '6px', borderRadius: '6px', border: 'none',
                                            background: settings.imageMemoryType === 'story' ? 'var(--bg-primary)' : 'transparent',
                                            color: settings.imageMemoryType === 'story' ? 'var(--text-primary)' : 'var(--text-secondary)',
                                            cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '6px',
                                            fontSize: '12px', fontWeight: settings.imageMemoryType === 'story' ? 500 : 400,
                                            boxShadow: settings.imageMemoryType === 'story' ? '0 2px 4px rgba(0,0,0,0.1)' : 'none',
                                            transition: 'all 0.2s'
                                        }}
                                    >
                                        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"/><path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"/></svg>
                                        {t('pipeline.image.memory_type_story') || 'Історія'}
                                    </button>
                                    <button
                                        className={`method-toggle-btn ${settings.imageMemoryType === 'external' ? 'active' : ''}`}
                                        onClick={() => handleChange('imageMemoryType', 'external')}
                                        style={{
                                            flex: 1, padding: '6px', borderRadius: '6px', border: 'none',
                                            background: settings.imageMemoryType === 'external' ? 'var(--bg-primary)' : 'transparent',
                                            color: settings.imageMemoryType === 'external' ? 'var(--text-primary)' : 'var(--text-secondary)',
                                            cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '6px',
                                            fontSize: '12px', fontWeight: settings.imageMemoryType === 'external' ? 500 : 400,
                                            boxShadow: settings.imageMemoryType === 'external' ? '0 2px 4px rgba(0,0,0,0.1)' : 'none',
                                            transition: 'all 0.2s'
                                        }}
                                    >
                                        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M3 5V19A9 3 0 0 0 21 19V5"/><path d="M3 12A9 3 0 0 0 21 12"/></svg>
                                        {t('pipeline.image.memory_type_external') || 'Повна'}
                                    </button>
                                </div>
                            )}

                            {((settings.imageMode || 'normal') === 'normal' || ((settings.imageMode === 'memory') && ((settings.imageMemoryType || 'primitive') === 'primitive' || settings.imageMemoryType === 'external' || settings.imageMemoryType === 'story'))) && (
                                <>
                                    {settings.imageMode === 'memory' && (settings.imageMemoryType || 'primitive') === 'primitive' && (
                                        <div className="settings-slider-container" style={{ marginBottom: '16px', paddingBottom: '16px', borderBottom: '1px solid var(--border-color)' }}>
                                            <span style={{ fontSize: '11px', color: 'var(--text-secondary)' }}>{t('pipeline.image.memory_chars') || "Кількість символів пам'яті:"} {settings.imageMemoryChars ?? 1000}</span>
                                            <input
                                                type="range"
                                                className="settings-slider"
                                                min="500"
                                                max="5000"
                                                step="100"
                                                value={settings.imageMemoryChars ?? 1000}
                                                style={{ '--range-progress': `${((settings.imageMemoryChars ?? 1000) - 500) / 4500 * 100}%`, marginTop: '8px', width: '100%' } as React.CSSProperties}
                                                onChange={(e) => handleChange('imageMemoryChars', parseInt(e.target.value))}
                                            />
                                        </div>
                                    )}

                                    <div className="settings-group-title" style={{ marginTop: '20px', marginBottom: '16px' }}>
                                        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg>
                                        {t('pipeline.group.prompt')}
                                    </div>

                                    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%', marginTop: '0px' }}>
                                        <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.image.determine_characters') || 'Визначити персонажів'}</label>
                                        <label className="stage-switch small">
                                            <input
                                                type="checkbox"
                                                checked={settings.imageDetermineCharacters || false}
                                                onChange={(e) => handleChange('imageDetermineCharacters', e.target.checked)}
                                            />
                                            <span className="stage-slider"></span>
                                        </label>
                                    </div>


                                    {settings.imageDetermineCharacters && (
                                        <div style={{ marginTop: '12px' }}>
                                            <label className="settings-label" style={{ fontSize: '11px' }}>{t('pipeline.image.determine_characters_mode')}</label>
                                            <div style={{ display: 'flex', background: 'var(--bg-tertiary)', borderRadius: '8px', padding: '4px', gap: '4px', marginBottom: '12px' }}>
                                                <button
                                                    className={`method-toggle-btn ${(settings.imageDetermineCharactersMode || 'dynamic') === 'dynamic' ? 'active' : ''}`}
                                                    onClick={() => handleChange('imageDetermineCharactersMode', 'dynamic')}
                                                    style={{
                                                        flex: 1, padding: '6px', borderRadius: '6px', border: 'none',
                                                        background: (settings.imageDetermineCharactersMode || 'dynamic') === 'dynamic' ? 'var(--bg-primary)' : 'transparent',
                                                        color: (settings.imageDetermineCharactersMode || 'dynamic') === 'dynamic' ? 'var(--text-primary)' : 'var(--text-secondary)',
                                                        cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '6px',
                                                        fontSize: '11px', fontWeight: (settings.imageDetermineCharactersMode || 'dynamic') === 'dynamic' ? 500 : 400,
                                                        boxShadow: (settings.imageDetermineCharactersMode || 'dynamic') === 'dynamic' ? '0 2px 4px rgba(0,0,0,0.1)' : 'none',
                                                        transition: 'all 0.2s'
                                                    }}
                                                >
                                                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z"/><path d="M5 3v4"/><path d="M19 17v4"/><path d="M3 5h4"/><path d="M17 19h4"/></svg>
                                                    {t('pipeline.image.determine_characters_mode_dynamic')}
                                                </button>
                                                <button
                                                    className={`method-toggle-btn ${settings.imageDetermineCharactersMode === 'static' ? 'active' : ''}`}
                                                    onClick={() => handleChange('imageDetermineCharactersMode', 'static')}
                                                    style={{
                                                        flex: 1, padding: '6px', borderRadius: '6px', border: 'none',
                                                        background: settings.imageDetermineCharactersMode === 'static' ? 'var(--bg-primary)' : 'transparent',
                                                        color: settings.imageDetermineCharactersMode === 'static' ? 'var(--text-primary)' : 'var(--text-secondary)',
                                                        cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '6px',
                                                        fontSize: '11px', fontWeight: settings.imageDetermineCharactersMode === 'static' ? 500 : 400,
                                                        boxShadow: settings.imageDetermineCharactersMode === 'static' ? '0 2px 4px rgba(0,0,0,0.1)' : 'none',
                                                        transition: 'all 0.2s'
                                                    }}
                                                >
                                                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
                                                    {t('pipeline.image.determine_characters_mode_static')}
                                                </button>
                                            </div>

                                            {(settings.imageDetermineCharactersMode || 'dynamic') === 'dynamic' ? (
                                                <div className="settings-control">
                                                    <label className="settings-label">{t('pipeline.image.determine_characters_prompt')}</label>
                                                    <textarea
                                                        className="settings-textarea"
                                                        style={{ height: '80px', resize: 'vertical' }}
                                                        value={settings.imageDetermineCharactersPrompt || ''}
                                                        onChange={(e) => handleChange('imageDetermineCharactersPrompt', e.target.value)}
                                                        placeholder={t('pipeline.image.determine_characters_prompt_desc')}
                                                    />
                                                </div>
                                            ) : (
                                                <div className="settings-control">
                                                    <label className="settings-label">{t('pipeline.image.determine_characters_static_desc')}</label>
                                                    <textarea
                                                        className="settings-textarea"
                                                        style={{ height: '80px', resize: 'vertical' }}
                                                        value={settings.imageDetermineCharactersStatic || ''}
                                                        onChange={(e) => handleChange('imageDetermineCharactersStatic', e.target.value)}
                                                        placeholder={t('pipeline.image.determine_characters_static_placeholder')}
                                                    />
                                                </div>
                                            )}
                                        </div>
                                    )}
                                </>
                            )}
                        </div>
                    </div>

                    <div className="settings-control">

                        <label className="settings-label">{t('pipeline.image.prompt') || 'Промт для інструкцій'}</label>
                        <textarea
                            className="settings-textarea"
                            style={{ height: '80px', resize: 'vertical' }}
                            value={settings.imagePrompt || ''}
                            onChange={(e) => handleChange('imagePrompt', e.target.value)}
                            placeholder={t('pipeline.image.prompt_placeholder') || 'Введіть промт...'}
                        />
                        <div className="settings-description" style={{ fontSize: '11px', color: 'var(--text-secondary)', marginTop: '8px', lineHeight: '1.4' }}>
                            <div style={{ marginBottom: '4px', opacity: 0.8 }}>{t('pipeline.image.available_tags') || 'Доступні теги:'}</div>
                            <ul style={{ margin: 0, paddingLeft: '0', listStyleType: 'none', display: 'flex', flexDirection: 'column', gap: '3px' }}>
                                <li style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                                    <code style={{ color: 'var(--accent-primary)', background: 'rgba(var(--accent-rgb), 0.1)', padding: '1px 4px', borderRadius: '4px', fontSize: '10px', fontWeight: 700 }}>{'{{content}}'}</code>
                                    <span>— {t('pipeline.image.tag_content') || 'поточний текст (завжди)'}</span>
                                </li>
                                {settings.imageDetermineCharacters && (
                                    <li style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                                        <code style={{ color: 'var(--accent-primary)', background: 'rgba(var(--accent-rgb), 0.1)', padding: '1px 4px', borderRadius: '4px', fontSize: '10px', fontWeight: 700 }}>{'{{characters}}'}</code>
                                        <span>— {t('pipeline.image.tag_characters') || 'опис персонажів'}</span>
                                    </li>
                                )}
                                {settings.imageMode === 'memory' && (settings.imageMemoryType || 'primitive') === 'primitive' && (
                                    <li style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                                        <code style={{ color: 'var(--accent-primary)', background: 'rgba(var(--accent-rgb), 0.1)', padding: '1px 4px', borderRadius: '4px', fontSize: '10px', fontWeight: 700 }}>{'{{memory}}'}</code>
                                        <span>— {t('pipeline.image.tag_memory') || 'контекст пам\'яті'}</span>
                                    </li>
                                )}
                                {settings.imageMode === 'memory' && settings.imageMemoryType === 'story' && (
                                    <li style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                                        <code style={{ color: 'var(--accent-primary)', background: 'rgba(var(--accent-rgb), 0.1)', padding: '1px 4px', borderRadius: '4px', fontSize: '10px', fontWeight: 700 }}>{'{{story}}'}</code>
                                        <span>— {t('pipeline.image.tag_story') || 'переказ всієї історії'}</span>
                                    </li>
                                )}
                            </ul>
                        </div>

                        {content && content.trim() !== '' && (
                            <div style={{ fontSize: '11px', color: 'var(--accent-primary)', marginTop: '8px', fontWeight: 500, display: 'flex', alignItems: 'center', gap: '6px' }}>
                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10"></circle><polyline points="12 6 12 12 16 14"></polyline></svg>
                                {t('pipeline.image.estimated_chunks') || 'Орієнтовна кількість промптів: '} {estimatedChunks}
                            </div>
                        )}
                    </div>

                </div>

                <div className="settings-group">
                    <div className="settings-group-title">
                        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12 2a10 10 0 1 0 10 10H12V2z" /><path d="M12 12 2.1 12a10.05 10.05 0 0 1 9.9-10v10z" /><path d="m9 16.5 3-3" /></svg>
                        {t('pipeline.group.ai')}
                    </div>

                    <div className="settings-control">
                        <label className="settings-label">{t('pipeline.model')}</label>
                        <select
                            className="settings-select"
                            value={settings.imagePromptModel || ''}
                            onChange={(e) => {
                                const val = e.target.value;
                                if (val === "ADD_NEW_MODEL") {
                                    if (setCurrentPath) setCurrentPath('settings.api.openrouter');
                                    return;
                                }
                                handleChange('imagePromptModel', val);
                            }}
                        >
                            <option value="">{t('pipeline.model.default')}</option>
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
                                value={settings.imagePromptTemperature ?? 0.7}
                                style={{ '--range-progress': `${((settings.imagePromptTemperature ?? 0.7) / 2) * 100}%` } as React.CSSProperties}
                                onChange={(e) => handleChange('imagePromptTemperature', parseFloat(e.target.value))}
                            />
                            {renderValueOrInput('imagePromptTemperature', settings.imagePromptTemperature ?? 0.7, true)}
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
                                value={settings.imagePromptMaxTokens ?? 0}
                                style={{ '--range-progress': `${((settings.imagePromptMaxTokens ?? 0) / 128000) * 100}%` } as React.CSSProperties}
                                onChange={(e) => handleChange('imagePromptMaxTokens', parseFloat(e.target.value))}
                            />
                            {renderValueOrInput('imagePromptMaxTokens', settings.imagePromptMaxTokens ?? 0, false)}
                        </div>
                    </div>
                </div>

                <div className="settings-group">
                    <div className="settings-group-title">
                        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polygon points="12 2 2 7 12 12 22 7 12 2"></polygon><polyline points="2 17 12 22 22 17"></polyline><polyline points="2 12 12 17 22 12"></polyline></svg>
                        {t('pipeline.group.provider')}
                    </div>
                    <div className="settings-control">
                        <label className="settings-label">{t('pipeline.image.service')}</label>
                        <select
                            className="settings-select"
                            value={settings.imageService}
                            onChange={(e) => {
                                const val = e.target.value;
                                handleChange('imageService', val);
                                if (val === 'pollinations') {
                                    fetchPollinationsModels();
                                }
                            }}
                        >
                            <option value="pollinations">{t('image.pollinationsai') || 'Pollinations.ai'}</option>
                            <option value="googler">{t('image.googler') || 'Googler'}</option>
                            <option value="elevenlabsimage">{t('image.elevenlabsimage') || 'ElevenLabs Image'}</option>
                        </select>
                    </div>

                    {settings.imageService === 'pollinations' && (
                        <>
                            <div className="settings-control">
                                <label className="settings-label">{t('pipeline.image.model')}</label>
                                <div style={{ display: 'flex', gap: '8px' }}>
                                    <select
                                        className="settings-select"
                                        style={{ flex: 1 }}
                                        value={settings.imageModel}
                                        onChange={(e) => handleChange('imageModel', e.target.value)}
                                        onFocus={() => {
                                            if (pollinationsModels.length === 0) fetchPollinationsModels();
                                        }}
                                    >
                                        {!settings.imageModel && <option value="">{loadingPollinationsModels ? t('common.loading') : t('pipeline.model.default')}</option>}
                                        {settings.imageModel && !pollinationsModels.includes(settings.imageModel) && (
                                            <option value={settings.imageModel}>{settings.imageModel}</option>
                                        )}
                                        {pollinationsModels.map(m => (
                                            <option key={m} value={m}>{m}</option>
                                        ))}
                                    </select>
                                    <button
                                        className="premium-btn-sm"
                                        style={{ padding: '0 10px', height: '32px', minWidth: 'auto', display: 'flex', alignItems: 'center', justifyContent: 'center' }}
                                        onClick={() => fetchPollinationsModels()}
                                        disabled={loadingPollinationsModels}
                                    >
                                        <svg
                                            className={loadingPollinationsModels ? 'animate-spin' : ''}
                                            xmlns="http://www.w3.org/2000/svg"
                                            width="14" height="14"
                                            viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"
                                        >
                                            <path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.85.83 6.72 2.24" />
                                            <polyline points="21 3 21 9 15 9" />
                                        </svg>
                                    </button>
                                </div>
                            </div>

                            <div className="settings-row" style={{ gap: '16px' }}>
                                <div className="settings-control" style={{ flex: 1 }}>
                                    <label className="settings-label">{t('pipeline.image.width')}</label>
                                    <input
                                        type="number"
                                        className="settings-input"
                                        value={settings.imageWidth || 1920}
                                        onChange={(e) => handleChange('imageWidth', parseInt(e.target.value))}
                                    />
                                </div>
                                <div className="settings-control" style={{ flex: 1 }}>
                                    <label className="settings-label">{t('pipeline.image.height')}</label>
                                    <input
                                        type="number"
                                        className="settings-input"
                                        value={settings.imageHeight || 1080}
                                        onChange={(e) => handleChange('imageHeight', parseInt(e.target.value))}
                                    />
                                </div>
                            </div>

                            <div className="settings-control">
                                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                                    <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.image.nologo')}</label>
                                    <label className="stage-switch small">
                                        <input
                                            type="checkbox"
                                            checked={settings.imageNoLogo}
                                            onChange={(e) => handleChange('imageNoLogo', e.target.checked)}
                                        />
                                        <span className="stage-slider"></span>
                                    </label>
                                </div>
                            </div>

                            <div className="settings-control">
                                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                                    <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.image.enhance')}</label>
                                    <label className="stage-switch small">
                                        <input
                                            type="checkbox"
                                            checked={settings.imageEnhance}
                                            onChange={(e) => handleChange('imageEnhance', e.target.checked)}
                                        />
                                        <span className="stage-slider"></span>
                                    </label>
                                </div>
                            </div>
                        </>
                    )}

                    {settings.imageService === 'googler' && (
                        <>
                            <div className="settings-control">
                                <label className="settings-label">{t('pipeline.image.model')}</label>
                                <select
                                    className="settings-select"
                                    value={settings.imageGooglerModel || 'flow'}
                                    onChange={(e) => handleChange('imageGooglerModel', e.target.value)}
                                >
                                    <option value="flow">Flow (v4)</option>
                                    <option value="flow_gempix2">Flow Nano Pro (v4)</option>
                                    <option value="flow_imagen4">Flow Imagen 4 (v4)</option>
                                    <option value="flow_narwhal">Flow Nano Banana 2 (v4)</option>
                                    <option value="grok">Grok (v4)</option>
                                    <option value="gemini">Gemini (v4)</option>
                                    <option value="flower">Flower / Veo 3.1 (v4)</option>
                                    <option value="openai">OpenAI / ChatGPT (v4)</option>
                                </select>
                            </div>

                            <div className="settings-control">
                                <label className="settings-label">{t('pipeline.image.aspect_ratio') || 'Співвідношення сторін'}</label>
                                <select
                                    className="settings-select"
                                    value={settings.imageGooglerAspectRatio || 'IMAGE_ASPECT_RATIO_LANDSCAPE'}
                                    onChange={(e) => handleChange('imageGooglerAspectRatio', e.target.value)}
                                >
                                    <option value="IMAGE_ASPECT_RATIO_PORTRAIT">{t('pipeline.image.aspect_ratio_portrait') || 'Портрет (9:16)'}</option>
                                    <option value="IMAGE_ASPECT_RATIO_LANDSCAPE">{t('pipeline.image.aspect_ratio_landscape') || 'Ландшафт (16:9)'}</option>
                                </select>
                            </div>

                            <>
                                    <div className="settings-control">
                                        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                                            <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.image.googler.remix_enabled')}</label>
                                            <label className="stage-switch small">
                                                <input
                                                    type="checkbox"
                                                    checked={settings.imageGooglerRemixEnabled || false}
                                                    onChange={(e) => handleChange('imageGooglerRemixEnabled', e.target.checked)}
                                                />
                                                <span className="stage-slider"></span>
                                            </label>
                                        </div>
                                    </div>

                                    {settings.imageGooglerRemixEnabled && (
                                        <>
                                            <div className="settings-control">
                                                <div
                                                    onClick={async () => {
                                                        try {
                                                            const path = await (window as any).go.main.App.SelectImage();
                                                            if (path) {
                                                                handleChange('imageGooglerReferenceImage', path);
                                                            }
                                                        } catch (err) {
                                                            console.error(err);
                                                        }
                                                    }}
                                                    style={{
                                                        width: '100%',
                                                        padding: '16px',
                                                        borderRadius: '12px',
                                                        border: settings.imageGooglerReferenceImage ? '1px solid var(--accent-color)' : '1px dashed var(--bg-tertiary)',
                                                        backgroundColor: settings.imageGooglerReferenceImage ? 'rgba(var(--accent-rgb), 0.05)' : 'var(--bg-secondary)',
                                                        backgroundImage: previewUrl ? `url(${previewUrl})` : 'none',
                                                        backgroundSize: 'contain',
                                                        backgroundRepeat: 'no-repeat',
                                                        backgroundPosition: 'center',
                                                        display: 'flex',
                                                        flexDirection: 'column',
                                                        alignItems: 'center',
                                                        justifyContent: 'center',
                                                        gap: '8px',
                                                        cursor: 'pointer',
                                                        transition: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
                                                        position: 'relative',
                                                        overflow: 'hidden',
                                                        minHeight: '100px'
                                                    }}
                                                    className="image-remix-dropzone"
                                                >
                                                    {previewUrl && (
                                                        <div style={{
                                                            position: 'absolute',
                                                            inset: 0,
                                                            backgroundColor: 'rgba(0,0,0,0.4)',
                                                            zIndex: 1
                                                        }} />
                                                    )}

                                                    <div style={{
                                                        fontSize: '24px',
                                                        opacity: settings.imageGooglerReferenceImage ? 1 : 0.5,
                                                        filter: settings.imageGooglerReferenceImage ? 'drop-shadow(0 0 8px var(--accent-color))' : 'none',
                                                        position: 'relative',
                                                        zIndex: 2
                                                    }}>
                                                        {settings.imageGooglerReferenceImage ? '🖼️' : '📁'}
                                                    </div>
                                                    <div style={{
                                                        fontSize: '11px',
                                                        fontWeight: '600',
                                                        color: settings.imageGooglerReferenceImage ? '#fff' : 'var(--text-secondary)',
                                                        textAlign: 'center',
                                                        position: 'relative',
                                                        zIndex: 2,
                                                        textShadow: previewUrl ? '0 1px 4px rgba(0,0,0,0.8)' : 'none'
                                                    }}>
                                                        {settings.imageGooglerReferenceImage
                                                            ? t('pipeline.image.googler.remix_change')
                                                            : t('pipeline.image.googler.remix_select')}
                                                    </div>
                                                    {settings.imageGooglerReferenceImage && (
                                                        <div style={{
                                                            fontSize: '9px',
                                                            color: '#ddd',
                                                            maxWidth: '100%',
                                                            overflow: 'hidden',
                                                            textOverflow: 'ellipsis',
                                                            whiteSpace: 'nowrap',
                                                            opacity: 0.9,
                                                            position: 'relative',
                                                            zIndex: 2,
                                                            textShadow: '0 1px 2px rgba(0,0,0,0.8)'
                                                        }}>
                                                            {settings.imageGooglerReferenceImage.split(/[\\/]/).pop()}
                                                        </div>
                                                    )}

                                                    {!settings.imageGooglerReferenceImage && (
                                                        <div style={{
                                                            position: 'absolute',
                                                            bottom: '4px',
                                                            fontSize: '8px',
                                                            color: 'var(--text-tertiary)',
                                                            opacity: 0.3
                                                        }}>
                                                            JPG, PNG, WEBP
                                                        </div>
                                                    )}
                                                </div>
                                            </div>

                                            <div className="settings-control">
                                                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                                                    <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.image.googler.strict_mode')}</label>
                                                    <label className="stage-switch small">
                                                        <input
                                                            type="checkbox"
                                                            checked={settings.imageGooglerRemixStrictMode || false}
                                                            onChange={(e) => handleChange('imageGooglerRemixStrictMode', e.target.checked)}
                                                        />
                                                        <span className="stage-slider"></span>
                                                    </label>
                                                </div>
                                            </div>
                                        </>
                                    )}
                            </>

                            <div className="settings-control" style={{ marginTop: '16px', paddingTop: '16px', borderTop: '1px solid var(--border-color)' }}>
                                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                                    <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.image.googler.video_enabled') || 'Анімація картинок'}</label>
                                    <label className="stage-switch small">
                                        <input
                                            type="checkbox"
                                            checked={settings.imageGooglerVideoEnabled || false}
                                            onChange={(e) => handleChange('imageGooglerVideoEnabled', e.target.checked)}
                                        />
                                        <span className="stage-slider"></span>
                                    </label>
                                </div>
                            </div>

                            {settings.imageGooglerVideoEnabled && (
                                <>
                                    <div className="settings-control">
                                        <label className="settings-label">{t('pipeline.image.googler.video_model') || 'Модель відео'}</label>
                                        <select
                                            className="settings-select"
                                            value={settings.imageGooglerVideoModel || 'flower'}
                                            onChange={(e) => handleChange('imageGooglerVideoModel', e.target.value)}
                                        >
                                            <option value="flower">Flower / Veo 3.1</option>
                                            <option value="flow">Flow</option>
                                            <option value="grok">Grok</option>
                                            <option value="gemini">Gemini</option>
                                            <option value="flower">Flower / Veo 3.1</option>
                                        </select>
                                    </div>
                                    <div className="settings-control">
                                        <label className="settings-label">{t('pipeline.image.googler.video_mode') || 'Джерело анімації'}</label>
                                        <select
                                            className="settings-select"
                                            value={settings.imageGooglerVideoMode || 'text'}
                                            onChange={(e) => handleChange('imageGooglerVideoMode', e.target.value)}
                                        >
                                            <option value="text">{t('pipeline.image.googler.video_mode_text') || 'З тексту (промту)'}</option>
                                            <option value="image">{t('pipeline.image.googler.video_mode_image') || 'З згенерованого зображення'}</option>
                                        </select>
                                    </div>
                                    <div className="settings-control">
                                        <label className="settings-label">{t('pipeline.image.googler.video_count') || 'Кількість відео'}</label>
                                        <input
                                            type="number"
                                            className="settings-input"
                                            value={settings.imageGooglerVideoCount ?? 1}
                                            onChange={(e) => handleChange('imageGooglerVideoCount', parseInt(e.target.value))}
                                            min="1"
                                        />
                                    </div>
                                    <div className="settings-control">
                                        <label className="settings-label">{t('pipeline.image.googler.fill_mode') || 'Заповнення дефіциту часу'}</label>
                                        <select
                                            className="settings-select"
                                            value={settings.imageShortVideoFillMode || 'boomerang'}
                                            onChange={(e) => handleChange('imageShortVideoFillMode', e.target.value)}
                                        >
                                            <option value="boomerang">{t('pipeline.image.googler.fill_mode_boomerang') || 'Бумеранг'}</option>
                                            <option value="mirror">{t('pipeline.image.googler.fill_mode_mirror') || 'Дзеркало'}</option>
                                        </select>
                                    </div>
                                    {settings.imageGooglerVideoModel === 'grok' && (
                                        <div className="settings-control">
                                            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                                                <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.image.googler.video_upscale') || 'Upscale відео (Grok)'}</label>
                                                <label className="stage-switch small">
                                                    <input
                                                        type="checkbox"
                                                        checked={settings.imageGooglerVideoUpscale || false}
                                                        onChange={(e) => handleChange('imageGooglerVideoUpscale', e.target.checked)}
                                                    />
                                                    <span className="stage-slider"></span>
                                                </label>
                                            </div>
                                        </div>
                                    )}
                                </>
                            )}
                        </>
                    )}

                    {settings.imageService === 'elevenlabsimage' && (
                        <>
                            <div className="settings-control">
                                <label className="settings-label">{t('pipeline.image.elevenlabsimage.aspect_ratio') || 'Співвідношення сторін'}</label>
                                <select
                                    className="settings-select"
                                    value={settings.elevenLabsImageAspectRatio || '16:9'}
                                    onChange={(e) => handleChange('elevenLabsImageAspectRatio', e.target.value)}
                                >
                                    <option value="16:9">16:9</option>
                                    <option value="9:16">9:16</option>
                                </select>
                            </div>
                        </>
                    )}
                </div>
            </div>
        </div >
    );
};
