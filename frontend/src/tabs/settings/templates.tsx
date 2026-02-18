import React, { useState, useMemo, useEffect } from 'react';
import { useI18n } from '../../contexts/I18nContext';
import { useTemplates, PipelineTemplate } from '../../contexts/TemplateContext';
import { useServices } from '../../contexts/ServiceContext';
import { ConfirmModal } from '../../components/ConfirmModal';
// @ts-ignore
import { GetOpenRouterSavedModels, SelectDirectory } from '../../../wailsjs/go/main/App';
import './templates.css';

export const Templates = () => {
    const { t, locale } = useI18n();
    const { templates, removeTemplate, updateTemplate, isLoading } = useTemplates();
    const { openRouterKeys } = useServices();
    const [templateToDelete, setTemplateToDelete] = useState<PipelineTemplate | null>(null);
    const [selectedIds, setSelectedIds] = useState<string[]>([]);
    const [isBulkEditOpen, setIsBulkEditOpen] = useState(false);
    const [models, setModels] = useState<string[]>([]);

    useEffect(() => {
        const fetchModels = async () => {
            const m = await GetOpenRouterSavedModels();
            setModels(m || []);
        };
        fetchModels();
    }, []);

    // Bulk Edit State
    const [bulkParam, setBulkParam] = useState<string>('');
    const [bulkValue, setBulkValue] = useState<any>('');

    const selectedTemplates = useMemo(() =>
        templates.filter(t => selectedIds.includes(t.id)),
        [templates, selectedIds]);

    const canBulkEdit = useMemo(() => {
        if (selectedIds.length === 0) return false;
        const firstType = selectedTemplates[0]?.type;
        return selectedTemplates.every(t => t.type === firstType);
    }, [selectedIds, selectedTemplates]);

    const selectedType = selectedTemplates[0]?.type;

    const handleDelete = async () => {
        if (templateToDelete) {
            await removeTemplate(templateToDelete.id);
            setTemplateToDelete(null);
            setSelectedIds(prev => prev.filter(id => id !== templateToDelete.id));
        }
    };

    const formatDate = (timestamp: number) => {
        return new Date(timestamp * 1000).toLocaleString(locale === 'uk' ? 'uk-UA' : locale === 'ru' ? 'ru-RU' : 'en-US', {
            year: 'numeric',
            month: 'short',
            day: 'numeric',
            hour: '2-digit',
            minute: '2-digit'
        });
    };

    const toggleSelect = (id: string) => {
        setSelectedIds(prev =>
            prev.includes(id) ? prev.filter(i => i !== id) : [...prev, id]
        );
    };

    const toggleSelectAll = () => {
        if (selectedIds.length === templates.length) {
            setSelectedIds([]);
        } else {
            setSelectedIds(templates.map(t => t.id));
        }
    };

    const handleBulkApply = async () => {
        if (!bulkParam || bulkValue === undefined) return;

        for (const tpl of selectedTemplates) {
            const newSettings = {
                ...tpl.settings,
                [bulkParam]: bulkValue
            };

            const cleanSettings: any = {};

            if (tpl.type === 'translate') {
                cleanSettings.translateModel = newSettings.translateModel || "";
                cleanSettings.translatePrompt = newSettings.translatePrompt || "";
                cleanSettings.translateTemperature = newSettings.translateTemperature || 0;
                cleanSettings.translateMaxTokens = newSettings.translateMaxTokens || 0;
                cleanSettings.translateOpenRouterKeyID = newSettings.translateOpenRouterKeyID || "";
                cleanSettings.translateEnabled = newSettings.translateEnabled === undefined ? true : newSettings.translateEnabled;
                cleanSettings.translatePipelineName = newSettings.translatePipelineName || '';
                cleanSettings.translateOutputPath = newSettings.translateOutputPath || '';
            } else {
                cleanSettings.rewriteModel = newSettings.rewriteModel || "";
                cleanSettings.rewritePrompt = newSettings.rewritePrompt || "";
                cleanSettings.rewriteTemperature = newSettings.rewriteTemperature || 0;
                cleanSettings.rewriteMaxTokens = newSettings.rewriteMaxTokens || 0;
                cleanSettings.rewriteOpenRouterKeyID = newSettings.rewriteOpenRouterKeyID || "";
                cleanSettings.rewriteEnabled = newSettings.rewriteEnabled === undefined ? true : newSettings.rewriteEnabled;
                cleanSettings.rewritePipelineName = newSettings.rewritePipelineName || '';
                cleanSettings.rewriteOutputPath = newSettings.rewriteOutputPath || '';
            }

            await updateTemplate(tpl.id, tpl.name, cleanSettings);
        }
        setIsBulkEditOpen(false);
        setBulkParam('');
        setBulkValue('');
    };

    const getEditableParams = () => {
        if (!selectedType) return [];
        const prefix = selectedType;
        return [
            { id: `${prefix}Model`, label: t('pipeline.model') },
            { id: `${prefix}Temperature`, label: t('pipeline.temperature') },
            { id: `${prefix}MaxTokens`, label: t('pipeline.max_tokens') },
            { id: `${prefix}Prompt`, label: t('pipeline.system_prompt') },
            { id: `${prefix}OpenRouterKeyID`, label: t('pipeline.group.api') },
            { id: `${prefix}OutputPath`, label: t('pipeline.group.path') },
        ];
    };

    return (
        <div className={`templates-page ${isBulkEditOpen ? 'has-panel' : ''}`}>
            <div className="content-wrapper">
                <div className="settings-container">
                    <div className="settings-header-group">
                        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
                            <div>
                                <h2 className="settings-title">{t('settings.templates')}</h2>
                                <p className="settings-description">{t('templatesTab.description')}</p>
                            </div>
                            {selectedIds.length > 0 && (
                                <div className="bulk-actions">
                                    <span className="selected-count">
                                        {t('templatesTab.selected_count', { count: selectedIds.length })}
                                    </span>
                                    <button
                                        className={`bulk-edit-trigger ${!canBulkEdit ? 'disabled' : ''}`}
                                        onClick={() => canBulkEdit && setIsBulkEditOpen(true)}
                                        disabled={!canBulkEdit}
                                    >
                                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path></svg>
                                        {t('templatesTab.bulk_edit')}
                                    </button>
                                </div>
                            )}
                        </div>
                    </div>

                    {isLoading ? (
                        <div className="loading-templates">
                            <div className="loader"></div>
                        </div>
                    ) : templates.length === 0 ? (
                        <div className="empty-templates">
                            <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1" strokeLinecap="round" strokeLinejoin="round" opacity="0.3">
                                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
                            </svg>
                            <p>{t('pipeline.no_templates')}</p>
                        </div>
                    ) : (
                        <div className="templates-list-view">
                            <div className="list-header">
                                <div className="col-check">
                                    <input
                                        type="checkbox"
                                        checked={selectedIds.length === templates.length && templates.length > 0}
                                        onChange={toggleSelectAll}
                                    />
                                </div>
                                <div className="col-name">{t('pipeline.name')}</div>
                                <div className="col-type">{t('templatesTab.type')}</div>
                                <div className="col-date">{t('templatesTab.created_at')}</div>
                                <div className="col-actions"></div>
                            </div>
                            <div className="list-body">
                                {templates.map(tpl => (
                                    <div
                                        key={tpl.id}
                                        className={`list-item ${selectedIds.includes(tpl.id) ? 'selected' : ''}`}
                                        onClick={() => toggleSelect(tpl.id)}
                                    >
                                        <div className="col-check">
                                            <input
                                                type="checkbox"
                                                checked={selectedIds.includes(tpl.id)}
                                                onChange={() => { }} // Handled by row click
                                            />
                                        </div>
                                        <div className="col-name">
                                            <span className="item-name">{tpl.name}</span>
                                        </div>
                                        <div className="col-type">
                                            <span className={`type-tag ${tpl.type}`}>
                                                {tpl.type === 'translate' ? t('text.translate') : t('text.rewrite')}
                                            </span>
                                        </div>
                                        <div className="col-date">
                                            {formatDate(tpl.createdAt)}
                                        </div>
                                        <div className="col-actions">
                                            <button
                                                className="item-delete-btn"
                                                onClick={(e) => {
                                                    e.stopPropagation();
                                                    setTemplateToDelete(tpl);
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

            {/* Bulk Editor Panel */}
            <div className={`bulk-edit-panel ${isBulkEditOpen ? 'open' : ''}`}>
                <div className="panel-header">
                    <h3>{t('templatesTab.bulk_edit_title')}</h3>
                    <button className="panel-close" onClick={() => setIsBulkEditOpen(false)}>
                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
                    </button>
                </div>
                <div className="panel-body">
                    <div className="panel-info">
                        {t('templatesTab.selected_count', { count: selectedIds.length })}
                    </div>

                    <div className="panel-group">
                        <label className="panel-label">{t('templatesTab.select_param')}</label>
                        <select
                            className="panel-select"
                            value={bulkParam}
                            onChange={(e) => {
                                setBulkParam(e.target.value);
                                setBulkValue('');
                            }}
                        >
                            <option value="">-- {t('templatesTab.select_param')} --</option>
                            {getEditableParams().map(p => (
                                <option key={p.id} value={p.id}>{p.label}</option>
                            ))}
                        </select>
                    </div>

                    {bulkParam && (
                        <div className="panel-group">
                            <label className="panel-label">{t('common.value') || 'Value'}</label>
                            {bulkParam.includes('Model') ? (
                                <select
                                    className="panel-select"
                                    value={bulkValue}
                                    onChange={(e) => setBulkValue(e.target.value)}
                                >
                                    <option value="">-- {t('pipeline.model')} --</option>
                                    {models.map(m => (
                                        <option key={m} value={m}>{m}</option>
                                    ))}
                                </select>
                            ) : bulkParam.includes('Temperature') ? (
                                <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                                    <input
                                        type="range"
                                        min="0"
                                        max="2"
                                        step="0.1"
                                        value={bulkValue || 0.7}
                                        onChange={(e) => setBulkValue(parseFloat(e.target.value))}
                                        className="panel-slider"
                                    />
                                    <span className="slider-value">{bulkValue || 0.7}</span>
                                </div>
                            ) : bulkParam.includes('MaxTokens') ? (
                                <input
                                    type="number"
                                    className="panel-input"
                                    value={bulkValue}
                                    onChange={(e) => setBulkValue(parseInt(e.target.value))}
                                />
                            ) : bulkParam.includes('Prompt') ? (
                                <textarea
                                    className="panel-textarea"
                                    value={bulkValue}
                                    onChange={(e) => setBulkValue(e.target.value)}
                                    rows={5}
                                />
                            ) : bulkParam.includes('OpenRouterKeyID') ? (
                                <select
                                    className="panel-select"
                                    value={bulkValue}
                                    onChange={(e) => setBulkValue(e.target.value)}
                                >
                                    <option value="">{t('api.openrouterSettings.noKeys')}</option>
                                    {openRouterKeys.map(k => (
                                        <option key={k.id} value={k.id}>{k.name}</option>
                                    ))}
                                </select>
                            ) : bulkParam.includes('OutputPath') ? (
                                <div style={{ display: 'flex', gap: '8px' }}>
                                    <input
                                        className="panel-input"
                                        value={bulkValue}
                                        readOnly
                                        placeholder={t('pipeline.select_path') || 'Select path...'}
                                        style={{ flex: 1 }}
                                    />
                                    <button
                                        className="panel-button"
                                        onClick={async () => {
                                            const path = await SelectDirectory();
                                            if (path) setBulkValue(path);
                                        }}
                                        style={{ padding: '8px 12px', background: 'var(--accent-primary)', border: 'none', borderRadius: '4px', cursor: 'pointer' }}
                                    >
                                        ...
                                    </button>
                                </div>
                            ) : null}
                        </div>
                    )}
                </div>
                <div className="panel-footer">
                    <button className="panel-cancel" onClick={() => setIsBulkEditOpen(false)}>
                        {t('common.cancel')}
                    </button>
                    <button
                        className="panel-apply"
                        onClick={handleBulkApply}
                        disabled={!bulkParam}
                    >
                        {t('templatesTab.apply_to_selected')}
                    </button>
                </div>
            </div>

            <ConfirmModal
                isOpen={!!templateToDelete}
                title={t('common.delete')}
                message={`${t('templatesTab.delete_confirm')} "${templateToDelete?.name}"`}
                onConfirm={handleDelete}
                onClose={() => setTemplateToDelete(null)}
                confirmText={t('common.delete')}
                cancelText={t('common.cancel')}
                isDanger={true}
            />
        </div>
    );
};
