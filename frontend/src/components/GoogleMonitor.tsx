import React, { useState, useEffect, useRef } from 'react';
import './GoogleMonitor.css';
import { useI18n } from '../contexts/I18nContext';
import { useTheme } from '../contexts/ThemeContext';
import { useGoogleMonitor } from '../contexts/GoogleMonitorContext';
import { useToast } from '../contexts/ToastContext';

interface GoogleMonitorProps {
    navigateTo?: (path: string) => void;
    currentPath?: string;
}

export const GoogleMonitor = ({ navigateTo, currentPath }: GoogleMonitorProps) => {
    const { t } = useI18n();
    const { accentColor } = useTheme();
    const { showToast } = useToast();
    const { 
        isParsing, 
        sheetResults, 
        activeSheetId, 
        setActiveSheetId, 
        scanSheets, 
        clearResults,
        handleCreateTask,
        fetchContentIfNeeded,
        loadingItemId,
        sheetsConfig
    } = useGoogleMonitor();

    const [isExpanded, setIsExpanded] = useState(false);
    const [copiedId, setCopiedId] = useState<string | null>(null);
    const [isPinned, setIsPinned] = useState(false);
    const wrapperRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (wrapperRef.current && !wrapperRef.current.contains(event.target as Node) && isExpanded && !isPinned) {
                setIsExpanded(false);
            }
        };
        document.addEventListener('mousedown', handleClickOutside);
        return () => document.removeEventListener('mousedown', handleClickOutside);
    }, [isExpanded, isPinned]);

    const handleExpand = (val: boolean) => {
        setIsExpanded(val);
        if (val) {
            // @ts-ignore
            window.runtime?.EventsEmit("monitor-opened", 'google');
        }
    };

    const onScan = async (e?: React.MouseEvent) => {
        if (e) e.stopPropagation();
        try {
            const data = await scanSheets();
            if (data && data.length > 0) {
                const total = data.reduce((acc: number, s: any) => acc + (s.results?.length || 0), 0);
                if (total > 0 && !isExpanded) setIsExpanded(true);
            }
        } catch (err) {
            // Error is handled in context/toast
        }
    };

    const copyToClipboard = async (text: string, id: string, docLink?: string, sheetId?: string, idx?: number) => {
        let content = text;
        
        if (!content && docLink && sheetId && idx !== undefined) {
            try {
                content = await fetchContentIfNeeded(sheetId, idx);
            } catch (err) {
                return; // Error handled in fetchContentIfNeeded
            }
        }

        if (!content) return;

        navigator.clipboard.writeText(content);
        // @ts-ignore
        window.runtime?.EventsEmit("applyHistoryEntry", { type: 'translate', content: content, replace: true });
        if (navigateTo) navigateTo('text.translate');
        setCopiedId(id);
        setTimeout(() => setCopiedId(null), 1500);
    };

    const activeResult = sheetResults.find(r => r.id === activeSheetId) || (sheetResults.length > 0 ? sheetResults[0] : null);
    const activeConfig = sheetsConfig.find(s => s.id === (activeResult?.id || activeSheetId)) || (sheetsConfig.length > 0 ? sheetsConfig[0] : null);
    const results = activeResult?.results || [];
    const displayColumns = activeConfig?.displayColumns || ['A'];

    const totalResultsCount = sheetResults.reduce((acc, s) => acc + (s.results?.length || 0), 0);
    
    const resultsWithTemplates = React.useMemo(() => {
        const mappings = activeConfig?.mappings || [];
        const hasGlobal = activeConfig?.globalTemplateIds && activeConfig.globalTemplateIds.length > 0;
        
        return results.map(item => {
            if (hasGlobal) return { ...item, hasTemplates: true };
            const hasMapping = mappings.some(m => {
                if (!m.keyword || !m.templateIds || m.templateIds.length === 0) return false;
                const kw = m.keyword.toLowerCase();
                return item.columns?.some((c: string) => c?.toLowerCase().includes(kw)) || item.title?.toLowerCase().includes(kw);
            });
            return { ...item, hasTemplates: hasMapping };
        });
    }, [results, activeConfig]);

    return (
        <div ref={wrapperRef} className={`google-monitor-wrapper ${isExpanded ? 'expanded' : ''} ${isPinned ? 'pinned' : ''}`}>
            {isExpanded && sheetResults.length > 1 && (
                <div className="google-monitor-tabs-sidebar">
                    {sheetResults.map(s => (
                        <div key={s.id} className={`google-tab-item ${activeSheetId === s.id ? 'active' : ''}`} onClick={() => setActiveSheetId(s.id)}>
                            <span className="google-tab-vertical-text">{s.name}</span>
                            {s.results?.length > 0 && <span className="google-tab-count">{s.results.length}</span>}
                        </div>
                    ))}
                </div>
            )}

            <div className="google-mini-panel premium-glass">
                <div className="google-mini-header">
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke={accentColor} strokeWidth="3"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line></svg>
                        <span style={{ fontWeight: 800, fontSize: '10px', textTransform: 'uppercase', color: accentColor, letterSpacing: '0.8px' }}>
                            {activeResult ? activeResult.name : 'Sheet'}
                        </span>
                    </div>

                    <div className="google-mini-actions">
                        <button className={`mini-header-btn ${isParsing ? 'spinning' : ''}`} onClick={onScan} disabled={isParsing}><svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5"><path d="M23 4v6h-6"></path><path d="M1 20v-6h6"></path><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path></svg></button>
                        <button className="mini-header-btn" onClick={clearResults}><svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg></button>
                        <button className={`mini-header-btn ${isPinned ? 'active' : ''}`} onClick={() => setIsPinned(!isPinned)} style={{ border: isPinned ? `1px solid ${accentColor}` : '1px solid rgba(255,255,255,0.1)' }}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke={isPinned ? accentColor : "currentColor"} strokeWidth="2.5"><path d="M21 8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16Z"></path><path d="m3.3 7 8.7 5 8.7-5"></path><path d="M12 22V12"></path></svg>
                        </button>
                        <button className="mini-header-btn close-btn" onClick={() => setIsExpanded(false)}><svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#ff4d4d" strokeWidth="3"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg></button>
                    </div>
                </div>

                <div className="google-mini-list custom-scrollbar">
                    {activeResult?.error && (
                        <div style={{ padding: '10px', color: '#ff4d4d', fontSize: '11px', textAlign: 'center', background: 'rgba(255,77,77,0.05)', borderRadius: '8px', margin: '5px' }}>
                            Помилка: {activeResult.error}
                        </div>
                    )}
                    {results.length === 0 && !activeResult?.error && (
                        <div style={{ padding: '20px', opacity: 0.3, fontSize: '11px', textAlign: 'center' }}>
                            Нічого не знайдено
                        </div>
                    )}
                    {resultsWithTemplates.map((item, idx) => {
                        const itemId = `${activeSheetId}-${idx}`;
                        return (
                            <div key={itemId} className="google-mini-item">
                                <div className="google-mini-item-main" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '4px' }}>
                                    <div style={{ display: 'flex', gap: '6px', alignItems: 'center', overflow: 'hidden', flex: 1 }}>
                                        <button className="google-mini-copy-btn" onClick={() => copyToClipboard(item.title, `title-${idx}`)} style={{ color: copiedId === `title-${idx}` ? '#4caf50' : '#ffc107', padding: '0', minWidth: '14px' }}>
                                            {copiedId === `title-${idx}` ? <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3"><polyline points="20 6 9 17 4 12"></polyline></svg> : <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>}
                                        </button>
                                        <div className="google-mini-item-title-container" style={{ display: 'flex', gap: '6px', overflow: 'hidden', alignItems: 'center', flexWrap: 'wrap' }}>
                                            {displayColumns.map((col, cIdx) => {
                                                const cleanCol = col.trim().toUpperCase();
                                                if (!cleanCol) return null;
                                                const colIdx = cleanCol.split('').reduce((acc: number, char: string) => acc * 26 + (char.charCodeAt(0) - 64), 0) - 1;
                                                const val = item.columns?.[colIdx] || "";
                                                
                                                if (cIdx === 0) {
                                                    if (!val) return null;
                                                    return <span key={cIdx} style={{ fontSize: '11px', color: '#4caf50', fontWeight: '800', maxWidth: '200px', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{val}</span>;
                                                }
                                                return (
                                                    <span key={cIdx} title={cleanCol} style={{ 
                                                        fontSize: '9px', 
                                                        color: val ? 'rgba(255,255,255,0.7)' : 'rgba(255,255,255,0.2)', 
                                                        background: val ? 'rgba(255,255,255,0.08)' : 'transparent',
                                                        padding: '1px 5px',
                                                        borderRadius: '4px',
                                                        border: val ? '1px solid rgba(255,255,255,0.1)' : '1px solid rgba(255,255,255,0.05)',
                                                        fontWeight: '600',
                                                        whiteSpace: 'nowrap',
                                                        minWidth: '15px',
                                                        textAlign: 'center'
                                                    }}>
                                                        {val || '-'}
                                                    </span>
                                                );
                                            })}
                                        </div>
                                    </div>
                                    <div style={{ display: 'flex', gap: '6px', marginLeft: '6px', flexShrink: 0 }}>
                                        {item.hasTemplates && (
                                            <button className="google-mini-create-btn" onClick={() => handleCreateTask(activeSheetId!, idx)} style={{ color: '#4caf50', padding: '2px', position: 'relative' }}>
                                                {loadingItemId === itemId ? <div className="spinner-mini" /> : <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>}
                                            </button>
                                        )}
                                        <button className="google-mini-copy-btn" onClick={() => copyToClipboard(item.content, `content-${idx}`, item.docLink, activeSheetId || 'default', idx)} style={{ color: copiedId === `content-${idx}` ? '#4caf50' : accentColor, padding: '2px', position: 'relative' }}>
                                            {loadingItemId === itemId ? <div className="spinner-mini" /> : (copiedId === `content-${idx}` ? <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3"><polyline points="20 6 9 17 4 12"></polyline></svg> : <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>)}
                                        </button>
                                    </div>
                                </div>
                            </div>
                        );
                    })}
                </div>
            </div>
            <div className={`google-monitor-circle ${isParsing ? 'is-parsing' : ''}`} onClick={() => handleExpand(!isExpanded)}>
                {totalResultsCount > 0 && <div className="google-count-badge">{totalResultsCount}</div>}
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line></svg>
            </div>
        </div>
    );
};
