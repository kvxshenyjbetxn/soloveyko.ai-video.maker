import React, { useState, useEffect } from 'react';
import './GoogleMonitor.css';
import { useI18n } from '../contexts/I18nContext';
import { useTheme } from '../contexts/ThemeContext';
// @ts-ignore
import { ParseGoogleSheet, GetGoogleSheetURL } from '../../wailsjs/go/main/App';

interface GoogleMonitorProps {
    navigateTo?: (path: string) => void;
    currentPath?: string;
}

export const GoogleMonitor = ({ navigateTo, currentPath }: GoogleMonitorProps) => {
    const { t } = useI18n();
    const { accentColor } = useTheme();
    const [isExpanded, setIsExpanded] = useState(false);
    const [isParsing, setIsParsing] = useState(false);
    const [results, setResults] = useState<any[]>([]);
    const [lastUpdate, setLastUpdate] = useState<Date | null>(null);
    const [copiedId, setCopiedId] = useState<string | null>(null);

    useEffect(() => {
        // @ts-ignore
        if (window.runtime) {
            // @ts-ignore
            const unsub = window.runtime.EventsOn("monitor-opened", (id: string) => {
                if (id !== 'google') {
                    setIsExpanded(false);
                }
            });
            return () => unsub();
        }
    }, []);

    const handleExpand = (val: boolean) => {
        setIsExpanded(val);
        if (val) {
            // @ts-ignore
            window.runtime?.EventsEmit("monitor-opened", 'google');
        }
    };

    const handleRefresh = async (e: React.MouseEvent) => {
        e.stopPropagation();
        setIsParsing(true);
        try {
            const data = await ParseGoogleSheet();
            setResults(data || []);
            setLastUpdate(new Date());
        } catch (err) {
            console.error(err);
        } finally {
            setIsParsing(false);
        }
    };

    const copyToClipboard = (text: string, id: string) => {
        if (!text) return;

        // Спробуємо спочатку стандартний Clipboard API
        navigator.clipboard.writeText(text).then(() => {
            setCopiedId(id);
            setTimeout(() => setCopiedId(null), 2000);
        }).catch(err => {
            console.error('Clipboard error:', err);
            // Фолбек для Wails, якщо доступно
            // @ts-ignore
            if (window.runtime?.ClipboardSetText) {
                // @ts-ignore
                window.runtime.ClipboardSetText(text);
                setCopiedId(id);
                setTimeout(() => setCopiedId(null), 2000);
            }
        });
    };

    return (
        <div className={`google-monitor-wrapper ${isExpanded ? 'expanded' : ''}`}>
            {/* Expanded Panel */}
            <div className="google-mini-panel">
                <div className="google-mini-header">
                    <span className="google-mini-title">Google Sheets</span>
                    <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                        <button
                            className={`mini-refresh-btn ${isParsing ? 'spinning' : ''}`}
                            onClick={handleRefresh}
                            disabled={isParsing}
                            title={t('api.googleSettings.parse')}
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                                <path d="M21 2v6h-6m-9 10H3v-6m18.1-1.9a9 9 0 1 1-2.2-4.9M3.9 16.1a9 9 0 0 1 2.2 4.9" />
                            </svg>
                        </button>
                        <button className="google-close-btn" onClick={() => setIsExpanded(false)}>×</button>
                    </div>
                </div>

                <div className="google-mini-list premium-scrollbar">
                    {results.length === 0 ? (
                        <div className="google-empty-state">
                            {isParsing ? t('api.googleSettings.parsing') : t('api.googleSettings.no_results')}
                        </div>
                    ) : (
                        results.map((item, idx) => (
                            <div key={idx} className="google-mini-item">
                                <div className="google-mini-item-top">
                                    <div style={{ display: 'flex', gap: '6px', alignItems: 'center', overflow: 'hidden' }}>
                                        <button
                                            className="google-mini-copy-btn"
                                            onClick={(e) => {
                                                e.stopPropagation();
                                                copyToClipboard(item.title, `title-${idx}`);
                                            }}
                                            title={t('common.copy')}
                                            style={{ padding: '2px', color: copiedId === `title-${idx}` ? '#4caf50' : '#ffc107' }}
                                        >
                                            {copiedId === `title-${idx}` ?
                                                <svg xmlns="http://www.w3.org/2000/svg" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>
                                                :
                                                <svg xmlns="http://www.w3.org/2000/svg" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
                                            }
                                        </button>
                                        <span className="google-item-index">#{item.index + 1}</span>
                                        {item.title && <span className="google-mini-item-title">{item.title}</span>}
                                    </div>
                                    <button
                                        className="google-mini-copy-btn"
                                        style={{ color: copiedId === `content-${idx}` ? '#4caf50' : accentColor }}
                                        onClick={() => {
                                            // Визначаємо куди вставляти
                                            let targetType = 'translate';
                                            if (currentPath?.includes('rewrite')) targetType = 'rewrite';
                                            if (currentPath?.includes('translate')) targetType = 'translate';

                                            // @ts-ignore
                                            window.runtime?.EventsEmit("applyHistoryEntry", {
                                                type: targetType,
                                                content: item.content,
                                                replace: true
                                            });

                                            copyToClipboard(item.content, `content-${idx}`);
                                        }}
                                        title={t('api.googleSettings.copy_content')}
                                    >
                                        {copiedId === `content-${idx}` ?
                                            <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>
                                            :
                                            <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
                                        }
                                    </button>
                                </div>
                                {item.content && (
                                    <div className="google-mini-content-preview">
                                        {item.content}
                                    </div>
                                )}
                            </div>
                        ))
                    )}
                </div>

                {lastUpdate && (
                    <div className="google-mini-footer">
                        {lastUpdate.toLocaleTimeString()}
                    </div>
                )}
            </div>

            {/* Floating Circle Button */}
            <div
                className={`google-monitor-circle ${isParsing ? 'is-parsing' : ''}`}
                onClick={() => handleExpand(!isExpanded)}
                style={{ backgroundColor: '#2e7d32' }}
            >
                {results.length > 0 && <div className="google-count-badge">{results.length}</div>}
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg>
            </div>
        </div>
    );
};
