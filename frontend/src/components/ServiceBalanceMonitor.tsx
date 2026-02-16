import React, { useState } from 'react';
import './ServiceBalanceMonitor.css';
import { useI18n } from '../contexts/I18nContext';
import { useServices } from '../contexts/ServiceContext';

export const ServiceBalanceMonitor = () => {
    const { t } = useI18n();
    const { openRouterBalance, loadingOpenRouter, refreshOpenRouterBalance } = useServices();
    const [isExpanded, setIsExpanded] = useState(false);

    const getIconColor = () => {
        if (loadingOpenRouter) return '#FFC107'; // Yellow
        if (openRouterBalance === null) return '#757575'; // Grey
        if (openRouterBalance < 1) return '#ff5252'; // Red
        return '#4caf50'; // Green
    };

    return (
        <div className={`service-balance-monitor ${isExpanded ? 'expanded' : 'collapsed'}`}>
            <div className="balance-monitor-container">
                {/* Panel */}
                <div className="balance-monitor-panel">
                    <div className="balance-monitor-header">
                        <h3>{t('balanceMonitor.title') || 'Баланси сервісів'}</h3>
                        <button className="balance-close-btn" onClick={() => setIsExpanded(false)}>&times;</button>
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
                    </div>
                </div>

                {/* Toggle Button */}
                <div
                    className="balance-monitor-toggle"
                    onClick={() => {
                        setIsExpanded(!isExpanded);
                        // Optional: Refresh on expand if needed, but context handles initial load
                        if (!isExpanded && openRouterBalance === null) {
                            refreshOpenRouterBalance();
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
