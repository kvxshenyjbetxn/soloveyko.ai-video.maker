import React, { createContext, useContext, useState, useEffect, useCallback } from 'react';
import { useI18n } from './I18nContext';
import { useTemplates } from './TemplateContext';
import { useQueueActions } from './QueueContext';
import { useToast } from './ToastContext';
// @ts-ignore
import { ParseAllGoogleSheets, GetGoogleSheets, FetchGoogleDocContent, AddToHistory, UpdateGoogleSheetStatus } from '../../wailsjs/go/main/App';

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

interface MultiSheetResult {
    id: string;
    name: string;
    results: any[];
    error?: string;
}

interface GoogleMonitorContextType {
    isParsing: boolean;
    sheetResults: MultiSheetResult[];
    activeSheetId: string | null;
    setActiveSheetId: (id: string | null) => void;
    scanSheets: () => Promise<MultiSheetResult[]>;
    clearResults: () => void;
    handleCreateTask: (sheetId: string, rowIndex: number) => Promise<{ count: number; taskNames: string[] }>;
    fetchContentIfNeeded: (sheetId: string, rowIndex: number) => Promise<string>;
    loadingItemId: string | null;
    sheetsConfig: GoogleSheetConfig[];
}

const GoogleMonitorContext = createContext<GoogleMonitorContextType | undefined>(undefined);

