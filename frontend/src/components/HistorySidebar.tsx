import React, { useState, useEffect } from 'react';
import { useI18n } from '../contexts/I18nContext';
// @ts-ignore
import { GetHistory } from '../../wailsjs/go/main/App';
// @ts-ignore
import { EventsOn } from '../../wailsjs/runtime/runtime';

interface HistoryEntry {
    id: string;
    taskName: string;
    type: string;
    templates: string[];
    content: string;
    timestamp: string;
}

export const HistorySidebar: React.FC = () => {
    const { t } = useI18n();
    const [history, setHistory] = useState<HistoryEntry[]>([]);
    const [isCollapsed, setIsCollapsed] = useState(false);

    const loadHistory = async () => {
        try {
            const data = await GetHistory();
            setHistory(data || []);
        } catch (err) {
            console.error("Failed to load history:", err);
        }
    };

    useEffect(() => {
        loadHistory();
        // @ts-ignore
        const unsub = EventsOn("historyUpdate", loadHistory);
        return () => {
            if (unsub) unsub();
        };
    }, []);

    return (
        <div className="history-sidebar-container">
            <div className="history-header" onClick={() => setIsCollapsed(!isCollapsed)}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                        <circle cx="12" cy="12" r="10" /><polyline points="12 6 12 12 16 14" />
                    </svg>
                    <span>{t('other.history')}</span>
                </div>
                <span className={`chevron ${!isCollapsed ? 'expanded' : ''}`}>
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                        <polyline points="9 18 15 12 9 6"></polyline>
                    </svg>
                </span>
            </div>
            {!isCollapsed && (
                <div className="history-list animate-fade-in">
                    {history.length === 0 ? (
                        <div className="no-history-mini">{t('logsTab.empty')}</div>
                    ) : (
                        history.map((entry) => (
                            <div key={entry.id} className="history-item" title={entry.content} onClick={() => {
                                // @ts-ignore
                                if (window.runtime) {
                                    // @ts-ignore
                                    window.runtime.EventsEmit("applyHistoryEntry", entry);
                                }
                            }}>
                                <div className="history-item-top">
                                    <span className="history-item-name">{entry.taskName}</span>
                                    <span className="history-item-type">{t(`text.${entry.type}`)}</span>
                                </div>
                                <div className="history-item-templates">
                                    {entry.templates.join(', ')}
                                </div>
                            </div>
                        ))
                    )}
                </div>
            )}
        </div>
    );
};
