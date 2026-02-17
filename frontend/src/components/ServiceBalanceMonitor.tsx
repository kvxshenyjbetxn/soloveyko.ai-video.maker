import React, { useState } from 'react';
import './ServiceBalanceMonitor.css';
import { useI18n } from '../contexts/I18nContext';
import { useServices } from '../contexts/ServiceContext';

interface ServiceBalanceMonitorProps {
    navigateTo?: (path: string) => void;
}

export const ServiceBalanceMonitor = ({ navigateTo }: ServiceBalanceMonitorProps) => {
    const { t } = useI18n();
    const {
        openRouterBalance, loadingOpenRouter, refreshOpenRouterBalance,
        elevenLabsBotBalance, loadingElevenLabsBot, refreshElevenLabsBotBalance,
        elevenLabsUnlimBalance, loadingElevenLabsUnlim, refreshElevenLabsUnlimBalance,
        voiceMakerBalance, loadingVoiceMaker, refreshVoiceMakerBalance,
        googlerUsage, loadingGoogler, refreshGooglerUsage,
        elevenLabsBotThreshold,
        elevenLabsUnlimThreshold,
        voiceMakerThreshold,
        openRouterThreshold,
        googlerVideoThreshold,
        googlerImageThreshold,
        refreshAllBalances
    } = useServices();
    const [isExpanded, setIsExpanded] = useState(false);

    const isAnyLoading = loadingOpenRouter || loadingElevenLabsBot || loadingElevenLabsUnlim || loadingVoiceMaker || loadingGoogler;

    const isGooglerVideoAlert = googlerVideoThreshold > 0 && (googlerUsage.current_usage.hourly_usage.video_generation || 0) >= googlerVideoThreshold;
    const isGooglerImageAlert = googlerImageThreshold > 0 && (googlerUsage.current_usage.hourly_usage.image_generation || 0) >= googlerImageThreshold;

    const isAnyAlertActive = (
        (elevenLabsBotBalance !== null && elevenLabsBotThreshold > 0 && elevenLabsBotBalance < elevenLabsBotThreshold) ||
        (elevenLabsUnlimBalance !== null && elevenLabsUnlimBalance !== -1 && elevenLabsUnlimThreshold > 0 && elevenLabsUnlimBalance < elevenLabsUnlimThreshold) ||
        (voiceMakerBalance !== null && voiceMakerThreshold > 0 && voiceMakerBalance < voiceMakerThreshold) ||
        (openRouterBalance !== null && openRouterThreshold > 0 && openRouterBalance < openRouterThreshold) ||
        isGooglerVideoAlert ||
        isGooglerImageAlert
    );

    const getIconColor = () => {
        if (isAnyLoading) return '#FFC107'; // Yellow

        if (isAnyAlertActive) {
            return '#ff5252'; // Red warning
        }

        if (openRouterBalance === null && elevenLabsBotBalance === null && elevenLabsUnlimBalance === null && voiceMakerBalance === null && googlerUsage.expiration_date === 0) return '#757575'; // Grey
        return '#4caf50'; // Green
    };

    return (
        <div className={`service-balance-monitor ${isExpanded ? 'expanded' : ''}`}>
            <div className="balance-monitor-container">
                {/* Panel */}
                <div className="balance-monitor-panel">
                    <div className="balance-monitor-header">
                        <h3>{t('balanceMonitor.title') || 'Баланси сервісів'}</h3>
                        <div style={{ display: 'flex', alignItems: 'center' }}>
                            <button
                                className={`refresh-all-btn ${isAnyLoading ? 'loading' : ''}`}
                                onClick={(e) => {
                                    e.stopPropagation();
                                    refreshAllBalances();
                                }}
                                disabled={isAnyLoading}
                                title={t('balanceMonitor.refreshAll') || 'Оновити все'}
                            >
                                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                                    <path d="M21 2v6h-6M3 12a9 9 0 0 1 15-6.7L21 8M3 22v-6h6M21 12a9 9 0 0 1-15 6.7L3 16" />
                                </svg>
                            </button>
                            <button className="balance-close-btn" onClick={() => setIsExpanded(false)}>&times;</button>
                        </div>
                    </div>

                    <div className="balance-list premium-scrollbar">
                        <div className="balance-item">
                            <div className="service-name">
                                <div className={`service-status-dot ${loadingOpenRouter ? 'loading' : (openRouterBalance === null ? 'error' : '')}`}></div>
                                {t('balanceMonitor.openrouter') || 'OpenRouter'}
                                {navigateTo && (
                                    <button
                                        className="service-settings-btn"
                                        onClick={() => { navigateTo('settings.api.openrouter'); setIsExpanded(false); }}
                                        title="Settings"
                                    >
                                        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>
                                    </button>
                                )}
                            </div>
                            <div className="service-balance" style={{
                                color: (openRouterBalance !== null && openRouterThreshold > 0 && openRouterBalance < openRouterThreshold) ? '#ff5252' : '#4caf50'
                            }}>
                                {loadingOpenRouter ? '...' : (openRouterBalance !== null ? `$${openRouterBalance.toFixed(4)}` : 'N/A')}
                                {openRouterBalance !== null && openRouterThreshold > 0 && openRouterBalance < openRouterThreshold && (
                                    <span style={{ fontSize: '0.8em', marginLeft: '4px', verticalAlign: 'middle' }}>⚠️</span>
                                )}
                            </div>
                        </div>

                        <div className="balance-item">
                            <div className="service-name">
                                <div className={`service-status-dot ${loadingElevenLabsBot ? 'loading' : (elevenLabsBotBalance === null ? 'error' : '')}`}></div>
                                {t('balanceMonitor.elevenlabsbot') || 'ElevenLabsBot'}
                                {navigateTo && (
                                    <button
                                        className="service-settings-btn"
                                        onClick={() => { navigateTo('settings.api.voice.elevenlabsbot'); setIsExpanded(false); }}
                                        title="Settings"
                                    >
                                        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>
                                    </button>
                                )}
                            </div>
                            <div className="service-balance" style={{
                                color: (elevenLabsBotBalance !== null && elevenLabsBotThreshold > 0 && elevenLabsBotBalance < elevenLabsBotThreshold) ? '#ff5252' : '#4caf50'
                            }}>
                                {loadingElevenLabsBot ? '...' : (elevenLabsBotBalance !== null ? elevenLabsBotBalance.toLocaleString() : 'N/A')}
                                {elevenLabsBotBalance !== null && elevenLabsBotThreshold > 0 && elevenLabsBotBalance < elevenLabsBotThreshold && (
                                    <span style={{ fontSize: '0.8em', marginLeft: '4px', verticalAlign: 'middle' }}>⚠️</span>
                                )}
                            </div>
                        </div>

                        <div className="balance-item">
                            <div className="service-name">
                                <div className={`service-status-dot ${loadingElevenLabsUnlim ? 'loading' : (elevenLabsUnlimBalance === null ? 'error' : '')}`}></div>
                                {t('balanceMonitor.elevenlabsunlim') || 'ElevenLabsUnlim'}
                                {navigateTo && (
                                    <button
                                        className="service-settings-btn"
                                        onClick={() => { navigateTo('settings.api.voice.elevenlabsunlim'); setIsExpanded(false); }}
                                        title="Settings"
                                    >
                                        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>
                                    </button>
                                )}
                            </div>
                            <div className="service-balance" style={{
                                color: (elevenLabsUnlimBalance !== null && elevenLabsUnlimBalance !== -1 && elevenLabsUnlimThreshold > 0 && elevenLabsUnlimBalance < elevenLabsUnlimThreshold) ? '#ff5252' : '#4caf50'
                            }}>
                                {loadingElevenLabsUnlim ? '...' : (elevenLabsUnlimBalance !== null ? (elevenLabsUnlimBalance === -1 ? 'Unlimited' : elevenLabsUnlimBalance.toLocaleString()) : 'N/A')}
                                {elevenLabsUnlimBalance !== null && elevenLabsUnlimBalance !== -1 && elevenLabsUnlimThreshold > 0 && elevenLabsUnlimBalance < elevenLabsUnlimThreshold && (
                                    <span style={{ fontSize: '0.8em', marginLeft: '4px', verticalAlign: 'middle' }}>⚠️</span>
                                )}
                            </div>
                        </div>

                        <div className="balance-item">
                            <div className="service-name">
                                <div className={`service-status-dot ${loadingVoiceMaker ? 'loading' : (voiceMakerBalance === null ? 'error' : '')}`}></div>
                                {t('balanceMonitor.voicemaker') || 'VoiceMaker'}
                                {navigateTo && (
                                    <button
                                        className="service-settings-btn"
                                        onClick={() => { navigateTo('settings.api.voice.voicemaker'); setIsExpanded(false); }}
                                        title="Settings"
                                    >
                                        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>
                                    </button>
                                )}
                            </div>
                            <div className="service-balance" style={{
                                color: (voiceMakerBalance !== null && voiceMakerThreshold > 0 && voiceMakerBalance < voiceMakerThreshold) ? '#ff5252' : '#4caf50'
                            }}>
                                {loadingVoiceMaker ? '...' : (voiceMakerBalance !== null ? voiceMakerBalance.toLocaleString() : 'N/A')}
                                {voiceMakerBalance !== null && voiceMakerThreshold > 0 && voiceMakerBalance < voiceMakerThreshold && (
                                    <span style={{ fontSize: '0.8em', marginLeft: '4px', verticalAlign: 'middle' }}>⚠️</span>
                                )}
                            </div>
                        </div>

                        <div className="balance-item" style={{ height: 'auto', flexDirection: 'column', alignItems: 'flex-start', gap: '4px', padding: '8px 0' }}>
                            <div className="service-name" style={{ marginBottom: '2px' }}>
                                <div className={`service-status-dot ${loadingGoogler ? 'loading' : (googlerUsage.expiration_date === 0 ? 'error' : '')}`}></div>
                                <span style={{ fontWeight: '600' }}>{t('balanceMonitor.googler') || 'Googler'}</span>
                                {navigateTo && (
                                    <button
                                        className="service-settings-btn"
                                        onClick={() => { navigateTo('settings.api.image.googler'); setIsExpanded(false); }}
                                        title="Settings"
                                    >
                                        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>
                                    </button>
                                )}
                            </div>
                            <div style={{ display: 'flex', flexDirection: 'column', gap: '2px', width: '100%', paddingLeft: '14px' }}>
                                {/* Загальна кількість зверху */}
                                <div style={{
                                    display: 'flex',
                                    justifyContent: 'space-between',
                                    fontSize: '0.75em',
                                    opacity: 0.9,
                                    color: isGooglerVideoAlert ? '#ff5252' : 'inherit',
                                    fontWeight: isGooglerVideoAlert ? '700' : 'normal'
                                }}>
                                    <span>🎬 {t('balanceMonitor.videoTotal') || 'Video'}:</span>
                                    <span>
                                        {loadingGoogler ? '...' : `${googlerUsage.current_usage.hourly_usage.video_generation || 0}/${googlerUsage.account_limits.video_gen_per_hour_limit}`}
                                        {isGooglerVideoAlert && <span style={{ marginLeft: '4px' }}>⚠️</span>}
                                    </span>
                                </div>
                                <div style={{
                                    display: 'flex',
                                    justifyContent: 'space-between',
                                    fontSize: '0.75em',
                                    opacity: 0.9,
                                    color: isGooglerImageAlert ? '#ff5252' : 'inherit',
                                    fontWeight: isGooglerImageAlert ? '700' : 'normal'
                                }}>
                                    <span>📸 {t('balanceMonitor.imageTotal') || 'Images'}:</span>
                                    <span>
                                        {loadingGoogler ? '...' : `${googlerUsage.current_usage.hourly_usage.image_generation || 0}/${googlerUsage.account_limits.img_gen_per_hour_limit}`}
                                        {isGooglerImageAlert && <span style={{ marginLeft: '4px' }}>⚠️</span>}
                                    </span>
                                </div>

                                <div style={{ height: '1px', background: 'rgba(255,255,255,0.05)', margin: '4px 0' }}></div>

                                {/* Потоки знизу */}
                                <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.7em', opacity: 0.6 }}>
                                    <span>⚙️ {t('balanceMonitor.videoThreads') || 'Video Threads'}:</span>
                                    <span style={{ color: googlerUsage.current_usage.active_threads.video_threads >= googlerUsage.account_limits.video_generation_threads_allowed && googlerUsage.account_limits.video_generation_threads_allowed > 0 ? '#ff5252' : 'inherit' }}>
                                        {loadingGoogler ? '...' : `${googlerUsage.current_usage.active_threads.video_threads}/${googlerUsage.account_limits.video_generation_threads_allowed}`}
                                    </span>
                                </div>
                                <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.7em', opacity: 0.6 }}>
                                    <span>⚙️ {t('balanceMonitor.imageThreads') || 'Image Threads'}:</span>
                                    <span>
                                        {loadingGoogler ? '...' : `${googlerUsage.current_usage.active_threads.image_threads}/${googlerUsage.account_limits.img_generation_threads_allowed}`}
                                    </span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                {/* Toggle Button */}
                <div
                    className={`balance-monitor-toggle ${isAnyAlertActive ? 'alert-active' : ''}`}
                    onClick={() => {
                        const newExpanded = !isExpanded;
                        setIsExpanded(newExpanded);
                        if (newExpanded) {
                            if (openRouterBalance === null) refreshOpenRouterBalance();
                            if (elevenLabsBotBalance === null) refreshElevenLabsBotBalance();
                            if (elevenLabsUnlimBalance === null) refreshElevenLabsUnlimBalance();
                            if (googlerUsage.expiration_date === 0) refreshGooglerUsage();
                        }
                    }}
                    title="Balance Monitor"
                    style={{ background: getIconColor() }}
                >
                    <div className="balance-icon">
                        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            <path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" />
                        </svg>
                    </div>
                </div>
            </div>
        </div>
    );
};
