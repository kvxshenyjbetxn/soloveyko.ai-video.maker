import React, { useState, useMemo, useEffect } from 'react';
import { useI18n } from '../../contexts/I18nContext';
import { useTemplates, PipelineTemplate } from '../../contexts/TemplateContext';
import { useServices } from '../../contexts/ServiceContext';
import { ConfirmModal } from '../../components/ConfirmModal';
// @ts-ignore
import { GetOpenRouterSavedModels, SelectDirectory, SelectVideo, SelectImage, GetPollinationsSavedModels, GetElevenLabsBotVoiceTemplates, GetVoiceMakerVoices, GetPipelineSettings, GetOpenRouterKeys, GetElevenLabsBotKeys, GetElevenLabsUAKeys, GetElevenLabsUnlimKeys, GetVoiceMakerKeys, GetPollinationsKeys, GetElevenLabsImageKeys, GetEdgeTTSVoices } from '../../../wailsjs/go/main/App';
import { MASS_EDITOR_BLOCKS, MassEditorSetting } from './MassEditorData';
import voicemakerVoicesData from '../../assets/voicemaker_voices.json';
import './templates.css';

export const Templates = () => {
    const { t, locale } = useI18n();
    const { templates, removeTemplate, updateTemplate, isLoading } = useTemplates();
    const { openRouterKeys } = useServices();
    const [templateToDelete, setTemplateToDelete] = useState<PipelineTemplate | null>(null);
    const [selectedIds, setSelectedIds] = useState<string[]>([]);
    const [isBulkEditOpen, setIsBulkEditOpen] = useState(false);
    const [models, setModels] = useState<string[]>([]);
    const [pollinationsModels, setPollinationsModels] = useState<string[]>([]);
    const [voiceTemplates, setVoiceTemplates] = useState<string[]>([]);
    const [voiceMakerVoices, setVoiceMakerVoices] = useState<any[]>([]);
    const [edgeTTSVoices, setEdgeTTSVoices] = useState<any[]>([]);

    const normalizeVoices = (data: any[]) => {
        if (!data || data.length === 0) return [];
        // Handle cases where data might be nested under 'default'
        const arrayData = (data as any).default || (Array.isArray(data) ? data : []);

        if (arrayData.length > 0 && arrayData[0].Voices && Array.isArray(arrayData[0].Voices)) {
            const flat: any[] = [];
            arrayData.forEach((langGroup: any) => {
                langGroup.Voices.forEach((voiceId: string) => {
                    flat.push({
                        VoiceId: voiceId,
                        LanguageName: langGroup.Language,
                        LanguageCode: langGroup.LanguageCode || 'multi-lang',
                        VoiceWebname: voiceId.split('-').pop() || voiceId,
                    });
                });
            });
            return flat;
        }
        return Array.isArray(arrayData) ? arrayData : [];
    };

    // Key Lists
    const [orKeys, setOrKeys] = useState<any[]>([]);
    const [elBotKeys, setElBotKeys] = useState<any[]>([]);
    const [elUAKeys, setElUAKeys] = useState<any[]>([]);
    const [elUnlimKeys, setElUnlimKeys] = useState<any[]>([]);
    const [vmKeys, setVmKeys] = useState<any[]>([]);
    const [polKeys, setPolKeys] = useState<any[]>([]);
    const [elImgKeys, setElImgKeys] = useState<any[]>([]);

    useEffect(() => {
        if (isBulkEditOpen) {
            document.documentElement.style.setProperty('--pipeline-sidebar-width', '380px');
        } else {
            document.documentElement.style.setProperty('--pipeline-sidebar-width', '0px');
        }
        return () => {
            document.documentElement.style.setProperty('--pipeline-sidebar-width', '0px');
        };
    }, [isBulkEditOpen]);

    useEffect(() => {
        const fetchData = async () => {
            const or = await GetOpenRouterSavedModels();
            setModels(or || []);

            const pol = await GetPollinationsSavedModels();
            setPollinationsModels(pol || []);

            // Keys
            setOrKeys(await GetOpenRouterKeys() || []);
            setElBotKeys(await GetElevenLabsBotKeys() || []);
            setElUAKeys(await GetElevenLabsUAKeys() || []);
            setElUnlimKeys(await GetElevenLabsUnlimKeys() || []);
            setVmKeys(await GetVoiceMakerKeys() || []);
            setPolKeys(await GetPollinationsKeys() || []);
            setElImgKeys(await GetElevenLabsImageKeys() || []);

            // For ElevenLabs Bot, we need to find the active key first
            const settings = await GetPipelineSettings();
            if (settings?.voiceoverElevenLabsBotKeyID) {
                const keyObj = (await GetElevenLabsBotKeys())?.find((k: any) => k.id === settings.voiceoverElevenLabsBotKeyID);
                if (keyObj) {
                    const vt = await GetElevenLabsBotVoiceTemplates(keyObj.key);
                    setVoiceTemplates(vt || []);
                }
            }

            // For VoiceMaker
            const initialVoices = normalizeVoices(voicemakerVoicesData || []);
            setVoiceMakerVoices(initialVoices);

            if (settings?.voiceoverVoiceMakerKeyID) {
                const keys = await (window as any).go.main.App.GetVoiceMakerKeys();
                const keyObj = keys?.find((k: any) => k.id === settings.voiceoverVoiceMakerKeyID);
                if (keyObj) {
                    const vv = await GetVoiceMakerVoices(keyObj.key);
                    if (vv && vv.length > 0) {
                        setVoiceMakerVoices(normalizeVoices(vv));
                    }
                }
            }

            // EdgeTTS
            try {
                const ev = await GetEdgeTTSVoices();
                setEdgeTTSVoices(ev || []);
            } catch (e) { }
        };
        fetchData();
    }, []);

    // Bulk Edit State
    const [selectedBlockId, setSelectedBlockId] = useState<string>('');
    const [bulkParam, setBulkParam] = useState<string>('');
    const [bulkValue, setBulkValue] = useState<any>('');

    const selectedTemplates = useMemo(() =>
        templates.filter(t => selectedIds.includes(t.id)),
        [templates, selectedIds]);

    const canBulkEdit = useMemo(() => {
        return selectedIds.length > 0;
    }, [selectedIds]);

    const selectedBlock = useMemo(() =>
        MASS_EDITOR_BLOCKS.find(b => b.id === selectedBlockId),
        [selectedBlockId]);

    const selectedSetting = useMemo(() =>
        selectedBlock?.settings.find(s => s.id === bulkParam),
        [selectedBlock, bulkParam]);

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

    const ALL_SETTINGS = useMemo(() => {
        return MASS_EDITOR_BLOCKS.flatMap(b => b.settings);
    }, []);

    const handleBulkApply = async () => {
        if (!bulkParam || bulkValue === undefined) return;

        const setDeep = (obj: any, path: string, value: any) => {
            const parts = path.split('.');
            let current = obj;
            for (let i = 0; i < parts.length - 1; i++) {
                if (!current[parts[i]]) current[parts[i]] = {};
                current = current[parts[i]];
            }
            current[parts[parts.length - 1]] = value;
        };

        for (const tpl of selectedTemplates) {
            const newSettings = { ...tpl.settings };
            const setting = ALL_SETTINGS.find(s => (s as any).id === bulkParam);
            const path = (setting as any)?.path;

            let finalValue = bulkValue;

            // EdgeTTS Formatting
            if (bulkParam === 'edgeTTSRate') {
                finalValue = (bulkValue >= 0 ? "+" : "") + bulkValue + "%";
            } else if (bulkParam === 'edgeTTSPitch') {
                finalValue = (bulkValue >= 0 ? "+" : "") + bulkValue + "Hz";
            } else if (bulkParam === 'edgeTTSVolume') {
                finalValue = (bulkValue >= 0 ? "+" : "") + bulkValue + "%";
            }

            if (path) {
                setDeep(newSettings, path, finalValue);
            } else {
                newSettings[bulkParam] = finalValue;
            }

            // Sync global output path to both pipelines
            if (bulkParam === 'outputPath') {
                newSettings.translateOutputPath = finalValue;
                newSettings.rewriteOutputPath = finalValue;
            }

            await updateTemplate(tpl.id, tpl.name, newSettings);
        }
        setIsBulkEditOpen(false);
        setBulkParam('');
        setBulkValue('');
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
                    ) : templates && templates.length === 0 ? (
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
                                                {tpl.type === 'translate' ? t('text.translate') : (tpl.type === 'rewrite' ? t('text.rewrite') : t('text.voiceover'))}
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

                    {/* Block Selection */}
                    <div className="panel-group">
                        <label className="panel-label">{t('common.stage') || 'Блок'}</label>
                        <select
                            className="panel-select"
                            value={selectedBlockId}
                            onChange={(e) => {
                                setSelectedBlockId(e.target.value);
                                setBulkParam('');
                                setBulkValue('');
                            }}
                        >
                            <option value="">-- {t('common.select') || 'Select'} --</option>
                            {MASS_EDITOR_BLOCKS.map(block => (
                                <option key={block.id} value={block.id}>{t(block.labelKey)}</option>
                            ))}
                        </select>
                    </div>

                    {/* Parameter Selection */}
                    {selectedBlock && (
                        <div className="panel-group">
                            <label className="panel-label">{t('templatesTab.select_param')}</label>
                            <select
                                className="panel-select"
                                value={bulkParam}
                                onChange={(e) => {
                                    const paramId = e.target.value;
                                    setBulkParam(paramId);
                                    const setting = selectedBlock.settings.find(s => s.id === paramId);
                                    if (setting?.type === 'switch') {
                                        setBulkValue(false);
                                    } else {
                                        setBulkValue('');
                                    }
                                }}
                            >
                                <option value="">-- {t('templatesTab.select_param')} --</option>
                                {selectedBlock.settings.map(p => (
                                    <option key={p.id} value={p.id}>{t(p.labelKey) || p.labelKey}</option>
                                ))}
                            </select>
                        </div>
                    )}

                    {/* Value Selection */}
                    {selectedSetting && (
                        <div className="panel-group">
                            <label className="panel-label">{t('common.value') || 'Value'}</label>
                            {selectedSetting.type === 'select' ? (
                                <select
                                    className="panel-select"
                                    value={bulkValue}
                                    onChange={(e) => setBulkValue(e.target.value)}
                                >
                                    <option value="">-- {t('common.select') || 'Select'} --</option>
                                    {selectedSetting.options?.map(opt => (
                                        <option key={opt.value} value={opt.value}>
                                            {t(opt.label) || opt.label}
                                        </option>
                                    ))}
                                    {/* Handle Dynamic Models */}
                                    {(selectedSetting as MassEditorSetting).dynamicModels === 'openrouter' && models.map(m => (
                                        <option key={`or-${m}`} value={m}>{m}</option>
                                    ))}
                                    {(selectedSetting as MassEditorSetting).dynamicModels === 'pollinations' && pollinationsModels.map(m => (
                                        <option key={`pol-${m}`} value={m}>{m}</option>
                                    ))}
                                    {(selectedSetting as MassEditorSetting).dynamicModels === 'elevenlabsbot' && voiceTemplates.map(m => (
                                        <option key={`el-${m}`} value={m}>{m}</option>
                                    ))}
                                    {(selectedSetting as MassEditorSetting).dynamicModels === 'voicemaker' && voiceMakerVoices.map(v => (
                                        <option key={`vm-${v.VoiceId}`} value={v.VoiceId}>
                                            {v.VoiceWebname} ({v.LanguageName})
                                        </option>
                                    ))}
                                    {(selectedSetting as MassEditorSetting).dynamicModels === 'edgetts' && edgeTTSVoices.map(m => (
                                        <option key={`edge-${m.ShortName}`} value={m.ShortName}>{m.FriendlyName || m.ShortName}</option>
                                    ))}

                                    {/* Dynamic Keys */}
                                    {(selectedSetting as MassEditorSetting).dynamicKeys === 'openrouter' && orKeys.map(k => (
                                        <option key={`orkey-${k.id}`} value={k.id}>{k.name}</option>
                                    ))}
                                    {(selectedSetting as MassEditorSetting).dynamicKeys === 'elevenlabsbot' && elBotKeys.map(k => (
                                        <option key={`elbotkey-${k.id}`} value={k.id}>{k.name}</option>
                                    ))}
                                    {(selectedSetting as MassEditorSetting).dynamicKeys === 'elevenlabsua' && elUAKeys.map(k => (
                                        <option key={`eluakey-${k.id}`} value={k.id}>{k.name}</option>
                                    ))}
                                    {(selectedSetting as MassEditorSetting).dynamicKeys === 'elevenlabsunlim' && elUnlimKeys.map(k => (
                                        <option key={`elunlimkey-${k.id}`} value={k.id}>{k.name}</option>
                                    ))}
                                    {(selectedSetting as MassEditorSetting).dynamicKeys === 'voicemaker' && vmKeys.map(k => (
                                        <option key={`vmkey-${k.id}`} value={k.id}>{k.name}</option>
                                    ))}
                                    {(selectedSetting as MassEditorSetting).dynamicKeys === 'pollinations' && polKeys.map(k => (
                                        <option key={`polkey-${k.id}`} value={k.id}>{k.name}</option>
                                    ))}
                                    {(selectedSetting as MassEditorSetting).dynamicKeys === 'elevenlabsimage' && elImgKeys.map(k => (
                                        <option key={`elimgkey-${k.id}`} value={k.id}>{k.name}</option>
                                    ))}
                                </select>
                            ) : selectedSetting.type === 'slider' ? (
                                <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                                    <input
                                        type="range"
                                        min={selectedSetting.min ?? 0}
                                        max={selectedSetting.max ?? 1}
                                        step={selectedSetting.step ?? 0.1}
                                        value={bulkValue || 0}
                                        onChange={(e) => setBulkValue(parseFloat(e.target.value))}
                                        className="panel-slider"
                                    />
                                    <span className="slider-value">
                                        {bulkValue || 0}
                                        {selectedSetting.dynamicModels === 'edgetts-r' && '%'}
                                        {selectedSetting.dynamicModels === 'edgetts-p' && 'Hz'}
                                        {selectedSetting.dynamicModels === 'edgetts-v' && '%'}
                                    </span>
                                </div>
                            ) : selectedSetting.type === 'switch' ? (
                                <label className="stage-switch">
                                    <input
                                        type="checkbox"
                                        checked={bulkValue}
                                        onChange={(e) => setBulkValue(e.target.checked)}
                                    />
                                    <span className="stage-slider"></span>
                                </label>
                            ) : selectedSetting.type === 'color' ? (
                                <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                                    <div style={{
                                        width: '32px',
                                        height: '32px',
                                        borderRadius: '6px',
                                        backgroundColor: bulkValue || '#ffffff',
                                        border: '2px solid var(--border-color)',
                                        position: 'relative',
                                        overflow: 'hidden',
                                        cursor: 'pointer'
                                    }}>
                                        <input
                                            type="color"
                                            value={bulkValue || '#ffffff'}
                                            onChange={(e) => setBulkValue(e.target.value)}
                                            style={{ position: 'absolute', top: '-5px', left: '-5px', width: '50px', height: '50px', cursor: 'pointer', opacity: 0 }}
                                        />
                                    </div>
                                    <input
                                        type="text"
                                        className="panel-input"
                                        style={{ fontFamily: 'monospace', fontSize: '11px', textTransform: 'uppercase', width: '100px' }}
                                        value={bulkValue || '#ffffff'}
                                        onChange={(e) => setBulkValue(e.target.value)}
                                    />
                                </div>
                            ) : selectedSetting.type === 'number' ? (
                                <input
                                    type="number"
                                    className="panel-input"
                                    value={bulkValue}
                                    onChange={(e) => setBulkValue(parseFloat(e.target.value))}
                                />
                            ) : selectedSetting.type === 'path' ? (
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
                                            let path = "";
                                            if (selectedSetting.id.includes('Watermark') || selectedSetting.id.includes('Image')) {
                                                path = await SelectImage();
                                            } else if (selectedSetting.id.includes('Intro') || selectedSetting.id.includes('Overlay')) {
                                                path = await SelectVideo();
                                            } else {
                                                path = await SelectDirectory();
                                            }
                                            if (path) setBulkValue(path);
                                        }}
                                        style={{ padding: '8px 12px', background: 'var(--accent-primary)', border: 'none', borderRadius: '4px', cursor: 'pointer' }}
                                    >
                                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>
                                    </button>
                                </div>
                            ) : (
                                <input
                                    type="text"
                                    className="panel-input"
                                    value={bulkValue}
                                    onChange={(e) => setBulkValue(e.target.value)}
                                />
                            )}
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
