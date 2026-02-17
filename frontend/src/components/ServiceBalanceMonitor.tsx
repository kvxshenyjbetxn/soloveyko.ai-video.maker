import React, { useState } from 'react';
import './ServiceBalanceMonitor.css';
import { useI18n } from '../contexts/I18nContext';
import { useServices } from '../contexts/ServiceContext';

export const ServiceBalanceMonitor = () => {
    const { t } = useI18n();
    const {
        openRouterBalance, loadingOpenRouter, refreshOpenRouterBalance,
        elevenLabsBotBalance, loadingElevenLabsBot, refreshElevenLabsBotBalance,
        refreshAllBalances
    } = useServices();
    const [isExpanded, setIsExpanded] = useState(false);

    const isAnyLoading = loadingOpenRouter || loadingElevenLabsBot;

    const getIconColor = () => {
        if (isAnyLoading) return '#FFC107'; // Yellow
        if (openRouterBalance === null && elevenLabsBotBalance === null) return '#757575'; // Grey
        if ((openRouterBalance !== null && openRouterBalance < 1) || (elevenLabsBotBalance !== null && elevenLabsBotBalance < 5000)) return '#ff5252'; // Red
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
