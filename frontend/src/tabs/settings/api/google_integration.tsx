import React, { useState, useEffect } from 'react';
import { useI18n } from '../../../contexts/I18nContext';
import { useTheme } from '../../../contexts/ThemeContext';
// @ts-ignore
import { GetGoogleSheets, SaveGoogleSheets, GetTemplates, ParseGoogleSheet } from '../../../../wailsjs/go/main/App';
import '../general.css';

interface GoogleSheetConfig {
    id: string;
    name: string;
    url: string;
    filter: string;
    mappings: Array<{ keyword: string; templateIds: string[] }>;
    globalTemplateIds: string[];
    displayColumns: string[];
    taskNameColumn: string;
    statusColumn: string;
    statusValue: string;
    ignoreRows: number;
}

export const GoogleIntegration = () => {
    const { t } = useI18n();
    const { accentColor } = useTheme();
    const [sheets, setSheets] = useState<GoogleSheetConfig[]>([]);
    const [activeIdx, setActiveIdx] = useState(0);
    const [displayColsInput, setDisplayColsInput] = useState("");
    const [allTemplates, setAllTemplates] = useState<any[]>([]);
    const [isParsing, setIsParsing] = useState(false);
    const [statusMsg, setStatusMsg] = useState<{ type: 'success' | 'error', text: string } | null>(null);

    useEffect(() => {
        const load = async () => {
            try {
                const s = await GetGoogleSheets();
                const tpls = await GetTemplates();
                setAllTemplates(tpls || []);
                
                if (s && s.length > 0) {
                    const loadedSheets = s.map((sheet: any) => ({
                        ...sheet,
                        globalTemplateIds: sheet.globalTemplateIds || []
                    }));
                    setSheets(loadedSheets);
                    setActiveIdx(0);
                    setDisplayColsInput(loadedSheets[0].displayColumns?.join(', ') || "");
                } else {
                    const newSheet: GoogleSheetConfig = {
                        id: Math.random().toString(36).substr(2, 9),
                        name: 'Таблиця 1',
                        url: '',
                        filter: '',
                        mappings: [],
                        globalTemplateIds: [],
                        displayColumns: ['A'],
                        taskNameColumn: 'B',
                        statusColumn: '',
                        statusValue: '',
                        ignoreRows: 0
                    };
                    setSheets([newSheet]);
                    setActiveIdx(0);
                    setDisplayColsInput(newSheet.displayColumns.join(', ') || "");
                }
            } catch (err) {
                console.error("Failed to load sheets", err);
            }
        };
        load();
    }, []);

    useEffect(() => {
        if (sheets[activeIdx]) {
            setDisplayColsInput(sheets[activeIdx].displayColumns?.join(', ') || "");
        }
    }, [activeIdx]); // Remove 'sheets' to prevent overwriting during typing

    const activeSheet = sheets[activeIdx];

    const updateActiveSheet = (updates: Partial<GoogleSheetConfig>) => {
        setSheets(prev => prev.map((s, i) => i === activeIdx ? { ...s, ...updates } : s));
    };

    const handleSave = async () => {
        try {
            await SaveGoogleSheets(sheets as any);
            setStatusMsg({ type: 'success', text: "Налаштування успішно збережено" });
            setTimeout(() => setStatusMsg(null), 3000);
        } catch (err) {
            setStatusMsg({ type: 'error', text: 'Error saving settings' });
        }
    };

    const handleAddTable = () => {
        const newSheet: GoogleSheetConfig = {
            id: Math.random().toString(36).substr(2, 9),
            name: `Таблиця ${sheets.length + 1}`,
            url: '',
            filter: '',
            mappings: [],
            globalTemplateIds: [],
            displayColumns: ['A'],
            taskNameColumn: 'B',
            statusColumn: '',
            statusValue: '',
            ignoreRows: 0
        };
        setSheets([...sheets, newSheet]);
        setActiveIdx(sheets.length);
    };

    const handleRemoveTable = (idx: number) => {
        if (sheets.length <= 1) return;
        const newSheets = sheets.filter((_, i) => i !== idx);
        setSheets(newSheets);
        if (activeIdx >= newSheets.length) {
            setActiveIdx(Math.max(0, newSheets.length - 1));
        }
    };

    const handleAddMapping = () => {
        if (!activeSheet) return;
        const newMappings = [...(activeSheet.mappings || []), { keyword: '', templateIds: [] }];
        updateActiveSheet({ mappings: newMappings });
    };

    const handleRemoveMapping = (mIdx: number) => {
        if (!activeSheet) return;
        const newMappings = activeSheet.mappings.filter((_, i) => i !== mIdx);
        updateActiveSheet({ mappings: newMappings });
    };

    const handleMappingChange = (mIdx: number, field: string, value: any) => {
        if (!activeSheet) return;
        const newMappings = activeSheet.mappings.map((m, i) => i === mIdx ? { ...m, [field]: value } : m);
        updateActiveSheet({ mappings: newMappings });
    };

    const handleToggleTemplate = (mIdx: number, tId: string) => {
        if (!activeSheet) return;
        const newMappings = activeSheet.mappings.map((m, i) => {
            if (i !== mIdx) return m;
            const currentIds = m.templateIds || [];
            if (currentIds.includes(tId)) {
                return { ...m, templateIds: currentIds.filter(id => id !== tId) };
            } else {
                return { ...m, templateIds: [...currentIds, tId] };
            }
        });
        updateActiveSheet({ mappings: newMappings });
    };

    const handleToggleGlobalTemplate = (tId: string) => {
        if (!activeSheet) return;
        const currentIds = activeSheet.globalTemplateIds || [];
        if (currentIds.includes(tId)) {
            updateActiveSheet({ globalTemplateIds: currentIds.filter(id => id !== tId) });
        } else {
            updateActiveSheet({ globalTemplateIds: [...currentIds, tId] });
        }
    };

    const handleParse = async () => {
        if (!activeSheet || !activeSheet.url) return;
        setIsParsing(true);
        try {
            await SaveGoogleSheets(sheets as any);
            const data = await ParseGoogleSheet(activeSheet as any);
            if (data && data.length > 0) {
                setStatusMsg({ type: 'success', text: `Знайдено ${data.length} рядків` });
            } else {
                setStatusMsg({ type: 'error', text: 'Нічого не знайдено за заданим фільтром' });
            }
        } catch (err: any) {
            setStatusMsg({ type: 'error', text: err?.message || 'Error parsing sheet' });
        } finally {
            setIsParsing(false);
        }
    };

    return (
        <div className="google-settings-multi" style={{ display: 'flex', gap: '20px', height: '100%', maxHeight: 'calc(100vh - 180px)' }}>
            {/* Sidebar with tables */}
            <div className="google-sidebar" style={{
                width: '200px',
                background: 'rgba(255,255,255,0.02)',
                borderRadius: '12px',
                padding: '12px',
                border: '1px solid rgba(255,255,255,0.05)',
                display: 'flex',
                flexDirection: 'column',
                gap: '8px',
                overflowY: 'auto'
            }}>
                <div style={{ padding: '0 5px 10px 5px', fontSize: '11px', opacity: 0.5, fontWeight: 'bold', textTransform: 'uppercase' }}>
                    Список таблиць
                </div>
                {sheets.map((s, idx) => (
                    <div
                        key={s.id}
                        onClick={() => setActiveIdx(idx)}
                        style={{
                            padding: '10px 12px',
                            borderRadius: '8px',
                            cursor: 'pointer',
                            background: activeIdx === idx ? accentColor + '15' : 'transparent',
                            border: `1px solid ${activeIdx === idx ? accentColor + '30' : 'transparent'}`,
                            color: activeIdx === idx ? accentColor : '#fff',
                            fontSize: '13px',
                            display: 'flex',
                            justifyContent: 'space-between',
                            alignItems: 'center',
                            transition: 'all 0.2s'
                        }}
                    >
                        <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', maxWidth: '130px' }}>
                            {s.name || `Table ${idx + 1}`}
                        </span>
                        {sheets.length > 1 && (
                            <button
                                onClick={(e) => { e.stopPropagation(); handleRemoveTable(idx); }}
                                style={{ background: 'none', border: 'none', color: '#ff4d4d', cursor: 'pointer', opacity: 0.5, padding: '2px', fontSize: '16px' }}
                            >
                                &times;
                            </button>
                        )}
                    </div>
                ))}
                <button
                    onClick={handleAddTable}
                    style={{
                        marginTop: '10px',
                        padding: '10px',
                        borderRadius: '8px',
                        background: 'rgba(255,255,255,0.03)',
                        border: '1px dotted rgba(255,255,255,0.2)',
                        color: '#666',
                        cursor: 'pointer',
                        fontSize: '12px',
                        transition: 'all 0.2s'
                    }}
                >
                    + Додати нову таблицю
                </button>
            </div>

            {/* Main Content */}
            <div className="google-content custom-scrollbar" style={{
                flex: 1,
                display: 'flex',
                flexDirection: 'column',
                gap: '20px',
                overflowY: 'auto',
                paddingRight: '12px'
            }}>
                {activeSheet && (
                    <>
                        <div className="glass-panel" style={{ padding: '20px', borderRadius: '16px', border: '1px solid rgba(255,255,255,0.05)', background: 'rgba(255,255,255,0.01)' }}>
                            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '15px', marginBottom: '15px' }}>
                                <div>
                                    <label style={{ display: 'block', marginBottom: '8px', opacity: 0.5, fontSize: '12px' }}>Назва таблиці</label>
                                    <input
                                        type="text"
                                        className="premium-input"
                                        value={activeSheet.name}
                                        onChange={e => updateActiveSheet({ name: e.target.value })}
                                        style={{ width: '100%', padding: '10px', background: 'rgba(0,0,0,0.2)', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '8px', color: '#fff' }}
                                    />
                                </div>
                                <div>
                                    <label style={{ display: 'block', marginBottom: '8px', opacity: 0.5, fontSize: '12px' }}>URL таблиці</label>
                                    <input
                                        type="text"
                                        className="premium-input"
                                        value={activeSheet.url}
                                        onChange={e => updateActiveSheet({ url: e.target.value })}
                                        style={{ width: '100%', padding: '10px', background: 'rgba(0,0,0,0.2)', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '8px', color: '#fff' }}
                                    />
                                </div>
                            </div>

                            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))', gap: '15px', marginBottom: '20px' }}>
                                <div>
                                    <label style={{ display: 'block', marginBottom: '8px', opacity: 0.5, fontSize: '12px' }}>Фільтр (напр. G:!Done)</label>
                                    <input
                                        type="text"
                                        className="premium-input"
                                        value={activeSheet.filter}
                                        onChange={e => updateActiveSheet({ filter: e.target.value })}
                                        style={{ width: '100%', padding: '10px', background: 'rgba(0,0,0,0.2)', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '8px', color: '#fff' }}
                                    />
                                </div>
                                <div>
                                    <label style={{ display: 'block', marginBottom: '8px', opacity: 0.5, fontSize: '12px' }}>Стовпці монітора (A, B)</label>
                                    <input
                                        type="text"
                                        className="premium-input"
                                        value={displayColsInput}
                                        onChange={e => setDisplayColsInput(e.target.value)}
                                        onBlur={() => {
                                            const cols = displayColsInput.split(',').map(s => s.trim().toUpperCase()).filter(Boolean);
                                            updateActiveSheet({ displayColumns: cols });
                                            setDisplayColsInput(cols.join(', '));
                                        }}
                                        style={{ width: '100%', padding: '10px', background: 'rgba(0,0,0,0.2)', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '8px', color: '#fff' }}
                                    />
                                </div>
                                <div>
                                    <label style={{ display: 'block', marginBottom: '8px', opacity: 0.5, fontSize: '12px' }}>Стовпець назви задачі</label>
                                    <input
                                        type="text"
                                        className="premium-input"
                                        value={activeSheet.taskNameColumn}
                                        onChange={e => updateActiveSheet({ taskNameColumn: e.target.value.trim().toUpperCase() })}
                                        style={{ width: '100%', padding: '10px', background: 'rgba(0,0,0,0.2)', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '8px', color: '#fff' }}
                                    />
                                </div>
                                <div>
                                    <label style={{ display: 'block', marginBottom: '8px', opacity: 0.5, fontSize: '12px' }}>Стовпець статусу (напр. G)</label>
                                    <input
                                        type="text"
                                        className="premium-input"
                                        value={activeSheet.statusColumn || ""}
                                        onChange={e => updateActiveSheet({ statusColumn: e.target.value.trim().toUpperCase() })}
                                        style={{ width: '100%', padding: '10px', background: 'rgba(0,0,0,0.2)', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '8px', color: '#fff' }}
                                    />
                                </div>
                                <div>
                                    <label style={{ display: 'block', marginBottom: '8px', opacity: 0.5, fontSize: '12px' }}>Значення статусу (напр. Done)</label>
                                    <input
                                        type="text"
                                        className="premium-input"
                                        value={activeSheet.statusValue || ""}
                                        onChange={e => updateActiveSheet({ statusValue: e.target.value })}
                                        style={{ width: '100%', padding: '10px', background: 'rgba(0,0,0,0.2)', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '8px', color: '#fff' }}
                                    />
                                </div>
                                <div>
                                    <label style={{ display: 'block', marginBottom: '8px', opacity: 0.5, fontSize: '12px' }}>Пропустити рядків (Ignore Rows)</label>
                                    <input
                                        type="number"
                                        className="premium-input"
                                        value={activeSheet.ignoreRows || 0}
                                        onChange={e => updateActiveSheet({ ignoreRows: parseInt(e.target.value) || 0 })}
                                        style={{ width: '100%', padding: '10px', background: 'rgba(0,0,0,0.2)', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '8px', color: '#fff' }}
                                    />
                                </div>
                            </div>

                            {/* Глобальний мапінг */}
                            <div style={{ marginBottom: '25px', padding: '15px', background: 'rgba(255,255,255,0.02)', borderRadius: '12px', border: '1px solid rgba(255,255,255,0.05)' }}>
                                <div style={{ fontSize: '14px', marginBottom: '10px', color: accentColor, fontWeight: 'bold' }}>Глобальний мапінг (для всіх рядків таблиці)</div>
                                <div style={{ display: 'flex', flexWrap: 'wrap', gap: '8px' }}>
                                    {allTemplates.map(tpl => {
                                        const isActive = (activeSheet.globalTemplateIds || []).includes(tpl.id);
                                        return (
                                            <button
                                                key={tpl.id}
                                                onClick={() => handleToggleGlobalTemplate(tpl.id)}
                                                style={{
                                                    padding: '6px 14px',
                                                    borderRadius: '8px',
                                                    fontSize: '12px',
                                                    background: isActive ? accentColor : 'rgba(255,255,255,0.03)',
                                                    border: `1px solid ${isActive ? accentColor : 'rgba(255,255,255,0.1)'}`,
                                                    color: isActive ? '#fff' : '#aaa',
                                                    cursor: 'pointer',
                                                    transition: 'all 0.2s'
                                                }}
                                            >
                                                {tpl.name}
                                            </button>
                                        );
                                    })}
                                </div>
                            </div>

                            {/* Mappings */}
                            <div style={{ marginBottom: '20px' }}>
                                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '15px' }}>
                                    <h4 style={{ margin: 0, opacity: 0.7, fontSize: '15px' }}>Мапінг шаблонів за ключовими словами</h4>
                                    <button onClick={handleAddMapping} style={{ padding: '6px 14px', borderRadius: '8px', cursor: 'pointer', background: accentColor, border: 'none', color: '#fff', fontSize: '12px', fontWeight: 'bold' }}>
                                        + Додати мапінг
                                    </button>
                                </div>

                                <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
                                    {activeSheet.mappings?.map((m, mIdx) => (
                                        <div key={mIdx} style={{ background: 'rgba(255,255,255,0.02)', padding: '15px', borderRadius: '12px', border: '1px solid rgba(255,255,255,0.05)' }}>
                                            <div style={{ display: 'flex', gap: '10px', marginBottom: '12px' }}>
                                                <input
                                                    type="text"
                                                    className="premium-input"
                                                    value={m.keyword}
                                                    onChange={e => handleMappingChange(mIdx, 'keyword', e.target.value)}
                                                    placeholder="Ключове слово у рядку..."
                                                    style={{ flex: 1, padding: '10px', background: 'rgba(0,0,0,0.2)', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '8px', color: '#fff' }}
                                                />
                                                <button onClick={() => handleRemoveMapping(mIdx)} style={{ padding: '0 15px', background: 'rgba(255,77,77,0.05)', border: '1px solid rgba(255,77,77,0.2)', color: '#ff4d4d', borderRadius: '8px', cursor: 'pointer', fontSize: '12px' }}>Видалити</button>
                                            </div>
                                            <div style={{ display: 'flex', flexWrap: 'wrap', gap: '8px' }}>
                                                {allTemplates.map(tpl => {
                                                    const isActive = (m.templateIds || []).includes(tpl.id);
                                                    return (
                                                        <button
                                                            key={tpl.id}
                                                            onClick={() => handleToggleTemplate(mIdx, tpl.id)}
                                                            style={{
                                                                padding: '5px 12px',
                                                                borderRadius: '6px',
                                                                fontSize: '11px',
                                                                background: isActive ? accentColor : 'transparent',
                                                                border: `1px solid ${isActive ? accentColor : 'rgba(255,255,255,0.1)'}`,
                                                                color: isActive ? '#fff' : '#aaa',
                                                                cursor: 'pointer'
                                                            }}
                                                        >
                                                            {tpl.name}
                                                        </button>
                                                    );
                                                })}
                                            </div>
                                        </div>
                                    ))}
                                </div>
                            </div>

                            {/* Actions */}
                            <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end', marginTop: '20px', borderTop: '1px solid rgba(255,255,255,0.05)', paddingTop: '20px' }}>
                                <button
                                    onClick={handleParse}
                                    disabled={isParsing}
                                    style={{ padding: '10px 20px', borderRadius: '10px', background: 'rgba(255,255,255,0.03)', border: '1px solid rgba(255,255,255,0.1)', color: '#fff', cursor: 'pointer', fontSize: '13px' }}
                                >
                                    {isParsing ? 'Парсинг...' : 'Перевірити таблицю'}
                                </button>
                                <button
                                    onClick={handleSave}
                                    style={{ padding: '10px 30px', borderRadius: '10px', background: accentColor, border: 'none', color: '#fff', fontWeight: 'bold', cursor: 'pointer', fontSize: '13px' }}
                                >
                                    Зберегти все
                                </button>
                            </div>
                        </div>

                        {statusMsg && (
                            <div style={{
                                padding: '15px',
                                borderRadius: '12px',
                                background: statusMsg.type === 'success' ? '#4caf5015' : '#ff4d4d15',
                                border: `1px solid ${statusMsg.type === 'success' ? '#4caf5030' : '#ff4d4d30'}`,
                                color: statusMsg.type === 'success' ? '#81c784' : '#e57373',
                                textAlign: 'center',
                                fontSize: '13px'
                            }}>
                                {statusMsg.text}
                            </div>
                        )}
                    </>
                )}
            </div>
        </div>
    );
};
