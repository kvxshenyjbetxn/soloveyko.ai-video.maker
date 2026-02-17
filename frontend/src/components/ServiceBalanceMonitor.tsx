import React, { useState } from 'react';
import './ServiceBalanceMonitor.css';
import { useI18n } from '../contexts/I18nContext';
import { useServices } from '../contexts/ServiceContext';

export const ServiceBalanceMonitor = () => {
    const { t } = useI18n();
    const {
        openRouterBalance, loadingOpenRouter, refreshOpenRouterBalance,
        elevenLabsBotBalance, loadingElevenLabsBot, refreshElevenLabsBotBalance,
        elevenLabsUnlimBalance, loadingElevenLabsUnlim, refreshElevenLabsUnlimBalance,
        voiceMakerBalance, loadingVoiceMaker, refreshVoiceMakerBalance,
        googlerUsage, loadingGoogler, refreshGooglerUsage,
        refreshAllBalances
    } = useServices();
    const [isExpanded, setIsExpanded] = useState(false);

    const isAnyLoading = loadingOpenRouter || loadingElevenLabsBot || loadingElevenLabsUnlim || loadingVoiceMaker || loadingGoogler;

    const getIconColor = () => {
        if (isAnyLoading) return '#FFC107'; // Yellow
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

                    <div className="balance-list">
                        <div className="balance-item">
                            <div className="service-name">
                                <div className={`service-status-dot ${loadingOpenRouter ? 'loading' : (openRouterBalance === null ? 'error' : '')}`}></div>
                                {t('balanceMonitor.openrouter') || 'OpenRouter'}
                            </div>
                            <div className="service-balance">
                                {loadingOpenRouter ? '...' : (openRouterBalance !== null ? `$${openRouterBalance.toFixed(4)}` : 'N/A')}
                            </div>
                        </div>

                        <div className="balance-item">
                            <div className="service-name">
                                <div className={`service-status-dot ${loadingElevenLabsBot ? 'loading' : (elevenLabsBotBalance === null ? 'error' : '')}`}></div>
                                {t('balanceMonitor.elevenlabsbot') || 'ElevenLabsBot'}
                            </div>
                            <div className="service-balance">
                                {loadingElevenLabsBot ? '...' : (elevenLabsBotBalance !== null ? elevenLabsBotBalance.toLocaleString() : 'N/A')}
                            </div>
                        </div>

                        <div className="balance-item">
                            <div className="service-name">
                                <div className={`service-status-dot ${loadingElevenLabsUnlim ? 'loading' : (elevenLabsUnlimBalance === null ? 'error' : '')}`}></div>
                                {t('balanceMonitor.elevenlabsunlim') || 'ElevenLabsUnlim'}
                            </div>
                            <div className="service-balance">
                                {loadingElevenLabsUnlim ? '...' : (elevenLabsUnlimBalance !== null ? (elevenLabsUnlimBalance === -1 ? 'Unlimited' : elevenLabsUnlimBalance.toLocaleString()) : 'N/A')}
                            </div>
                        </div>

                        <div className="balance-item">
                            <div className="service-name">
                                <div className={`service-status-dot ${loadingVoiceMaker ? 'loading' : (voiceMakerBalance === null ? 'error' : '')}`}></div>
                                {t('balanceMonitor.voicemaker') || 'VoiceMaker'}
                            </div>
                            <div className="service-balance">
                                {loadingVoiceMaker ? '...' : (voiceMakerBalance !== null ? voiceMakerBalance.toLocaleString() : 'N/A')}
                            </div>
                        </div>

                        <div className="balance-item" style={{ height: 'auto', flexDirection: 'column', alignItems: 'flex-start', gap: '4px', padding: '8px 0' }}>
                            <div className="service-name" style={{ marginBottom: '2px' }}>
                                <div className={`service-status-dot ${loadingGoogler ? 'loading' : (googlerUsage.expiration_date === 0 ? 'error' : '')}`}></div>
                                <span style={{ fontWeight: '600' }}>{t('balanceMonitor.googler') || 'Googler'}</span>
                            </div>
                            <div style={{ display: 'flex', flexDirection: 'column', gap: '2px', width: '100%', paddingLeft: '14px' }}>
                                {/* Загальна кількість зверху */}
                                <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.75em', opacity: 0.9 }}>
                                    <span>🎬 {t('balanceMonitor.videoTotal') || 'Video'}:</span>
                                    <span>{loadingGoogler ? '...' : `${googlerUsage.current_usage.hourly_usage.video_generation || 0}/${googlerUsage.account_limits.video_gen_per_hour_limit}`}</span>
                                </div>
                                <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.75em', opacity: 0.9 }}>
                                    <span>📸 {t('balanceMonitor.imageTotal') || 'Images'}:</span>
                                    <span>{loadingGoogler ? '...' : `${googlerUsage.current_usage.hourly_usage.image_generation || 0}/${googlerUsage.account_limits.img_gen_per_hour_limit}`}</span>
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
                    className="balance-monitor-toggle"
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
