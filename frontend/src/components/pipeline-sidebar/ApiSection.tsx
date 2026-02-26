import React from 'react';
import { useI18n } from '../../contexts/I18nContext';

interface ApiSectionProps {
    type: string;
    settings: any;
    handleChange: (field: string, value: any) => void;
    openRouterKeys: any[];
    elevenLabsBotKeys: any[];
    elevenLabsUnlimKeys: any[];
    elevenLabsUAKeys: any[];
    voiceMakerKeys: any[];
    pollinationsKeys: any[];
    elevenLabsImageKeys: any[];
    fetchVoiceTemplates: (keyID?: string) => void;
    fetchVoiceMakerVoices: (keyID?: string) => void;
    setCurrentPath?: (path: string) => void;
}

const ApiIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3" />
        <circle cx="7.5" cy="16.5" r=".5" fill="currentColor" />
    </svg>
);

export const ApiSection: React.FC<ApiSectionProps> = ({
    type, settings, handleChange, openRouterKeys, elevenLabsBotKeys, elevenLabsUnlimKeys, elevenLabsUAKeys, voiceMakerKeys, pollinationsKeys, elevenLabsImageKeys, fetchVoiceTemplates, fetchVoiceMakerVoices, setCurrentPath
}) => {
    const { t } = useI18n();
    const isTranslate = type === 'translate';
    const isRewrite = type === 'rewrite';

    const selectedApiKeyID = isTranslate ? settings.translateOpenRouterKeyID : (isRewrite ? settings.rewriteOpenRouterKeyID : '');
    const selectedElevenLabsBotKeyID = isTranslate ? settings.translateElevenLabsBotKeyID : (isRewrite ? settings.rewriteElevenLabsBotKeyID : settings.voiceoverElevenLabsBotKeyID);

    return (
        <div className={`pipeline-stage-container ${settings.apiCollapsed ? 'is-collapsed' : ''}`}>
            <div
                className="pipeline-stage-header"
                onClick={() => handleChange('apiCollapsed', !settings.apiCollapsed)}
            >
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flex: 1 }}>
                    <svg
                        className={`stage-chevron ${settings.apiCollapsed ? 'rotated' : ''}`}
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
                        <ApiIcon />
                    </div>
                    <div style={{ display: 'flex', flexDirection: 'column' }}>
                        <span className="pipeline-stage-title">{t('pipeline.group.api')}</span>
                    </div>
                </div>
            </div>

            <div className={`stage-settings-content ${settings.apiCollapsed ? 'collapsed' : ''}`}>
                <div className="settings-group">
                    <div className="settings-control">
                        <label className="settings-label">{t('settings.api_keys.openrouter')}</label>
                        <select
                            className="settings-select"
                            value={selectedApiKeyID}
                            onChange={(e) => {
                                const val = e.target.value;
                                if (val === "MANAGE_KEYS") {
                                    if (setCurrentPath) setCurrentPath('settings.api.openrouter');
                                    return;
                                }
                                let field = '';
                                if (isTranslate) field = 'translateOpenRouterKeyID';
                                else if (isRewrite) field = 'rewriteOpenRouterKeyID';
                                if (field) handleChange(field, val);
                            }}
                        >
                            {openRouterKeys.length === 0 ? (
                                <option value="">{t('api.openrouterSettings.noKeys')}</option>
                            ) : (
                                openRouterKeys.map(k => <option key={k.id} value={k.id}>{k.name}</option>)
                            )}
                            <option value="MANAGE_KEYS" style={{ color: 'var(--accent-primary)', fontWeight: 'bold' }}>
                                ⚙️ {t('tabs.settings')}
                            </option>
                        </select>
                    </div>

                    <div className="settings-control">
                        <label className="settings-label">{t('settings.api_keys.elevenlabs_bot')}</label>
                        <select
                            className="settings-select"
                            value={selectedElevenLabsBotKeyID}
                            onChange={(e) => {
                                const val = e.target.value;
                                if (val === "MANAGE_KEYS") {
                                    if (setCurrentPath) setCurrentPath('settings.api.voice.elevenlabsbot');
                                    return;
                                }
                                let field = '';
                                if (isTranslate) field = 'translateElevenLabsBotKeyID';
                                else if (isRewrite) field = 'rewriteElevenLabsBotKeyID';
                                else field = 'voiceoverElevenLabsBotKeyID';
                                handleChange(field, val);
                                if (val !== "MANAGE_KEYS" && settings.voiceoverService === 'elevenlabsbot') {
                                    fetchVoiceTemplates(val);
                                }
                            }}
                        >
                            {elevenLabsBotKeys.length === 0 ? (
                                <option value="">{t('api.openrouterSettings.noKeys')}</option>
                            ) : (
                                elevenLabsBotKeys.map(k => <option key={k.id} value={k.id}>{k.name}</option>)
                            )}
                            <option value="MANAGE_KEYS" style={{ color: 'var(--accent-primary)', fontWeight: 'bold' }}>
                                ⚙️ {t('tabs.settings')}
                            </option>
                        </select>
                    </div>

                    <div className="settings-control">
                        <label className="settings-label">{t('settings.api_keys.elevenlabs_unlim')}</label>
                        <select
                            className="settings-select"
                            value={settings.voiceoverElevenLabsUnlimKeyID}
                            onChange={(e) => {
                                const val = e.target.value;
                                if (val === "MANAGE_KEYS") {
                                    if (setCurrentPath) setCurrentPath('settings.api.voice.elevenlabsunlim');
                                    return;
                                }
                                handleChange('voiceoverElevenLabsUnlimKeyID', val);
                            }}
                        >
                            {elevenLabsUnlimKeys.length === 0 ? (
                                <option value="">{t('api.openrouterSettings.noKeys')}</option>
                            ) : (
                                elevenLabsUnlimKeys.map(k => <option key={k.id} value={k.id}>{k.name}</option>)
                            )}
                            <option value="MANAGE_KEYS" style={{ color: 'var(--accent-primary)', fontWeight: 'bold' }}>
                                ⚙️ {t('tabs.settings')}
                            </option>
                        </select>
                    </div>

                    <div className="settings-control">
                        <label className="settings-label">{t('settings.api_keys.elevenlabs_ua')}</label>
                        <select
                            className="settings-select"
                            value={settings.voiceoverElevenLabsUAKeyID}
                            onChange={(e) => {
                                const val = e.target.value;
                                if (val === "MANAGE_KEYS") {
                                    if (setCurrentPath) setCurrentPath('settings.api.voice.elevenlabsua');
                                    return;
                                }
                                handleChange('voiceoverElevenLabsUAKeyID', val);
                            }}
                        >
                            {elevenLabsUAKeys.length === 0 ? (
                                <option value="">{t('api.openrouterSettings.noKeys')}</option>
                            ) : (
                                elevenLabsUAKeys.map(k => <option key={k.id} value={k.id}>{k.name}</option>)
                            )}
                            <option value="MANAGE_KEYS" style={{ color: 'var(--accent-primary)', fontWeight: 'bold' }}>
                                ⚙️ {t('tabs.settings')}
                            </option>
                        </select>
                    </div>

                    <div className="settings-control">
                        <label className="settings-label">{t('settings.api_keys.voicemaker')}</label>
                        <select
                            className="settings-select"
                            value={settings.voiceoverVoiceMakerKeyID}
                            onChange={(e) => {
                                const val = e.target.value;
                                if (val === "MANAGE_KEYS") {
                                    if (setCurrentPath) setCurrentPath('settings.api.voice.voicemaker');
                                    return;
                                }
                                handleChange('voiceoverVoiceMakerKeyID', val);
                                if (settings.voiceoverService === 'voicemaker') {
                                    fetchVoiceMakerVoices(val);
                                }
                            }}
                        >
                            {voiceMakerKeys.length === 0 ? (
                                <option value="">{t('api.openrouterSettings.noKeys')}</option>
                            ) : (
                                voiceMakerKeys.map(k => <option key={k.id} value={k.id}>{k.name}</option>)
                            )}
                            <option value="MANAGE_KEYS" style={{ color: 'var(--accent-primary)', fontWeight: 'bold' }}>
                                ⚙️ {t('tabs.settings')}
                            </option>
                        </select>
                    </div>

                    <div className="settings-control">
                        <label className="settings-label">{t('settings.api_keys.pollinations')}</label>
                        <select
                            className="settings-select"
                            value={settings.imagePollinationsKeyID}
                            onChange={(e) => {
                                const val = e.target.value;
                                if (val === "MANAGE_KEYS") {
                                    if (setCurrentPath) setCurrentPath('settings.api.image.pollinationsai');
                                    return;
                                }
                                handleChange('imagePollinationsKeyID', val);
                            }}
                        >
                            {pollinationsKeys.length === 0 ? (
                                <option value="">{t('api.openrouterSettings.noKeys')}</option>
                            ) : (
                                pollinationsKeys.map(k => <option key={k.id} value={k.id}>{k.name}</option>)
                            )}
                            <option value="MANAGE_KEYS" style={{ color: 'var(--accent-primary)', fontWeight: 'bold' }}>
                                ⚙️ {t('tabs.settings')}
                            </option>
                        </select>
                    </div>

                    <div className="settings-control">
                        <label className="settings-label">{t('settings.api_keys.elevenlabs_image')}</label>
                        <select
                            className="settings-select"
                            value={settings.elevenLabsImageKeyID || ''}
                            onChange={(e) => {
                                const val = e.target.value;
                                if (val === "MANAGE_KEYS") {
                                    if (setCurrentPath) setCurrentPath('settings.api.image.elevenlabsimage');
                                    return;
                                }
                                handleChange('elevenLabsImageKeyID', val);
                            }}
                        >
                            {elevenLabsImageKeys.length === 0 ? (
                                <option value="">{t('api.openrouterSettings.noKeys')}</option>
                            ) : (
                                elevenLabsImageKeys.map(k => <option key={k.id} value={k.id}>{k.name}</option>)
                            )}
                            <option value="MANAGE_KEYS" style={{ color: 'var(--accent-primary)', fontWeight: 'bold' }}>
                                ⚙️ {t('tabs.settings')}
                            </option>
                        </select>
                    </div>
                </div>
            </div>
        </div>
    );
};
