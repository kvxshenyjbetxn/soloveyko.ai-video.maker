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
        openRouterBalances, openRouterKeys, loadingOpenRouter, refreshOpenRouterBalance,
        elevenLabsBotBalances, elevenLabsBotKeys, loadingElevenLabsBot, refreshElevenLabsBotBalance,
        elevenLabsUnlimBalances, elevenLabsUnlimKeys, loadingElevenLabsUnlim, refreshElevenLabsUnlimBalance,
        elevenLabsUABalances, elevenLabsUAKeys, loadingElevenLabsUA, refreshElevenLabsUABalance,
        voiceMakerBalances, voiceMakerKeys, loadingVoiceMaker, refreshVoiceMakerBalance,
        googlerUsage, loadingGoogler, refreshGooglerUsage,
        elevenLabsBotThreshold,
        elevenLabsUnlimThreshold,
        elevenLabsUAThreshold,
        voiceMakerThreshold,
        openRouterThreshold,
        googlerVideoThreshold,
        googlerImageThreshold,
        elevenLabsImageUsage, refreshElevenLabsImageUsage,
        refreshAllBalances
    } = useServices();
    const [isExpanded, setIsExpanded] = useState(false);
    const [isPinned, setIsPinned] = useState(false);

    React.useEffect(() => {
        // @ts-ignore
        if (window.runtime) {
            // @ts-ignore
            const unsub = window.runtime.EventsOn("monitor-opened", (id: string) => {
                if (id !== 'balance' && !isPinned) {
                    setIsExpanded(false);
                }
            });
            return () => unsub();
        }
    }, [isPinned]);

    const handleExpand = (val: boolean) => {
        setIsExpanded(val);
        if (val) {
            // @ts-ignore
            window.runtime?.EventsEmit("monitor-opened", 'balance');
        }
    };

    const isAnyLoading = loadingOpenRouter || loadingElevenLabsBot || loadingElevenLabsUnlim || loadingElevenLabsUA || loadingVoiceMaker || loadingGoogler;

    const isGooglerVideoAlert = googlerVideoThreshold > 0 && (googlerUsage.current_usage.hourly_usage.video_generation || 0) >= googlerVideoThreshold;
    const isGooglerImageAlert = googlerImageThreshold > 0 && (googlerUsage.current_usage.hourly_usage.image_generation || 0) >= googlerImageThreshold;

    const isOpenRouterAlert = Object.entries(openRouterBalances).some(([id, balance]) =>
        balance !== null && openRouterThreshold > 0 && balance < openRouterThreshold
    );

    const isElevenLabsBotAlert = Object.entries(elevenLabsBotBalances).some(([id, balance]) =>
        balance !== null && elevenLabsBotThreshold > 0 && balance < elevenLabsBotThreshold
    );

    const isElevenLabsUnlimAlert = Object.entries(elevenLabsUnlimBalances).some(([id, balance]) =>
        balance !== null && balance !== -1 && elevenLabsUnlimThreshold > 0 && balance < elevenLabsUnlimThreshold
    );

    const isElevenLabsUAAlert = Object.entries(elevenLabsUABalances).some(([id, balance]) =>
        balance !== null && elevenLabsUAThreshold > 0 && balance < elevenLabsUAThreshold
    );

    const isAnyAlertActive = (
        isElevenLabsBotAlert ||
        isElevenLabsUnlimAlert ||
        isElevenLabsUAAlert ||
        Object.entries(voiceMakerBalances).some(([id, balance]) =>
            balance !== null && voiceMakerThreshold > 0 && balance < voiceMakerThreshold
        ) ||
        isOpenRouterAlert ||
        isGooglerVideoAlert ||
        isGooglerImageAlert
    );

    const getIconColor = () => {
        if (isAnyLoading) return '#FFC107'; // Yellow

        if (isAnyAlertActive) {
            return '#ff5252'; // Red warning
        }

        const hasAnyBalance = Object.values(openRouterBalances).some(b => b !== null) ||
            Object.values(elevenLabsBotBalances).some(b => b !== null) ||
            Object.values(elevenLabsUnlimBalances).some(b => b !== null) ||
            Object.values(elevenLabsUABalances).some(b => b !== null) ||
            Object.values(voiceMakerBalances).some(b => b !== null) ||
            googlerUsage.expiration_date !== 0;

        if (!hasAnyBalance) return '#757575'; // Grey
        return '#4caf50'; // Green
    };

    return (
        <div className={`service-balance-monitor ${isExpanded ? 'expanded' : ''} ${isPinned ? 'pinned' : ''}`}>
            <div className="balance-monitor-container">
                {/* Panel */}
                <div className="balance-monitor-panel">
                    <div className="balance-monitor-header">
                        <h3>{t('balanceMonitor.title') || 'Баланси сервісів'}</h3>
                        <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
                            <button
                                className={`pin-btn ${isPinned ? 'active' : ''}`}
                                onClick={() => setIsPinned(!isPinned)}
                                title={isPinned ? t('common.unpin') : t('common.pin')}
                            >
                                <svg width="14" height="14" viewBox="0 0 24 24" fill={isPinned ? "currentColor" : "none"} stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                                    <line x1="12" y1="17" x2="12" y2="22"></line>
                                    <path d="M5 17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.79-.9A2 2 0 0 1 15 10.76V6a3 3 0 0 0-3-3 3 3 0 0 0-3 3v4.76a2 2 0 0 1-1.11 1.79l-1.79.9A2 2 0 0 0 5 15.24Z"></path>
                                </svg>
                            </button>
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
                        {openRouterKeys.map((keyItem) => {
                            const balance = openRouterBalances[keyItem.id];
                            const isAlert = balance !== null && openRouterThreshold > 0 && balance < openRouterThreshold;

                            return (
                                <div className="balance-item" key={keyItem.id}>
                                    <div className="service-name">
                                        <div className={`service-status-dot ${loadingOpenRouter ? 'loading' : (balance === null ? 'error' : '')}`}></div>
                                        <span style={{ fontSize: '0.7em', opacity: 0.5, marginRight: '4px', textTransform: 'uppercase' }}>OpenRouter:</span>
                                        <span style={{ maxWidth: '90px', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{keyItem.name}</span>
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
                                    <div className="service-balance" style={{ color: isAlert ? '#ff5252' : '#4caf50' }}>
                                        {loadingOpenRouter ? '...' : (typeof balance === 'number' ? `$${balance.toFixed(4)}` : 'N/A')}
                                        {isAlert && <span style={{ fontSize: '0.8em', marginLeft: '4px', verticalAlign: 'middle' }}>⚠️</span>}
                                    </div>
                                </div>
                            );
                        })}

                        {elevenLabsBotKeys.map((keyItem) => {
                            const balance = elevenLabsBotBalances[keyItem.id];
                            const isAlert = balance !== null && elevenLabsBotThreshold > 0 && balance < elevenLabsBotThreshold;

                            return (
                                <div className="balance-item" key={keyItem.id}>
                                    <div className="service-name">
                                        <div className={`service-status-dot ${loadingElevenLabsBot ? 'loading' : (balance === null ? 'error' : '')}`}></div>
                                        <span style={{ fontSize: '0.7em', opacity: 0.5, marginRight: '4px', textTransform: 'uppercase' }}>11Labs:</span>
                                        <span style={{ maxWidth: '90px', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{keyItem.name}</span>
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
                                    <div className="service-balance" style={{ color: isAlert ? '#ff5252' : '#4caf50' }}>
                                        {loadingElevenLabsBot ? '...' : (typeof balance === 'number' ? balance.toLocaleString() : 'N/A')}
                                        {isAlert && <span style={{ fontSize: '0.8em', marginLeft: '4px', verticalAlign: 'middle' }}>⚠️</span>}
                                    </div>
                                </div>
                            );
                        })}

                        {elevenLabsUnlimKeys.map((keyItem) => {
                            const balance = elevenLabsUnlimBalances[keyItem.id];
                            const isAlert = balance !== null && balance !== -1 && elevenLabsUnlimThreshold > 0 && balance < elevenLabsUnlimThreshold;

                            return (
                                <div className="balance-item" key={keyItem.id}>
                                    <div className="service-name">
                                        <div className={`service-status-dot ${loadingElevenLabsUnlim ? 'loading' : (balance === null ? 'error' : '')}`}></div>
                                        <span style={{ fontSize: '0.7em', opacity: 0.5, marginRight: '4px', textTransform: 'uppercase' }}>Unlim:</span>
                                        <span style={{ maxWidth: '90px', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{keyItem.name}</span>
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
                                    <div className="service-balance" style={{ color: isAlert ? '#ff5252' : (balance === -1 ? '#FFC107' : '#4caf50') }}>
                                        {loadingElevenLabsUnlim ? '...' : (balance !== null ? (balance === -1 ? 'Unlimited' : balance.toLocaleString()) : 'N/A')}
                                        {isAlert && <span style={{ fontSize: '0.8em', marginLeft: '4px', verticalAlign: 'middle' }}>⚠️</span>}
                                    </div>
                                </div>
                            );
                        })}

                        {elevenLabsUAKeys.map((keyItem) => {
                            const balance = elevenLabsUABalances[keyItem.id];
                            const isAlert = balance !== null && elevenLabsUAThreshold > 0 && balance < elevenLabsUAThreshold;

                            return (
                                <div className="balance-item" key={keyItem.id}>
                                    <div className="service-name">
                                        <div className={`service-status-dot ${loadingElevenLabsUA ? 'loading' : (balance === null ? 'error' : '')}`}></div>
                                        <span style={{ fontSize: '0.7em', opacity: 0.5, marginRight: '4px', textTransform: 'uppercase' }}>11UA:</span>
                                        <span style={{ maxWidth: '90px', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{keyItem.name}</span>
                                        {navigateTo && (
                                            <button
                                                className="service-settings-btn"
                                                onClick={() => { navigateTo('settings.api.voice.elevenlabsua'); setIsExpanded(false); }}
                                                title="Settings"
                                            >
                                                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>
                                            </button>
                                        )}
                                    </div>
                                    <div className="service-balance" style={{ color: isAlert ? '#ff5252' : '#4caf50' }}>
                                        {loadingElevenLabsUA ? '...' : (balance !== null ? balance.toLocaleString() : 'N/A')}
                                        {isAlert && <span style={{ fontSize: '0.8em', marginLeft: '4px', verticalAlign: 'middle' }}>⚠️</span>}
                                    </div>
                                </div>
                            );
                        })}


                        {voiceMakerKeys.map((keyItem) => {
                            const balance = voiceMakerBalances[keyItem.id];
                            const isAlert = balance !== null && voiceMakerThreshold > 0 && balance < voiceMakerThreshold;

                            return (
                                <div className="balance-item" key={keyItem.id}>
                                    <div className="service-name">
                                        <div className={`service-status-dot ${loadingVoiceMaker ? 'loading' : (balance === null ? 'error' : '')}`}></div>
                                        <span style={{ fontSize: '0.7em', opacity: 0.5, marginRight: '4px', textTransform: 'uppercase' }}>V-Maker:</span>
                                        <span style={{ maxWidth: '90px', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{keyItem.name}</span>
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
                                    <div className="service-balance" style={{ color: isAlert ? '#ff5252' : '#4caf50' }}>
                                        {loadingVoiceMaker ? '...' : (typeof balance === 'number' ? balance.toLocaleString() : 'N/A')}
                                        {isAlert && <span style={{ fontSize: '0.8em', marginLeft: '4px', verticalAlign: 'middle' }}>⚠️</span>}
                                    </div>
                                </div>
                            );
                        })}

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
                        handleExpand(newExpanded);
                        if (newExpanded) {
                            if (Object.values(openRouterBalances).some(b => b === null)) refreshOpenRouterBalance();
                            if (Object.values(elevenLabsBotBalances).some(b => b === null)) refreshElevenLabsBotBalance();
                            if (Object.values(elevenLabsUnlimBalances).some(b => b === null)) refreshElevenLabsUnlimBalance();
                            if (Object.values(elevenLabsUABalances).some(b => b === null)) refreshElevenLabsUABalance();
                            if (Object.values(voiceMakerBalances).some(b => b === null)) refreshVoiceMakerBalance();
                            if (googlerUsage.expiration_date === 0) refreshGooglerUsage();
                            refreshElevenLabsImageUsage();
                        }
                    }}
                    title="Balance Monitor"
                    style={{ background: getIconColor() }}
                >
                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                        <path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" />
                    </svg>
                </div>
            </div>
        </div>
    );
};
