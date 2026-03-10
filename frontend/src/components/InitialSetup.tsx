import React, { useState, useEffect } from 'react';
import './InitialSetup.css';
import { useI18n } from '../contexts/I18nContext';
import { useTheme } from '../contexts/ThemeContext';
import { ConfirmModal } from './ConfirmModal';
// @ts-ignore
import logo from '../assets/logo.png';

interface InitialSetupProps {
    onFinish: () => void;
}

export const InitialSetup: React.FC<InitialSetupProps> = ({ onFinish }) => {
    const { t, locale, setLocale } = useI18n();
    const { theme, setTheme, accentColor, setAccentColor } = useTheme();
    const [step, setStep] = useState(1);
    const [apiKey, setApiKey] = useState('');
    const [whisperEngine, setWhisperEngine] = useState('standard');
    const [montageCodec, setMontageCodec] = useState('cpu');
    const [isSaving, setIsSaving] = useState(false);

    // Download states
    const [isDownloading, setIsDownloading] = useState(false);
    const [downloadProgress, setDownloadProgress] = useState(0);
    const [showPrompt, setShowPrompt] = useState(false);
    const [promptEngine, setPromptEngine] = useState('');

    useEffect(() => {
        // @ts-ignore
        const unsubProgress = window.runtime?.EventsOn("whisperxDownloadProgress", (progress: number) => {
            setDownloadProgress(progress);
        });
        // @ts-ignore
        const unsubAmdProgress = window.runtime?.EventsOn("amdDownloadProgress", (progress: number) => {
            setDownloadProgress(progress);
        });
        // @ts-ignore
        const unsubInstalled = window.runtime?.EventsOn("whisperxInstalled", () => {
            setIsDownloading(false);
            setWhisperEngine('whisperx');
        });
        // @ts-ignore
        const unsubAmdInstalled = window.runtime?.EventsOn("amdInstalled", () => {
            setIsDownloading(false);
            setWhisperEngine('amd');
        });

        return () => {
            if (unsubProgress) unsubProgress();
            if (unsubAmdProgress) unsubAmdProgress();
            if (unsubInstalled) unsubInstalled();
            if (unsubAmdInstalled) unsubAmdInstalled();
        };
    }, []);

    const nextStep = () => {
        if (isDownloading) return;
        setStep(prev => prev + 1);
    };
    const prevStep = () => {
        if (isDownloading) return;
        setStep(prev => prev - 1);
    };

    const handleEngineSelect = async (s: string) => {
        if (isDownloading) return;

        if (s === 'amd') {
            // @ts-ignore
            const installed = await window.go.main.App.IsAmdWhisperInstalled();
            if (!installed) {
                setPromptEngine('amd');
                setShowPrompt(true);
                return;
            }
        } else if (s === 'whisperx') {
            // @ts-ignore
            const installed = await window.go.main.App.IsWhisperXInstalled();
            if (!installed) {
                setPromptEngine('whisperx');
                setShowPrompt(true);
                return;
            }
        }
        setWhisperEngine(s);
    };

    const startDownload = async () => {
        setShowPrompt(false);
        setIsDownloading(true);
        setDownloadProgress(0);
        try {
            if (promptEngine === 'amd') {
                // @ts-ignore
                await window.go.main.App.InstallAmdWhisper();
            } else if (promptEngine === 'whisperx') {
                // @ts-ignore
                await window.go.main.App.DownloadWhisperX();
            }
        } catch (err) {
            console.error('Failed to start download:', err);
            setIsDownloading(false);
        }
    };

    const handleFinish = async () => {
        setIsSaving(true);
        try {
            // Save API Key if provided
            if (apiKey.trim()) {
                // @ts-ignore
                await window.go.main.App.SaveOpenRouterAPIKey(apiKey.trim());
            }

            // Save Whisper Engine
            // @ts-ignore
            await window.go.main.App.SetGeneralWhisperEngine(whisperEngine);

            // Save Montage Codec
            // @ts-ignore
            await window.go.main.App.SetGeneralMontageCodec(montageCodec);

            // Mark initial setup as finished (mandatory)
            // @ts-ignore
            await window.go.main.App.SetFirstRun(false);

            onFinish();
        } catch (error) {
            console.error('Failed to finish initial setup:', error);
        } finally {
            setIsSaving(false);
        }
    };

    const accentPresets = ['#ff00c3', '#0078d4', '#ff4500', '#32cd32', '#9370db', '#ffd700', '#ffffff'];

    const renderStep = () => {
        switch (step) {
            case 1: // Language
                return (
                    <div className="wizard-step animate-slide" key={step}>
                        <h2>{t('initial_setup.step_language')}</h2>
                        <p className="wizard-desc">{t('initial_setup.language_desc')}</p>
                        <div className="wizard-options">
                            <div
                                className={`wizard-option-card ${locale === 'uk' ? 'active' : ''}`}
                                onClick={() => setLocale('uk')}
                            >
                                <span className="option-title">Українська</span>
                            </div>
                            <div
                                className={`wizard-option-card ${locale === 'en' ? 'active' : ''}`}
                                onClick={() => setLocale('en')}
                            >
                                <span className="option-title">English</span>
                            </div>
                            <div
                                className={`wizard-option-card ${locale === 'ru' ? 'active' : ''}`}
                                onClick={() => setLocale('ru')}
                            >
                                <span className="option-title">Русский</span>
                            </div>
                        </div>
                        <div className="wizard-actions">
                            <button className="wizard-btn-primary" onClick={nextStep}>{t('common.ready')}</button>
                        </div>
                    </div>
                );
            case 2: // Theme & Accent
                return (
                    <div className="wizard-step animate-slide" key={step}>
                        <h2>{t('initial_setup.step_theme')}</h2>
                        <p className="wizard-desc">{t('initial_setup.theme_desc')}</p>

                        <div className="wizard-theme-selector">
                            <div
                                className={`wizard-theme-card dark ${theme === 'dark' ? 'active' : ''}`}
                                onClick={() => setTheme('dark')}
                            >
                                <div className="wizard-theme-preview" style={{ backgroundColor: '#1e1e1e' }}></div>
                                <span>{t('general.themeDark')}</span>
                            </div>
                            <div
                                className={`wizard-theme-card amoled ${theme === 'amoled' ? 'active' : ''}`}
                                onClick={() => setTheme('amoled')}
                            >
                                <div className="wizard-theme-preview" style={{ backgroundColor: '#000000' }}></div>
                                <span>{t('general.themeAmoled')}</span>
                            </div>
                        </div>

                        <div className="wizard-accent-selector">
                            <div className="wizard-accent-palette">
                                {accentPresets.map(color => (
                                    <div
                                        key={color}
                                        className={`wizard-accent-dot ${accentColor === color ? 'active' : ''}`}
                                        style={{ backgroundColor: color }}
                                        onClick={() => setAccentColor(color)}
                                    />
                                ))}
                            </div>
                        </div>

                        <div className="wizard-actions">
                            <button className="wizard-btn-secondary" onClick={prevStep}>{t('common.cancel')}</button>
                            <button className="wizard-btn-primary" onClick={nextStep}>{t('common.ready')}</button>
                        </div>
                    </div>
                );
            case 3: // API Key (Optional)
                return (
                    <div className="wizard-step animate-slide" key={step}>
                        <h2>{t('initial_setup.step_api')}</h2>
                        <p className="wizard-desc">{t('initial_setup.api_desc')}</p>
                        <div className="wizard-input-group">
                            <input
                                type="password"
                                className="wizard-input"
                                placeholder="sk-or-v1-..."
                                value={apiKey}
                                onChange={(e) => setApiKey(e.target.value)}
                            />
                        </div>
                        <div className="wizard-actions">
                            <button className="wizard-btn-secondary" onClick={apiKey ? () => setApiKey('') : prevStep}>
                                {apiKey ? t('common.clear') : t('common.cancel')}
                            </button>
                            <button className="wizard-btn-primary" onClick={nextStep}>
                                {apiKey ? t('initial_setup.finish') : t('initial_setup.skip')}
                            </button>
                        </div>
                    </div>
                );
            case 4: // Whisper Engine
                return (
                    <div className="wizard-step animate-slide" key={step}>
                        <h2>{t('initial_setup.step_whisper')}</h2>
                        <p className="wizard-desc">{t('initial_setup.whisper_desc')}</p>
                        <div className="wizard-options">
                            <div
                                className={`wizard-option-card whisper ${whisperEngine === 'standard' ? 'active' : ''}`}
                                onClick={() => handleEngineSelect('standard')}
                            >
                                <div className="option-info">
                                    <span className="option-title">{t('initial_setup.whisper_standard_title')}</span>
                                    <span className="option-desc">{t('initial_setup.whisper_standard_desc')}</span>
                                </div>
                            </div>
                            <div
                                className={`wizard-option-card whisper ${whisperEngine === 'amd' ? 'active' : ''}`}
                                onClick={() => handleEngineSelect('amd')}
                            >
                                <div className="option-info">
                                    <span className="option-title">{t('initial_setup.whisper_amd_title')}</span>
                                    <span className="option-desc">{t('initial_setup.whisper_amd_desc')}</span>
                                </div>
                            </div>
                            <div
                                className={`wizard-option-card whisper ${whisperEngine === 'whisperx' ? 'active' : ''}`}
                                onClick={() => handleEngineSelect('whisperx')}
                            >
                                <div className="option-info">
                                    <span className="option-title">{t('initial_setup.whisper_x_title')}</span>
                                    <span className="option-desc">{t('initial_setup.whisper_x_desc')}</span>
                                </div>
                            </div>
                            <div
                                className={`wizard-option-card whisper ${whisperEngine === 'assemblyai' ? 'active' : ''}`}
                                onClick={() => handleEngineSelect('assemblyai')}
                            >
                                <div className="option-info">
                                    <span className="option-title">{t('initial_setup.whisper_assembly_title')}</span>
                                    <span className="option-desc">{t('initial_setup.whisper_assembly_desc')}</span>
                                </div>
                            </div>
                        </div>

                        {isDownloading && (
                            <div className="download-progress-container" style={{ marginTop: '24px', marginBottom: '24px', width: '100%' }}>
                                <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '8px', fontSize: '13px' }}>
                                    <span style={{ color: 'var(--accent-primary)', fontWeight: 600 }}>
                                        {promptEngine === 'amd' ? t('performanceTab.amd_downloading') : t('performanceTab.whisperx_downloading').replace('{{progress}}', downloadProgress.toString())}
                                    </span>
                                    <span style={{ color: 'var(--text-secondary)' }}>{downloadProgress}%</span>
                                </div>
                                <div style={{ height: '8px', background: 'rgba(255,255,255,0.05)', borderRadius: '4px', overflow: 'hidden', border: '1px solid rgba(255,255,255,0.1)' }}>
                                    <div style={{
                                        height: '100%',
                                        width: `${downloadProgress}%`,
                                        background: 'var(--accent-primary)',
                                        boxShadow: '0 0 10px var(--accent-primary)',
                                        transition: 'width 0.3s ease'
                                    }} />
                                </div>
                            </div>
                        )}

                        <div className="wizard-actions">
                            <button className="wizard-btn-secondary" onClick={prevStep} disabled={isDownloading}>{t('common.cancel')}</button>
                            <button className="wizard-btn-primary" onClick={nextStep} disabled={isDownloading}>
                                {isDownloading ? t('common.loading') : t('common.ready')}
                            </button>
                        </div>
                    </div>
                );
            case 5: // Montage Codec
                return (
                    <div className="wizard-step animate-slide" key={step}>
                        <h2>{t('initial_setup.step_montage')}</h2>
                        <p className="wizard-desc">{t('initial_setup.montage_desc')}</p>
                        <div className="wizard-options">
                            <div
                                className={`wizard-option-card whisper ${montageCodec === 'cpu' ? 'active' : ''}`}
                                onClick={() => setMontageCodec('cpu')}
                            >
                                <div className="option-info">
                                    <span className="option-title">{t('initial_setup.montage_cpu_title')}</span>
                                    <span className="option-desc">{t('initial_setup.montage_cpu_desc')}</span>
                                </div>
                            </div>
                            <div
                                className={`wizard-option-card whisper ${montageCodec === 'nvidia' ? 'active' : ''}`}
                                onClick={() => setMontageCodec('nvidia')}
                            >
                                <div className="option-info">
                                    <span className="option-title">{t('initial_setup.montage_nvidia_title')}</span>
                                    <span className="option-desc">{t('initial_setup.montage_nvidia_desc')}</span>
                                </div>
                            </div>
                            <div
                                className={`wizard-option-card whisper ${montageCodec === 'amd' ? 'active' : ''}`}
                                onClick={() => setMontageCodec('amd')}
                            >
                                <div className="option-info">
                                    <span className="option-title">{t('initial_setup.montage_amd_title')}</span>
                                    <span className="option-desc">{t('initial_setup.montage_amd_desc')}</span>
                                </div>
                            </div>
                            <div
                                className={`wizard-option-card whisper ${montageCodec === 'apple' ? 'active' : ''}`}
                                onClick={() => setMontageCodec('apple')}
                            >
                                <div className="option-info">
                                    <span className="option-title">{t('initial_setup.montage_apple_title')}</span>
                                    <span className="option-desc">{t('initial_setup.montage_apple_desc')}</span>
                                </div>
                            </div>
                        </div>
                        <div className="wizard-actions">
                            <button className="wizard-btn-secondary" onClick={prevStep}>{t('common.cancel')}</button>
                            <button className="wizard-btn-primary" onClick={nextStep}>{t('common.ready')}</button>
                        </div>
                    </div>
                );
            case 6: // Ready
                return (
                    <div className="wizard-step animate-slide" key={step}>
                        <div className="success-icon">✨</div>
                        <h2>{t('initial_setup.ready')}</h2>
                        <p className="wizard-desc" style={{ marginBottom: '32px' }}>{t('initial_setup.ready_desc')}</p>
                        <div className="wizard-actions">
                            <button className="wizard-btn-primary" onClick={handleFinish} disabled={isSaving}>
                                {t('initial_setup.go')}
                            </button>
                        </div>
                    </div>
                );
            default:
                return null;
        }
    };

    return (
        <div className="wizard-overlay">
            <div className="wizard-container glass-panel animate-scale">
                <div className="wizard-header">
                    <img src={logo} alt="Soloveyko" className="wizard-logo" />
                    <h1>{t('initial_setup.title')}</h1>
                </div>

                <div className="wizard-progress">
                    <div className={`progress-dot ${step === 1 ? 'active' : step > 1 ? 'completed' : ''}`}></div>
                    <div className={`progress-dot ${step === 2 ? 'active' : step > 2 ? 'completed' : ''}`}></div>
                    <div className={`progress-dot ${step === 3 ? 'active' : step > 3 ? 'completed' : ''}`}></div>
                    <div className={`progress-dot ${step === 4 ? 'active' : step > 4 ? 'completed' : ''}`}></div>
                    <div className={`progress-dot ${step === 5 ? 'active' : step > 5 ? 'completed' : ''}`}></div>
                    <div className={`progress-dot ${step === 6 ? 'active' : ''}`}></div>
                </div>

                <div className="wizard-content">
                    {renderStep()}
                </div>
            </div>

            <ConfirmModal
                isOpen={showPrompt}
                onClose={() => setShowPrompt(false)}
                onConfirm={startDownload}
                title={promptEngine === 'amd' ? t('performanceTab.whisper_amd_not_found_title') : t('performanceTab.whisperx_not_found_title')}
                message={promptEngine === 'amd' ? t('performanceTab.whisper_amd_not_found_desc') : t('performanceTab.whisperx_not_found_desc')}
                confirmText={t('performanceTab.whisperx_download_btn')}
                isDanger={false}
                type="info"
            />
        </div>
    );
};
