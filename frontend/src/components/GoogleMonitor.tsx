import React, { useState, useEffect, useRef } from 'react';
import './GoogleMonitor.css';
import { useI18n } from '../contexts/I18nContext';
import { useTheme } from '../contexts/ThemeContext';
import { useTemplates } from '../contexts/TemplateContext';
import { useQueueActions } from '../contexts/QueueContext';
import { useToast } from '../contexts/ToastContext';
// @ts-ignore
import { ParseGoogleSheet, GetGoogleSheetURL, GetGoogleMonitorMappings, CheckExistingTasks, GetGoogleMonitorDisplayColumns, GetGoogleMonitorTaskNameColumn } from '../../wailsjs/go/main/App';

interface GoogleMonitorProps {
    navigateTo?: (path: string) => void;
    currentPath?: string;
}

export const GoogleMonitor = ({ navigateTo, currentPath }: GoogleMonitorProps) => {
    const { t } = useI18n();
    const { accentColor } = useTheme();
    const { templates, flattenSettings } = useTemplates();
    const { addTasks, addTask, getNextTaskName } = useQueueActions();
    const { showToast } = useToast();

    const [isExpanded, setIsExpanded] = useState(false);
    const [isParsing, setIsParsing] = useState(false);
    const [results, setResults] = useState<any[]>([]);
    const [mappings, setMappings] = useState<any[]>([]);
    const [displayColumns, setDisplayColumns] = useState<string[]>(['A']);
    const [taskNameColumn, setTaskNameColumn] = useState<string>('B');
    const [lastUpdate, setLastUpdate] = useState<Date | null>(null);
    const [copiedId, setCopiedId] = useState<string | null>(null);
    const [isPinned, setIsPinned] = useState(false);
    const wrapperRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const loadSettings = async () => {
            try {
                let m: any[] = [];
                let cols = ['A'];
                let nameCol = 'B';
                
                try { 
                    m = await GetGoogleMonitorMappings(); 
                } catch(e) { console.error("Monitor mappings load fail", e); }
                
                try {
                    if (typeof GetGoogleMonitorDisplayColumns === 'function') {
                        cols = await GetGoogleMonitorDisplayColumns();
                    }
                } catch(e) { console.error("Monitor cols load fail", e); }

                try {
                    if (typeof GetGoogleMonitorTaskNameColumn === 'function') {
                        nameCol = await GetGoogleMonitorTaskNameColumn();
                    }
                } catch(e) { console.error("Monitor nameCol load fail", e); }
                
                setMappings(m || []);
                setDisplayColumns(cols || ['A']);
                setTaskNameColumn(nameCol || 'B');
            } catch (err) {
                console.error("Failed to load monitor settings:", err);
            }
        };
        if (isExpanded) {
            loadSettings();
        }
    }, [isExpanded]);

    useEffect(() => {
        // @ts-ignore
        if (window.runtime) {
            // @ts-ignore
            const unsub = window.runtime.EventsOn("monitor-opened", (id: string) => {
                if (id !== 'google' && !isPinned) {
                    setIsExpanded(false);
                }
            });
            return () => unsub();
        }
    }, [isPinned]);

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (wrapperRef.current && !wrapperRef.current.contains(event.target as Node) && isExpanded && !isPinned) {
                setIsExpanded(false);
            }
        };

        document.addEventListener('mousedown', handleClickOutside);
        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
        };
    }, [isExpanded, isPinned]);

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

    const handleClear = (e: React.MouseEvent) => {
        e.stopPropagation();
        setResults([]);
        setLastUpdate(null);
    };

    const handleCreateTask = async (item: any) => {
        if (!item.content) {
            showToast("Content is empty", "error");
            return;
        }

        const mapping = mappings.find(m => {
            if (!m.keyword) return false;
            const kw = m.keyword.toLowerCase();
            return item.columns?.some((c: string) => c?.toLowerCase().includes(kw)) || item.title?.toLowerCase().includes(kw);
        });

        if (!mapping || !mapping.templateIds || mapping.templateIds.length === 0) {
            showToast("No mapping found for this item", "info");
            return;
        }

        // Determine task name from configured column
        let taskName = getNextTaskName();
        if (taskNameColumn) {
            const colIdx = taskNameColumn.toUpperCase().split('').reduce((acc, char) => acc * 26 + (char.charCodeAt(0) - 64), 0) - 1;
            const customName = item.columns && item.columns[colIdx];
            if (customName && customName.trim()) {
                taskName = customName.trim();
            }
        }
        
        const content = item.content;

        // Find relevant templates
        const matchedTemplates = mapping.templateIds
            .map((id: string) => templates.find(t => t.id === id))
            .filter(Boolean);

        if (matchedTemplates.length === 0) {
            showToast("Mapped templates not found", "error");
            return;
        }

        // We assume the first template determines the type (translate/rewrite/voiceover)
        // or we just use 'translate' as default if not clear.
        // Actually templates HAVE a type.
        const type = matchedTemplates[0]!.type;

        const tasksToCheck = matchedTemplates.map((tpl: any) => ({
            taskName,
            taskType: tpl!.type,
            subName: tpl!.name,
            settings: flattenSettings(tpl!.settings)
        }));

        try {
            const results = await CheckExistingTasks(tasksToCheck);
            // We can just add them, CheckExistingTasks is primarily for UI warning which we skip here for speed
            // because the user wants "automatic" creation.
            addTasks(type, content, tasksToCheck, taskName);
            showToast(`Added ${tasksToCheck.length} tasks to queue`, "success");
        } catch (err) {
            console.error("Failed to check tasks:", err);
            addTasks(type, content, tasksToCheck, taskName);
        }
    };

    return (
        <div className={`google-monitor-wrapper ${isExpanded ? 'expanded' : ''} ${isPinned ? 'pinned' : ''}`} ref={wrapperRef}>
            {/* Expanded Panel */}
            <div className="google-mini-panel">
                <div className="google-mini-header">
                    <span className="google-mini-title">Google Sheets</span>
                    <div className="google-header-controls">
                        <button
                            className="mini-refresh-btn"
                            onClick={handleClear}
                            title={t('api.googleSettings.clear')}
                            style={{ color: '#f44336' }}
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                                <path d="M3 6h18m-2 0v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6m3 0V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
                            </svg>
                        </button>
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
                        <button className="google-close-btn" onClick={() => setIsExpanded(false)}>&times;</button>
                    </div>
                </div>

                <div className="google-mini-list premium-scrollbar">
                    {results.length === 0 ? (
                        <div className="google-empty-state">
                            {isParsing ? t('api.googleSettings.parsing') : t('api.googleSettings.no_results')}
                        </div>
                    ) : (
                        results.map((item, idx) => {
                            const mapping = mappings.find(m => {
                                if (!m.keyword) return false;
                                const kw = m.keyword.toLowerCase();
                                return item.columns?.some((c: string) => c?.toLowerCase().includes(kw)) || item.title?.toLowerCase().includes(kw);
                            });
                            const hasTemplates = mapping && mapping.templateIds && mapping.templateIds.length > 0;

                            return (
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
                                            {item.columns && item.columns.length > 8 && item.columns[8] && <span className="google-item-index">{item.columns[8]}</span>}
                                            <div className="google-mini-item-title-container">
                                                {displayColumns.map((col, cIdx) => {
                                                    const colIdx = col.toUpperCase().split('').reduce((acc, char) => acc * 26 + (char.charCodeAt(0) - 64), 0) - 1;
                                                    const val = item.columns && item.columns[colIdx];
                                                    if (!val) return null;
                                                    
                                                    // Стріпуємо, якщо це дублікат ключа мапінгу
                                                    const kw = mapping?.keyword?.toLowerCase();
                                                    if (cIdx > 0 && kw && val.toLowerCase().includes(kw)) return null;

                                                    return (
                                                        <span key={cIdx} className={`google-mini-item-col-${col}`} style={{ 
                                                            color: cIdx === 0 ? '#4caf50' : 'rgba(255,255,255,0.7)',
                                                            fontWeight: cIdx === 0 ? 'bold' : 'normal',
                                                            marginRight: '6px',
                                                            whiteSpace: 'nowrap',
                                                            overflow: 'hidden',
                                                            textOverflow: 'ellipsis'
                                                        }}>
                                                            {val}
                                                        </span>
                                                    );
                                                })}
                                            </div>
                                        </div>
                                        <div style={{ display: 'flex', gap: '4px', alignItems: 'center' }}>
                                            {hasTemplates && (
                                                <button
                                                    className="google-mini-create-btn"
                                                    onClick={() => handleCreateTask(item)}
                                                    title={t('google_monitor.create_task') || "Create Task"}
                                                    style={{ color: '#4caf50' }}
                                                >
                                                    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
                                                        <line x1="12" y1="5" x2="12" y2="19"></line>
                                                        <line x1="5" y1="12" x2="19" y2="12"></line>
                                                    </svg>
                                                </button>
                                            )}
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
                                    </div>
                                    {item.content && (
                                        <div className="google-mini-content-preview">
                                            {item.content}
                                        </div>
                                    )}
                                </div>
                            );
                        })
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
