import React from 'react';
import { useI18n } from '../../contexts/I18nContext';
import SearchableSelect from '../SearchableSelect';

interface VoiceoverSectionProps {
    settings: any;
    handleChange: (field: string, value: any) => void;
    setSettings: React.Dispatch<React.SetStateAction<any>>;
    fetchVoiceTemplates: (keyID?: string) => void;
    fetchVoiceMakerVoices: (keyID?: string) => void;
    voiceTemplates: string[];
    voiceMakerVoices: any[];
    loadingTemplates: boolean;
}

export const VoiceoverSection: React.FC<VoiceoverSectionProps> = ({
    settings, handleChange, setSettings, fetchVoiceTemplates, fetchVoiceMakerVoices, voiceTemplates, voiceMakerVoices, loadingTemplates
}) => {
    const { t } = useI18n();

    return (
        <div className={`pipeline-stage-container ${settings.voiceoverCollapsed || !settings.voiceoverEnabled ? 'is-collapsed' : ''}`}>
            <div
                className="pipeline-stage-header"
                onClick={() => handleChange('voiceoverCollapsed', !settings.voiceoverCollapsed)}
            >
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flex: 1 }}>
                    <svg
                        className={`stage-chevron ${settings.voiceoverCollapsed || !settings.voiceoverEnabled ? 'rotated' : ''}`}
                        xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"
                    >
                        <path d="m6 9 6 6 6-6" />
                    </svg>
                    <div style={{ display: 'flex', flexDirection: 'column' }}>
                        <span className="pipeline-stage-title">{t('pipeline.stage.voiceover')}</span>
                        <span className="stage-status-text">
                            {settings.voiceoverEnabled ? t('pipeline.stage.enabled') : t('pipeline.stage.disabled_simple')}
                        </span>
                    </div>
                </div>
                <label className="stage-switch" onClick={(e) => e.stopPropagation()}>
                    <input
                        type="checkbox"
                        checked={settings.voiceoverEnabled}
                        onChange={(e) => {
                            const val = e.target.checked;
                            setSettings((prev: any) => ({
                                ...prev,
                                voiceoverEnabled: val,
                                voiceoverCollapsed: !val ? true : prev.voiceoverCollapsed
                            }));
                        }}
                    />
                    <span className="stage-slider"></span>
                </label>
            </div>

            <div className={`stage-settings-content ${settings.voiceoverCollapsed || !settings.voiceoverEnabled ? 'collapsed' : ''}`}>
                <div className="settings-group">
                    <div className="settings-control">
                        <label className="settings-label">{t('pipeline.voiceover.service') || 'Сервіс озвучки'}</label>
                        <select
                            className="settings-select"
                            value={settings.voiceoverService}
                            onChange={(e) => {
                                const val = e.target.value;
                                handleChange('voiceoverService', val);
                                if (val === 'elevenlabsbot') {
                                    fetchVoiceTemplates();
                                } else if (val === 'voicemaker') {
                                    fetchVoiceMakerVoices();
                                }
                            }}
                        >
                            <option value="elevenlabsbot">{t('pipeline.voiceover.services.elevenlabsbot') || 'ElevenLabs Bot'}</option>
                            <option value="elevenlabsunlim">{t('pipeline.voiceover.services.elevenlabsunlim') || 'ElevenLabs Unlim'}</option>
                            <option value="elevenlabsua">{t('pipeline.voiceover.services.elevenlabsua') || 'ElevenLabs UA'}</option>
                            <option value="voicemaker">{t('pipeline.voiceover.services.voicemaker') || 'VoiceMaker'}</option>
                        </select>
                    </div>

                    {settings.voiceoverService === 'elevenlabsbot' && (
                        <div className="settings-control">
                            <label className="settings-label">{t('pipeline.voiceover.template') || 'Шаблон голосу'}</label>
                            <div style={{ display: 'flex', gap: '8px' }}>
                                <select
                                    className="settings-select"
                                    style={{ flex: 1 }}
                                    value={settings.voiceoverTemplate}
                                    onChange={(e) => handleChange('voiceoverTemplate', e.target.value)}
                                    disabled={loadingTemplates}
                                >
                                    <option value="">{loadingTemplates ? (t('common.loading') || 'Loading...') : (t('common.select_template') || 'Select template...')}</option>
                                    {settings.voiceoverTemplate && !voiceTemplates.includes(settings.voiceoverTemplate) && (
                                        <option value={settings.voiceoverTemplate}>{settings.voiceoverTemplate}</option>
                                    )}
                                    {voiceTemplates.map(tpl => (
                                        <option key={tpl} value={tpl}>{tpl}</option>
                                    ))}
                                </select>
                                <button
                                    className="premium-btn-sm"
                                    style={{ padding: '0 10px', height: '32px', minWidth: 'auto', display: 'flex', alignItems: 'center', justifyContent: 'center' }}
                                    onClick={() => fetchVoiceTemplates()}
                                    disabled={loadingTemplates}
                                    title={t('common.refresh') || 'Refresh'}
                                >
                                    <svg
                                        className={loadingTemplates ? 'animate-spin' : ''}
                                        xmlns="http://www.w3.org/2000/svg"
                                        width="14" height="14"
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        strokeWidth="2.5"
                                        strokeLinecap="round"
                                        strokeLinejoin="round"
                                    >
                                        <path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.85.83 6.72 2.24" />
                                        <polyline points="21 3 21 9 15 9" />
                                    </svg>
                                </button>
                            </div>
                        </div>
                    )}

                    {settings.voiceoverService === 'elevenlabsunlim' && (
                        <>
                            <div className="settings-control">
                                <label className="settings-label">Voice ID</label>
                                <input
                                    className="settings-input"
                                    value={settings.elevenLabsUnlimVoiceID || ''}
                                    onChange={(e) => handleChange('elevenLabsUnlimVoiceID', e.target.value)}
                                    placeholder="AB9XsbSA..."
                                />
                            </div>

                            <div className="settings-control">
                                <label className="settings-label">{t('pipeline.voiceover.settings.stability') || 'Stability'}</label>
                                <div className="settings-slider-container">
                                    <input
                                        type="range"
                                        className="settings-slider"
                                        min="0"
                                        max="1"
                                        step="0.01"
                                        value={settings.elevenLabsUnlimStability ?? 0.5}
                                        style={{ '--range-progress': `${(settings.elevenLabsUnlimStability ?? 0.5) * 100}%` } as React.CSSProperties}
                                        onChange={(e) => handleChange('elevenLabsUnlimStability', parseFloat(e.target.value))}
                                    />
                                    <span className="settings-slider-value">{(settings.elevenLabsUnlimStability ?? 0.5).toFixed(2)}</span>
                                </div>
                            </div>

                            <div className="settings-control">
                                <label className="settings-label">{t('pipeline.voiceover.settings.similarity') || 'Similarity'}</label>
                                <div className="settings-slider-container">
                                    <input
                                        type="range"
                                        className="settings-slider"
                                        min="0"
                                        max="1"
                                        step="0.01"
                                        value={settings.elevenLabsUnlimSimilarity ?? 0.75}
                                        style={{ '--range-progress': `${(settings.elevenLabsUnlimSimilarity ?? 0.75) * 100}%` } as React.CSSProperties}
                                        onChange={(e) => handleChange('elevenLabsUnlimSimilarity', parseFloat(e.target.value))}
                                    />
                                    <span className="settings-slider-value">{(settings.elevenLabsUnlimSimilarity ?? 0.75).toFixed(2)}</span>
                                </div>
                            </div>

                            <div className="settings-control">
                                <label className="settings-label">{t('pipeline.voiceover.settings.style') || 'Style Exaggeration'}</label>
                                <div className="settings-slider-container">
                                    <input
                                        type="range"
                                        className="settings-slider"
                                        min="0"
                                        max="1"
                                        step="0.01"
                                        value={settings.elevenLabsUnlimStyle ?? 0}
                                        style={{ '--range-progress': `${(settings.elevenLabsUnlimStyle ?? 0) * 100}%` } as React.CSSProperties}
                                        onChange={(e) => handleChange('elevenLabsUnlimStyle', parseFloat(e.target.value))}
                                    />
                                    <span className="settings-slider-value">{(settings.elevenLabsUnlimStyle ?? 0).toFixed(2)}</span>
                                </div>
                            </div>

                            <div className="settings-control">
                                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                                    <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.voiceover.settings.speaker_boost') || 'Speaker Boost'}</label>
                                    <label className="stage-switch small">
                                        <input
                                            type="checkbox"
                                            checked={settings.elevenLabsUnlimSpeakerBoost}
                                            onChange={(e) => handleChange('elevenLabsUnlimSpeakerBoost', e.target.checked)}
                                        />
                                        <span className="stage-slider"></span>
                                    </label>
                                </div>
                            </div>
                        </>
                    )}

                    {settings.voiceoverService === 'elevenlabsua' && (
                        <>
                            <div className="settings-control">
                                <label className="settings-label">Voice ID</label>
                                <input
                                    className="settings-input"
                                    value={settings.elevenLabsUAVoiceID || ''}
                                    onChange={(e) => handleChange('elevenLabsUAVoiceID', e.target.value)}
                                    placeholder="eBthAb30..."
                                />
                            </div>

                            <div className="settings-control">
                                <label className="settings-label">Model ID</label>
                                <select
                                    className="settings-select"
                                    value={settings.elevenLabsUAModel || 'eleven_multilingual_v2'}
                                    onChange={(e) => handleChange('elevenLabsUAModel', e.target.value)}
                                >
                                    <option value="eleven_multilingual_v2">Multilingual v2</option>
                                    <option value="eleven_flash_v2_5">Flash v2.5</option>
                                    <option value="eleven_turbo_v2_5">Turbo v2.5</option>
                                    <option value="eleven_multilingual_v3">v3 (Emotions)</option>
                                </select>
                            </div>

                            <div className="settings-control">
                                <label className="settings-label">{t('pipeline.voiceover.settings.stability') || 'Stability'}</label>
                                <div className="settings-slider-container">
                                    <input
                                        type="range"
                                        className="settings-slider"
                                        min="0"
                                        max="1"
                                        step="0.01"
                                        value={settings.elevenLabsUAStability ?? 0.5}
                                        style={{ '--range-progress': `${(settings.elevenLabsUAStability ?? 0.5) * 100}%` } as React.CSSProperties}
                                        onChange={(e) => handleChange('elevenLabsUAStability', parseFloat(e.target.value))}
                                    />
                                    <span className="settings-slider-value">{(settings.elevenLabsUAStability ?? 0.5).toFixed(2)}</span>
                                </div>
                            </div>

                            <div className="settings-control">
                                <label className="settings-label">{t('pipeline.voiceover.settings.similarity') || 'Similarity'}</label>
                                <div className="settings-slider-container">
                                    <input
                                        type="range"
                                        className="settings-slider"
                                        min="0"
                                        max="1"
                                        step="0.01"
                                        value={settings.elevenLabsUASimilarity ?? 0.75}
                                        style={{ '--range-progress': `${(settings.elevenLabsUASimilarity ?? 0.75) * 100}%` } as React.CSSProperties}
                                        onChange={(e) => handleChange('elevenLabsUASimilarity', parseFloat(e.target.value))}
                                    />
                                    <span className="settings-slider-value">{(settings.elevenLabsUASimilarity ?? 0.75).toFixed(2)}</span>
                                </div>
                            </div>

                            <div className="settings-control">
                                <label className="settings-label">{t('pipeline.voiceover.settings.style') || 'Style Exaggeration'}</label>
                                <div className="settings-slider-container">
                                    <input
                                        type="range"
                                        className="settings-slider"
                                        min="0"
                                        max="1"
                                        step="0.01"
                                        value={settings.elevenLabsUAStyle ?? 0}
                                        style={{ '--range-progress': `${(settings.elevenLabsUAStyle ?? 0) * 100}%` } as React.CSSProperties}
                                        onChange={(e) => handleChange('elevenLabsUAStyle', parseFloat(e.target.value))}
                                    />
                                    <span className="settings-slider-value">{(settings.elevenLabsUAStyle ?? 0).toFixed(2)}</span>
                                </div>
                            </div>

                            <div className="settings-control">
                                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                                    <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.voiceover.settings.speaker_boost') || 'Speaker Boost'}</label>
                                    <label className="stage-switch small">
                                        <input
                                            type="checkbox"
                                            checked={settings.elevenLabsUASpeakerBoost}
                                            onChange={(e) => handleChange('elevenLabsUASpeakerBoost', e.target.checked)}
                                        />
                                        <span className="stage-slider"></span>
                                    </label>
                                </div>
                            </div>
                        </>
                    )}

                    {settings.voiceoverService === 'voicemaker' && (
                        <>
                            <div className="settings-control">
                                <label className="settings-label">{t('pipeline.voiceover.template') || 'Голос'}</label>
                                <div style={{ display: 'flex', gap: '8px' }}>
                                    <SearchableSelect
                                        options={(voiceMakerVoices || []).map((v: any) => ({
                                            value: v.VoiceId,
                                            label: v.VoiceWebname,
                                            subLabel: v.LanguageName
                                        }))}
                                        value={settings.voiceMakerVoiceID}
                                        onChange={(val) => {
                                            const vInfo = voiceMakerVoices.find(v => v.VoiceId === val);
                                            setSettings((prev: any) => ({
                                                ...prev,
                                                voiceMakerVoiceID: val,
                                                voiceMakerLanguageCode: vInfo?.LanguageCode || 'multi-lang'
                                            }));
                                        }}
                                        loading={loadingTemplates}
                                        placeholder="Виберіть голос..."
                                        searchPlaceholder={t('common.search') || 'Пошук...'}
                                    />
                                    <button
                                        className="premium-btn-sm"
                                        style={{ padding: '0 10px', height: '32px', minWidth: 'auto', display: 'flex', alignItems: 'center', justifyContent: 'center' }}
                                        onClick={() => fetchVoiceMakerVoices()}
                                        disabled={loadingTemplates}
                                    >
                                        <svg
                                            className={loadingTemplates ? 'animate-spin' : ''}
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

                            <div className="settings-control">
                                <label className="settings-label">Max Chars per Request</label>
                                <div className="settings-slider-container">
                                    <input
                                        type="range"
                                        className="settings-slider"
                                        min="500"
                                        max="10000"
                                        step="100"
                                        value={settings.voiceMakerCharLimit ?? 3000}
                                        style={{ '--range-progress': `${((settings.voiceMakerCharLimit ?? 3000) - 500) / 9500 * 100}%` } as React.CSSProperties}
                                        onChange={(e) => handleChange('voiceMakerCharLimit', parseInt(e.target.value))}
                                    />
                                    <span className="settings-slider-value">{settings.voiceMakerCharLimit ?? 3000}</span>
                                </div>
                            </div>
                        </>
                    )}
                </div>
            </div>
        </div>
    );
};
