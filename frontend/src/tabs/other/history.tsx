import React, { useState, useEffect, useCallback } from 'react';
import { useI18n } from '../../contexts/I18nContext';
// @ts-ignore
import { GetFullHistory, GetFullHistoryEntry, DeleteFullHistoryEntry } from '../../../wailsjs/go/main/App';
import { EventsOn } from '../../../wailsjs/runtime/runtime';
import { ConfirmModal } from '../../components/ConfirmModal';
import './history.css';

interface HistoryMetadata {
    id: string;
    taskName: string;
    type: string;
    templates: string[];
    stages: string[];
    timestamp: number;
    formattedDate: string;
}

interface FullHistoryEntry extends HistoryMetadata {
    originalText: string;
    processedText: string;
}

export const History = () => {
    const { t, locale } = useI18n();
    const [historyList, setHistoryList] = useState<HistoryMetadata[]>([]);
    const [selectedEntry, setSelectedEntry] = useState<FullHistoryEntry | null>(null);
    const [isLoading, setIsLoading] = useState(true);
    const [isLoadingDetail, setIsLoadingDetail] = useState(false);
    const [isPanelOpen, setIsPanelOpen] = useState(false);
    const [entryToDelete, setEntryToDelete] = useState<HistoryMetadata | null>(null);

    const loadHistory = useCallback(async () => {
        setIsLoading(true);
        try {
            const data = await GetFullHistory();
            setHistoryList(data || []);
        } catch (err) {
            console.error("Failed to load full history:", err);
        } finally {
            setIsLoading(false);
        }
    }, []);

    useEffect(() => {
        loadHistory();

        // Listen for updates from backend
        // @ts-ignore
        const unsub = EventsOn("fullHistoryUpdate", () => {
            loadHistory();
        });

        return () => {
            if (unsub) unsub();
        };
    }, [loadHistory]);

    const handleViewDetail = async (metadata: HistoryMetadata) => {
        setIsLoadingDetail(true);
        setIsPanelOpen(true);
        try {
            const entry = await GetFullHistoryEntry(metadata.id);
            setSelectedEntry(entry);
        } catch (err) {
            console.error("Failed to load history detail:", err);
        } finally {
            setIsLoadingDetail(false);
        }
    };

    const handleDelete = async () => {
        if (!entryToDelete) return;
        try {
            await DeleteFullHistoryEntry(entryToDelete.id);
            setHistoryList(prev => prev.filter(e => e.id !== entryToDelete.id));
            if (selectedEntry?.id === entryToDelete.id) {
                setIsPanelOpen(false);
                setSelectedEntry(null);
            }
            setEntryToDelete(null);
        } catch (err) {
            console.error("Failed to delete history entry:", err);
        }
    };

    const formatDate = (dateStr: string) => {
        try {
            const date = new Date(dateStr);
            return date.toLocaleString(locale === 'uk' ? 'uk-UA' : locale === 'ru' ? 'ru-RU' : 'en-US', {
                year: 'numeric',
                month: 'short',
                day: 'numeric',
                hour: '2-digit',
                minute: '2-digit'
            });
        } catch (e) {
            return dateStr;
        }
    };

    return (
        <div className={`history-page ${isPanelOpen ? 'has-panel' : ''}`}>
            <div className="content-wrapper animate-fade">
                <div className="settings-container">
                    <div className="settings-header-group">
                        <h2 className="settings-title">{t('other.history')}</h2>
                        <p className="settings-description">{t('historyTab.description') || 'Long-term history of your completed tasks (30 days).'}</p>
                    </div>

                    {isLoading ? (
                        <div className="loading-history">
                            <div className="spinner-small" style={{ width: '24px', height: '24px' }}></div>
                        </div>
                    ) : historyList.length === 0 ? (
                        <div className="empty-history">
                            <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1" strokeLinecap="round" strokeLinejoin="round" opacity="0.3">
                                <path d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                            </svg>
                            <p>{t('historyTab.empty') || 'No history entries yet.'}</p>
                        </div>
                    ) : (
                        <div className="history-list-view">
                            <div className="history-list-header">
                                <div className="col-name">{t('pipeline.name')}</div>
                                <div className="col-type">{t('templatesTab.type')}</div>
                                <div className="col-date">{t('templatesTab.created_at')}</div>
                                <div className="col-actions"></div>
                            </div>
                            <div className="history-list-body premium-scrollbar">
                                {historyList.map(entry => (
                                    <div
                                        key={entry.id}
                                        className={`history-list-item ${selectedEntry?.id === entry.id ? 'selected' : ''}`}
                                        onClick={() => handleViewDetail(entry)}
                                    >
                                        <div className="col-name">
                                            <span className="history-item-name">{entry.taskName}</span>
                                            {entry.templates && entry.templates.length > 0 && (
                                                <span className="history-item-template">{entry.templates[0]}</span>
                                            )}
                                        </div>
                                        <div className="col-type">
                                            <span className={`type-tag ${entry.type}`}>
                                                {entry.type === 'translate' ? t('text.translate') : (entry.type === 'rewrite' ? t('text.rewrite') : t('text.voiceover'))}
                                            </span>
                                        </div>
                                        <div className="col-date history-col-date">
                                            {formatDate(entry.formattedDate)}
                                        </div>
                                        <div className="col-actions">
                                            <button
                                                className="history-item-delete-btn"
                                                onClick={(e) => {
                                                    e.stopPropagation();
                                                    setEntryToDelete(entry);
                                                }}
                                                title={t('common.delete')}
                                            >
                                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path><line x1="10" y1="11" x2="10" y2="17"></line><line x1="14" y1="11" x2="14" y2="17"></line></svg>
                                            </button>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        </div>
                    )}
                </div>
            </div>

            {/* Detail Panel */}
            <div className={`history-detail-panel ${isPanelOpen ? 'open' : ''}`}>
                <div className="h-panel-header">
                    <h3>{t('historyTab.details')}</h3>
                    <button className="h-panel-close" onClick={() => setIsPanelOpen(false)}>
                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
                    </button>
                </div>
                <div className="h-panel-body premium-scrollbar">
                    {isLoadingDetail ? (
                        <div className="loading-history">
                            <div className="spinner-small" style={{ width: '24px', height: '24px' }}></div>
                        </div>
                    ) : selectedEntry ? (
                        <>
                            <div className="h-panel-section">
                                <span className="h-panel-label">{t('pipeline.name')}</span>
                                <div style={{ fontSize: '15px', fontWeight: 600 }}>{selectedEntry.taskName}</div>
                            </div>

                            <div className="h-panel-section">
                                <span className="h-panel-label">{t('templatesTab.type')}</span>
                                <span className={`type-tag ${selectedEntry.type}`} style={{ width: 'fit-content' }}>
                                    {selectedEntry.type === 'translate' ? t('text.translate') : (selectedEntry.type === 'rewrite' ? t('text.rewrite') : t('text.voiceover'))}
                                </span>
                            </div>

                            {selectedEntry.templates && selectedEntry.templates.length > 0 && (
                                <div className="h-panel-section">
                                    <span className="h-panel-label">{t('sidebar.templates') || 'Templates'}</span>
                                    <div style={{ display: 'flex', gap: '4px', flexWrap: 'wrap' }}>
                                        {selectedEntry.templates.map(tpl => (
                                            <span key={tpl} style={{ fontSize: '12px', background: 'var(--bg-tertiary)', padding: '2px 8px', borderRadius: '4px', border: '1px solid var(--border-color)' }}>
                                                {tpl}
                                            </span>
                                        ))}
                                    </div>
                                </div>
                            )}

                            <div className="h-panel-section">
                                <span className="h-panel-label">{t('historyTab.stages') || 'Executed Stages'}</span>
                                <div className="h-stages-list">
                                    {['translate', 'rewrite', 'voiceover', 'image', 'subtitles', 'montage'].map(stage => {
                                        const isActive = selectedEntry.stages?.includes(stage);
                                        return (
                                            <div key={stage} className={`h-stage-tag ${isActive ? 'active' : ''}`}>
                                                <div className="h-stage-icon"></div>
                                                {t(`stages.${stage}`) || stage}
                                            </div>
                                        );
                                    })}
                                </div>
                            </div>

                            <div className="h-panel-section">
                                <span className="h-panel-label">{t('historyTab.original')}</span>
                                <div className="h-text-container premium-scrollbar">
                                    {selectedEntry.originalText || (<i>{t('common.no_data') || 'No data'}</i>)}
                                </div>
                            </div>

                            <div className="h-panel-section">
                                <span className="h-panel-label">{t('historyTab.processed')}</span>
                                <div className="h-text-container premium-scrollbar">
                                    {selectedEntry.processedText || (<i>{t('common.no_data') || 'No data'}</i>)}
                                </div>
                            </div>
                        </>
                    ) : null}
                </div>
            </div>

            <ConfirmModal
                isOpen={!!entryToDelete}
                title={t('common.delete')}
                message={`${t('historyTab.delete_confirm') || 'Are you sure you want to delete this history entry?'} "${entryToDelete?.taskName}"`}
                onConfirm={handleDelete}
                onClose={() => setEntryToDelete(null)}
                confirmText={t('common.delete')}
                cancelText={t('common.cancel')}
                isDanger={true}
            />
        </div>
    );
};