export const GoogleMonitorProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
    const { t } = useI18n();
    const { templates, flattenSettings } = useTemplates();
    const { addTask, getNextTaskName } = useQueueActions();
    const { showToast } = useToast();

    const [isParsing, setIsParsing] = useState(false);
    const [sheetsConfig, setSheetsConfig] = useState<GoogleSheetConfig[]>([]);
    const [sheetResults, setSheetResults] = useState<MultiSheetResult[]>([]);
    const [activeSheetId, setActiveSheetId] = useState<string | null>(null);
    const [lastUpdate, setLastUpdate] = useState<Date | null>(null);
    const [loadingItemId, setLoadingItemId] = useState<string | null>(null);

    const loadSheetsConfig = useCallback(async () => {
        try {
            const s = await GetGoogleSheets();
            setSheetsConfig(s || []);
            if (s && s.length > 0 && !activeSheetId) {
                setActiveSheetId(s[0].id || 'default');
            }
        } catch (err) {
            console.error("Monitor configuration load fail", err);
        }
    }, [activeSheetId]);

    // Initial load of config
    useEffect(() => {
        loadSheetsConfig();
    }, []);

    const scanSheets = async (): Promise<MultiSheetResult[]> => {
        setIsParsing(true);
        try {
            const data = await ParseAllGoogleSheets();
            setSheetResults(data || []);
            setLastUpdate(new Date());
            
            // Update active sheet if not set or not in results
            if (data && data.length > 0) {
                if (!activeSheetId || !data.find(s => s.id === activeSheetId)) {
                    setActiveSheetId(data[0].id);
                }
            }
            
            return data || [];
        } catch (err: any) {
            showToast(t('google_monitor.parse_error') || "Parsing Error", 'error' as any);
            throw err;
        } finally {
            setIsParsing(false);
        }
    };

    const clearResults = () => {
        setSheetResults([]);
        setLastUpdate(null);
        showToast(t('google_monitor.cleared') || "Cleared Monitor Data", 'info' as any);
    };

    const fetchContentIfNeeded = async (sheetId: string, idx: number): Promise<string> => {
        const sheet = sheetResults.find(s => s.id === sheetId);
        const item = sheet?.results[idx];
        if (!item) return "";

        let content = item.content;
        const itemId = `${sheetId}-${idx}`;

        if (!content && item.docLink) {
            setLoadingItemId(itemId);
            try {
                content = await FetchGoogleDocContent(item.docLink);
                // Update local state to keep content
                setSheetResults(prev => prev.map(s => s.id === sheetId ? {
                    ...s,
                    results: s.results.map((r, i) => i === idx ? { ...r, content: content } : r)
                } : s));
            } catch (err) {
                showToast("Помилка завантаження документа", 'error');
                setLoadingItemId(null);
                throw err;
            } finally {
                setLoadingItemId(null);
            }
        }
        return content || "";
    };

    const handleCreateTask = async (sheetId: string, idx: number): Promise<{ count: number; taskNames: string[] }> => {
        const sheetConfig = sheetsConfig.find(s => s.id === sheetId);
        const sheet = sheetResults.find(s => s.id === sheetId);
        if (!sheetConfig) throw new Error(`Конфігурація таблиці не знайдена: ${sheetId}`);
        if (!sheet) throw new Error(`Дані таблиці не знайдені: ${sheetId}`);
        const item = sheet.results[idx];
        if (!item) throw new Error(`Рядок з індексом ${idx} не знайдено у таблиці.`);

        try {
            const content = await fetchContentIfNeeded(sheetId, idx);

            const mapping = (sheetConfig.mappings || []).find(m => {
                if (!m.keyword) return false;
                const kw = m.keyword.toLowerCase();
                return item.columns?.some((c: string) => c?.toLowerCase().includes(kw)) || item.title?.toLowerCase().includes(kw);
            });

            const globalIds = sheetConfig.globalTemplateIds || [];
            const mappingIds = mapping?.templateIds || [];
            const allTemplateIds = Array.from(new Set([...globalIds, ...mappingIds]));

            if (allTemplateIds.length === 0) {
                const msg = t('google_monitor.no_mapping') || "Не налаштовано мапінг шаблонів для цього рядка";
                showToast(msg, 'error' as any);
                throw new Error(msg);
            }

            let taskNameBase = getNextTaskName();
            if (sheetConfig.taskNameColumn) {
                const colIdx = sheetConfig.taskNameColumn.toUpperCase().split('').reduce((acc, char) => acc * 26 + (char.charCodeAt(0) - 64), 0) - 1;
                const customName = item.columns?.[colIdx];
                if (customName) taskNameBase = customName;
            }

            const templatesToApply = allTemplateIds.map(id => templates.find(t => t.id === id)).filter(Boolean);
            const templateNames = templatesToApply.map((t: any) => t.name);

            if (templatesToApply.length > 0) {
                // Record to history
                try {
                    await AddToHistory(taskNameBase, 'translate', templateNames, content);
                } catch (err) {
                    console.error("Failed to add to history:", err);
                }

                templatesToApply.forEach((template: any) => {
                    const activeSettings = flattenSettings(JSON.parse(JSON.stringify(template.settings)));
                    addTask('translate' as any, content, activeSettings, taskNameBase, template.name);
                });

                // Update status in Google Sheets if configured
                if (sheetConfig.statusColumn && sheetConfig.statusValue) {
                    try {
                        await UpdateGoogleSheetStatus(sheetConfig.url, item.index, sheetConfig.statusColumn, sheetConfig.statusValue);
                        showToast(t('google_monitor.status_updated') || "Статус у таблиці оновлено", 'success');
                    } catch (err: any) {
                        console.error("Failed to update status in Google Sheets:", err);
                        showToast(`${t('google_monitor.status_error') || "Помилка оновлення статусу"}: ${err?.message || err}`, 'error');
                    }
                }

                showToast(t('google_monitor.created', { count: templatesToApply.length }), 'success');
                return { 
                    count: templatesToApply.length, 
                    taskNames: templatesToApply.map((tmpl: any) => tmpl.name ? `${taskNameBase} - ${tmpl.name}` : taskNameBase) 
                };
            }
            return { count: 0, taskNames: [] };
        } catch (err: any) {
            console.error("Failed to create task from monitor:", err);
            throw err;
        }
    };

    return (
        <GoogleMonitorContext.Provider value={{
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
        }}>
            {children}
        </GoogleMonitorContext.Provider>
    );
};

export const useGoogleMonitor = () => {
    const context = useContext(GoogleMonitorContext);
    if (!context) {
        throw new Error('useGoogleMonitor must be used within a GoogleMonitorProvider');
    }
    return context;
};
