import React, { useState, useEffect } from 'react';
import { useI18n } from '../../../contexts/I18nContext';
import { useTheme } from '../../../contexts/ThemeContext';
// @ts-ignore
import { GetGoogleSheetURL, SaveGoogleSheetURL, GetGoogleFilter, SaveGoogleFilter, ParseGoogleSheet, GetTemplates, GetGoogleMonitorMappings, SaveGoogleMonitorMappings, GetGoogleMonitorDisplayColumns, SaveGoogleMonitorDisplayColumns, GetGoogleMonitorTaskNameColumn, SaveGoogleMonitorTaskNameColumn } from '../../../../wailsjs/go/main/App';
import '../general.css';

export const GoogleIntegration = () => {
    const { t } = useI18n();
    const { accentColor } = useTheme();
    const [sheetUrl, setSheetUrl] = useState('');
    const [filter, setFilter] = useState('');
    const [mappings, setMappings] = useState<any[]>([]);
    const [displayColumns, setDisplayColumns] = useState<string>('A');
    const [taskNameColumn, setTaskNameColumn] = useState<string>('B');
    const [allTemplates, setAllTemplates] = useState<any[]>([]);
    const [isParsing, setIsParsing] = useState(false);
    const [results, setResults] = useState<any[]>([]);
    const [statusMsg, setStatusMsg] = useState<{ type: 'success' | 'error', text: string } | null>(null);

    useEffect(() => {
        const loadSettings = async () => {
            try {
                // Завантажуємо URL та фільтр окремо, щоб вони не залежали від нових функцій
                try { setSheetUrl(await GetGoogleSheetURL() || ''); } catch(e) { console.error("URL load fail", e); }
                try { setFilter(await GetGoogleFilter() || ''); } catch(e) { console.error("Filter load fail", e); }
                
                // Завантажуємо мапінги
                try {
                    const m = await GetGoogleMonitorMappings();
                    setMappings(m || []);
                } catch(e) { console.error("Mappings load fail", e); }
                
                // Завантажуємо нове налаштування колонок (безпечно)
                try {
                    if (typeof GetGoogleMonitorDisplayColumns === 'function') {
                        const cols = await GetGoogleMonitorDisplayColumns();
                        setDisplayColumns(cols?.join(', ') || 'A');
                    }
                } catch(e) { console.error("Cols load fail", e); }
                
                // Завантажуємо нове налаштування колонки назви (безпечно)
                try {
                    if (typeof GetGoogleMonitorTaskNameColumn === 'function') {
                        setTaskNameColumn(await GetGoogleMonitorTaskNameColumn() || 'B');
                    }
                } catch(e) { console.error("TaskNameCol load fail", e); }
                
                try {
                    const tpls = await GetTemplates();
                    setAllTemplates(tpls || []);
                } catch(e) { console.error("Templates load fail", e); }
            } catch (err) {
                console.error("General load settings fail", err);
            }
        };
        loadSettings();
    }, []);

    const handleSave = async () => {
        await SaveGoogleSheetURL(sheetUrl);
        await SaveGoogleFilter(filter);
        await SaveGoogleMonitorMappings(mappings);
        
        const colsArray = displayColumns.split(',').map(s => s.trim().toUpperCase()).filter(Boolean);
        if (typeof SaveGoogleMonitorDisplayColumns === 'function') {
            await SaveGoogleMonitorDisplayColumns(colsArray);
        }

        if (typeof SaveGoogleMonitorTaskNameColumn === 'function') {
            await SaveGoogleMonitorTaskNameColumn(taskNameColumn.trim().toUpperCase() || 'B');
        }
        
        setStatusMsg({ type: 'success', text: t('api.googleSettings.saveSuccess') });
        setTimeout(() => setStatusMsg(null), 3000);
    };

    const handleAddMapping = () => {
        setMappings([...mappings, { keyword: '', templateIds: [] }]);
    };

    const handleRemoveMapping = (index: number) => {
        const newMappings = [...mappings];
        newMappings.splice(index, 1);
        setMappings(newMappings);
    };

    const handleMappingChange = (index: number, field: string, value: any) => {
        setMappings(prev => prev.map((m, i) => i === index ? { ...m, [field]: value } : m));
    };

    const handleToggleTemplate = (mIndex: number, tId: string) => {
        setMappings(prev => prev.map((m, i) => {
            if (i !== mIndex) return m;
            const currentIds = m.templateIds || [];
            if (currentIds.includes(tId)) {
                return { ...m, templateIds: currentIds.filter((id: string) => id !== tId) };
            } else {
                return { ...m, templateIds: [...currentIds, tId] };
            }
        }));
    };

    const handleParse = async () => {
        setIsParsing(true);
        setStatusMsg(null);
        try {
            await SaveGoogleSheetURL(sheetUrl);
            await SaveGoogleFilter(filter);
            const data = await ParseGoogleSheet();
            setResults(data || []);
            if (data && data.length > 0) {
                setStatusMsg({ type: 'success', text: t('api.googleSettings.found').replace('{{count}}', data.length.toString()) });
            } else {
                setStatusMsg({ type: 'error', text: t('api.googleSettings.no_results') });
            }
        } catch (err: any) {
            setStatusMsg({ type: 'error', text: err?.message || 'Error' });
        } finally {
            setIsParsing(false);
        }
    };

    const copyToClipboard = (text: string) => {
        navigator.clipboard.writeText(text);
        // Maybe some notification?
    };

    return (
        <div className="content-wrapper animate-fade premium-scrollbar" style={{ overflowY: 'auto', paddingRight: '10px' }}>
            <div className="settings-container" style={{ maxWidth: '1000px' }}>
                <h2 className="settings-title">{t('api.googleSettings.title')}</h2>

                <div className="settings-section glass-panel" style={{ padding: '25px', borderRadius: '12px', background: 'rgba(255, 255, 255, 0.03)', border: '1px solid rgba(255, 255, 255, 0.05)', marginBottom: '30px' }}>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
                        <div>
                            <label style={{ display: 'block', marginBottom: '8px', opacity: 0.7, fontSize: '0.9em' }}>{t('api.googleSettings.spreadsheetUrl')}</label>
                            <input
                                type="text"
                                className="premium-input"
                                style={{
                                    width: '100%',
                                    padding: '12px 16px',
                                    borderRadius: '8px',
                                    border: '1px solid rgba(255, 255, 255, 0.08)',
                                    background: 'rgba(0, 0, 0, 0.3)',
                                    color: '#fff',
                                    outline: 'none',
                                    fontSize: '0.95em'
                                }}
                                value={sheetUrl}
                                onChange={(e) => setSheetUrl(e.target.value)}
                                placeholder={t('api.googleSettings.spreadsheetUrlPlaceholder')}
                            />
                        </div>
                        <div>
                            <label style={{ display: 'block', marginBottom: '8px', opacity: 0.7, fontSize: '0.9em' }}>{t('api.googleSettings.filter')}</label>
                            <input
                                type="text"
                                className="premium-input"
                                style={{
                                    width: '100%',
                                    padding: '12px 16px',
                                    borderRadius: '8px',
                                    border: '1px solid rgba(255, 255, 255, 0.08)',
                                    background: 'rgba(0, 0, 0, 0.3)',
                                    color: '#fff',
                                    outline: 'none',
                                    fontSize: '0.95em'
                                }}
                                value={filter}
                                onChange={(e) => setFilter(e.target.value)}
                                placeholder={t('api.googleSettings.filterPlaceholder')}
                            />
                        </div>
                        <div>
                            <label style={{ display: 'block', marginBottom: '8px', opacity: 0.7, fontSize: '0.9em' }}>{t('google_monitor.display_columns')}</label>
                            <input
                                type="text"
                                className="premium-input"
                                style={{
                                    width: '100%',
                                    padding: '12px 16px',
                                    borderRadius: '8px',
                                    border: '1px solid rgba(255, 255, 255, 0.08)',
                                    background: 'rgba(0, 0, 0, 0.3)',
                                    color: '#fff',
                                    outline: 'none',
                                    fontSize: '0.95em'
                                }}
                                value={displayColumns}
                                onChange={(e) => setDisplayColumns(e.target.value)}
                                placeholder={t('google_monitor.display_columns_placeholder')}
                            />
                        </div>
                        <div>
                            <label style={{ display: 'block', marginBottom: '8px', opacity: 0.7, fontSize: '0.9em' }}>{t('google_monitor.task_name_column')}</label>
                            <input
                                type="text"
                                className="premium-input"
                                style={{
                                    width: '100%',
                                    padding: '12px 16px',
                                    borderRadius: '8px',
                                    border: '1px solid rgba(255, 255, 255, 0.08)',
                                    background: 'rgba(0, 0, 0, 0.3)',
                                    color: '#fff',
                                    outline: 'none',
                                    fontSize: '0.95em'
                                }}
                                value={taskNameColumn}
                                onChange={(e) => setTaskNameColumn(e.target.value)}
                                placeholder={t('google_monitor.task_name_column_placeholder')}
                            />
                        </div>

                        <div style={{ display: 'flex', gap: '15px', justifyContent: 'flex-end', marginTop: '10px' }}>
                            <button
                                onClick={handleSave}
                                style={{
                                    padding: '10px 20px',
                                    borderRadius: '8px',
                                    background: 'rgba(255,255,255,0.05)',
                                    border: '1px solid rgba(255,255,255,0.1)',
                                    color: '#fff',
                                    cursor: 'pointer',
                                    fontWeight: '500'
                                }}
                            >
                                {t('common.save')}
                            </button>
                            <button
                                onClick={handleParse}
                                disabled={isParsing || !sheetUrl}
                                style={{
                                    padding: '10px 24px',
                                    borderRadius: '8px',
                                    background: accentColor,
                                    border: 'none',
                                    color: '#fff',
                                    cursor: 'pointer',
                                    fontWeight: '600',
                                    display: 'flex',
                                    alignItems: 'center',
                                    gap: '8px',
                                    opacity: (isParsing || !sheetUrl) ? 0.5 : 1,
                                    boxShadow: `0 4px 15px ${accentColor}33`
                                }}
                            >
                                {isParsing ? <div className="spinner-small" /> : <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M21 12a9 9 0 0 1-9 9m9-9a9 9 0 0 0-9-9m9 9H3m9 9a9 9 0 0 1-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9" /></svg>}
                                {isParsing ? t('api.googleSettings.parsing') : t('api.googleSettings.parse')}
                            </button>
                        </div>

                        {statusMsg && (
                            <div style={{ color: statusMsg.type === 'success' ? '#4caf50' : '#ff5252', fontSize: '0.85em', textAlign: 'right', fontWeight: '500' }}>
                                {statusMsg.text}
                            </div>
                        )}
                    </div>
                </div>

                <div className="settings-section glass-panel" style={{ padding: '25px', borderRadius: '12px', background: 'rgba(255, 255, 255, 0.03)', border: '1px solid rgba(255, 255, 255, 0.05)', marginBottom: '30px' }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
                        <h3 style={{ margin: 0, fontSize: '1.1em', opacity: 0.9 }}>{t('google_monitor.mappings')}</h3>
                        <button 
                            onClick={handleAddMapping}
                            className="premium-button"
                            style={{ padding: '6px 12px', fontSize: '0.85em', background: 'rgba(255,255,255,0.05)', border: '1px solid rgba(255,255,255,0.1)' }}
                        >
                            + {t('google_monitor.add_mapping')}
                        </button>
                    </div>

                    <div style={{ display: 'flex', flexDirection: 'column', gap: '15px' }}>
                        {mappings.length === 0 && (
                            <div style={{ padding: '20px', textAlign: 'center', opacity: 0.5 }}>
                                {t('google_monitor.no_mappings')}
                            </div>
                        )}
                        {mappings.map((m, mIdx) => (
                            <div key={mIdx} style={{ background: 'rgba(0,0,0,0.2)', padding: '15px', borderRadius: '8px', border: '1px solid rgba(255,255,255,0.05)' }}>
                                <div style={{ display: 'flex', gap: '15px', marginBottom: '15px', alignItems: 'flex-start' }}>
                                    <div style={{ flex: 1 }}>
                                        <label style={{ display: 'block', marginBottom: '6px', opacity: 0.6, fontSize: '0.85em' }}>{t('common.value') || 'Keyword (in Title)'}</label>
                                        <input 
                                            type="text" 
                                            className="premium-input"
                                            style={{ width: '100%', padding: '8px 12px', fontSize: '0.9em', color: '#fff' }}
                                            value={m.keyword}
                                            onChange={(e) => handleMappingChange(mIdx, 'keyword', e.target.value)}
                                            placeholder={t('google_monitor.keyword_placeholder')}
                                        />
                                    </div>
                                    <button 
                                        onClick={() => handleRemoveMapping(mIdx)}
                                        style={{ marginTop: '28px', background: 'none', border: 'none', color: '#ff5252', cursor: 'pointer', opacity: 0.7 }}
                                    >
                                        &times;
                                    </button>
                                </div>
                                
                                <div>
                                    <label style={{ display: 'block', marginBottom: '8px', opacity: 0.6, fontSize: '0.85em' }}>{t('google_monitor.select_templates')}</label>
                                    <div style={{ display: 'flex', flexWrap: 'wrap', gap: '8px' }}>
                                        {allTemplates.map(tpl => (
                                            <button
                                                key={tpl.id}
                                                onClick={() => handleToggleTemplate(mIdx, tpl.id)}
                                                style={{
                                                    padding: '4px 10px',
                                                    fontSize: '0.8em',
                                                    borderRadius: '4px',
                                                    background: m.templateIds?.includes(tpl.id) ? accentColor : 'rgba(255,255,255,0.05)',
                                                    border: '1px solid ' + (m.templateIds?.includes(tpl.id) ? accentColor : 'rgba(255,255,255,0.1)'),
                                                    color: m.templateIds?.includes(tpl.id) ? '#fff' : '#aaa',
                                                    cursor: 'pointer',
                                                    transition: 'all 0.2s'
                                                }}
                                            >
                                                {tpl.name} ({tpl.type})
                                            </button>
                                        ))}
                                    </div>
                                </div>
                            </div>
                        ))}
                    </div>
                </div>

                {results.length > 0 && (
                    <div className="results-list animate-fade" style={{ display: 'flex', flexDirection: 'column', gap: '15px' }}>
                        {results.map((item, idx) => (
                            <div key={idx} className="glass-panel" style={{ padding: '20px', borderRadius: '12px', background: 'rgba(255, 255, 255, 0.02)', border: '1px solid rgba(255, 255, 255, 0.05)' }}>
                                <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '15px', alignItems: 'flex-start' }}>
                                    <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                                        {item.title && (
                                            <div style={{ fontSize: '1em', fontWeight: '700', color: accentColor, opacity: 0.9 }}>
                                                {item.title}
                                            </div>
                                        )}
                                        <div style={{ display: 'flex', flexWrap: 'wrap', gap: '6px' }}>
                                            {item.columns.map((col: string, cidx: number) => (
                                                col.trim().length > 0 && col.length < 50 && !col.includes('docs.google.com') && col !== item.title && (
                                                    <span key={cidx} style={{ padding: '3px 8px', background: 'rgba(255,255,255,0.05)', borderRadius: '4px', fontSize: '0.75em', opacity: 0.8 }}>
                                                        {col}
                                                    </span>
                                                )
                                            ))}
                                        </div>
                                    </div>
                                    <button
                                        onClick={() => copyToClipboard(item.content)}
                                        style={{
                                            padding: '6px 14px',
                                            borderRadius: '6px',
                                            background: accentColor + '22',
                                            border: `1px solid ${accentColor}44`,
                                            color: accentColor,
                                            fontSize: '0.8em',
                                            cursor: 'pointer',
                                            fontWeight: '600',
                                            whiteSpace: 'nowrap'
                                        }}
                                    >
                                        {t('api.googleSettings.copy_content')}
                                    </button>
                                </div>
                                {item.content && (
                                    <div style={{
                                        background: 'rgba(0,0,0,0.2)',
                                        padding: '10px',
                                        borderRadius: '8px',
                                        fontSize: '0.85em',
                                        maxHeight: '100px',
                                        overflowY: 'auto',
                                        opacity: 0.6,
                                        lineHeight: '1.4'
                                    }}>
                                        {item.content}
                                    </div>
                                )}
                                <div style={{ marginTop: '10px', display: 'flex', justifyContent: 'flex-end' }}>
                                    <a
                                        href={item.docLink}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        style={{ fontSize: '0.8em', color: accentColor, opacity: 0.6, textDecoration: 'none' }}
                                    >
                                        {t('api.googleSettings.open_link')} →
                                    </a>
                                </div>
                            </div>
                        ))}
                    </div>
                )}
            </div>

            <style>{`
                .spinner-small { width: 16px; height: 16px; border: 2px solid rgba(255,255,255,0.3); border-top-color: #fff; border-radius: 50%; animation: spin 0.8s linear infinite; }
                @keyframes spin { to { transform: rotate(360deg); } }
                .glass-panel { backdrop-filter: blur(10px); }
            `}</style>
        </div>
    );
};
