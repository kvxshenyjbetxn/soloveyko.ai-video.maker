import React, { useState, useEffect, useRef, useCallback, useMemo } from 'react';
import './PipelineSidebar.css';
import { useI18n } from '../contexts/I18nContext';
import { useQueue } from '../contexts/QueueContext';
import { useServices } from '../contexts/ServiceContext';
import { useTemplates } from '../contexts/TemplateContext';
// @ts-ignore
import { GetPipelineSettings, SavePipelineSettings, GetOpenRouterSavedModels, SelectDirectory, GetDefaultVideosPath, GetElevenLabsBotVoiceTemplates, GetVoiceMakerVoices, GetPollinationsImageModels, GetPollinationsSavedModels, SavePollinationsModels, GetEdgeTTSVoices } from '../../wailsjs/go/main/App';
import voicemakerVoicesData from '../assets/voicemaker_voices.json';

import { TaskNameModal } from './TaskNameModal';
import { ConfirmModal } from './ConfirmModal';

// Pipeline Sidebar Modules
import { SidebarHeader } from './pipeline-sidebar/SidebarHeader';
import { TemplatesSection } from './pipeline-sidebar/TemplatesSection';
import { ControlSection } from './pipeline-sidebar/ControlSection';
import { ApiSection } from './pipeline-sidebar/ApiSection';
import { PathSection } from './pipeline-sidebar/PathSection';
import { TextSection } from './pipeline-sidebar/TextSection';
import { VoiceoverSection } from './pipeline-sidebar/VoiceoverSection';
import { SubtitleSection } from './pipeline-sidebar/SubtitleSection';
import { ImageSection } from './pipeline-sidebar/ImageSection';
import { MontageSection } from './pipeline-sidebar/MontageSection';
import { SidebarFooter } from './pipeline-sidebar/SidebarFooter';

interface PipelineSidebarProps {
    type: 'translate' | 'rewrite' | 'voiceover';
    isOpen: boolean;
    onToggle: () => void;
    content: string;
    setCurrentPath?: (path: string) => void;
}

export const PipelineSidebar: React.FC<PipelineSidebarProps> = ({ type, isOpen, onToggle, content, setCurrentPath }) => {
    const { t } = useI18n();
    const { addTasks, addTask } = useQueue();
    const { openRouterKeys, elevenLabsBotKeys, elevenLabsUnlimKeys, elevenLabsUAKeys, voiceMakerKeys, pollinationsKeys, elevenLabsImageKeys } = useServices();
    const [settings, setSettings] = useState<any>(null);
    const [models, setModels] = useState<string[]>([]);
    const [isResizing, setIsResizing] = useState(false);
    const [editingField, setEditingField] = useState<string | null>(null);
    const [isModalOpen, setIsModalOpen] = useState(false);
    const [templateToDelete, setTemplateToDelete] = useState<any | null>(null);
    const { templates, saveTemplate, removeTemplate, selectedTemplateIds, setSelectedTemplateIds } = useTemplates();
    const [voiceTemplates, setVoiceTemplates] = useState<string[]>([]);
    const [loadingTemplates, setLoadingTemplates] = useState(false);
    const [voiceMakerVoices, setVoiceMakerVoices] = useState<any[]>([]);
    const [pollinationsModels, setPollinationsModels] = useState<string[]>([]);
    const [loadingPollinationsModels, setLoadingPollinationsModels] = useState(false);
    const [edgeTTSVoices, setEdgeTTSVoices] = useState<any[]>([]);

    const sidebarRef = useRef<HTMLDivElement>(null);
    const lastSavedRef = useRef<string>("");

    const fetchVoiceTemplates = async (keyID?: string) => {
        // @ts-ignore
        const app = window.go.main.App as any;

        // 1. Пріоритет: переданий ID -> settings зі стейту -> settings з пропсів (якщо є)
        const id = keyID || settings?.voiceoverElevenLabsBotKeyID;

        if (app && app.LogFromUI) {
            app.LogFromUI("INFO", "[Frontend] Запит на отримання шаблонів ElevenLabs Bot...");
        }

        let keyObj = elevenLabsBotKeys.find((k: any) => k.id === id);

        // Якщо за ID не знайшли або ID "default", беремо перший доступний ключ
        if ((!keyObj || id === 'default') && elevenLabsBotKeys.length > 0) {
            keyObj = elevenLabsBotKeys[0];
            if (app && app.LogFromUI) app.LogFromUI("INFO", "[Frontend] Використовую ключ: " + keyObj.name);
        }

        if (!keyObj) {
            if (app && app.LogFromUI) app.LogFromUI("ERROR", "[Frontend] Помилка: Ключі ElevenLabs Bot не знайдені в системі!");
            return;
        }

        setLoadingTemplates(true);
        try {
            const results = await GetElevenLabsBotVoiceTemplates(keyObj.key);
            if (results && results.length > 0) {
                setVoiceTemplates(results);
                if (app && app.LogFromUI) {
                    app.LogFromUI("SUCCESS", `[Frontend] Шаблони успішно завантажені (кількість: ${results.length})`);
                }
            } else {
                if (app && app.LogFromUI) app.LogFromUI("WARN", "[Frontend] Сервер повернув порожній список шаблонів.");
                setVoiceTemplates([]);
            }
        } catch (err: any) {
            if (app && app.LogFromUI) app.LogFromUI("ERROR", "[Frontend] Помилка: " + (err?.message || String(err)));
            setVoiceTemplates([]);
        } finally {
            setLoadingTemplates(false);
        }
    };

    const normalizeVoices = (data: any[]) => {
        if (!data || data.length === 0) return [];
        if (data[0].Voices && Array.isArray(data[0].Voices)) {
            const flat: any[] = [];
            data.forEach((langGroup: any) => {
                langGroup.Voices.forEach((voiceId: string) => {
                    flat.push({
                        VoiceId: voiceId,
                        LanguageName: langGroup.Language,
                        LanguageCode: langGroup.LanguageCode || 'multi-lang',
                        VoiceWebname: voiceId.split('-').pop() || voiceId,
                        Engine: voiceId.startsWith('ai') ? 'neural' : 'standard'
                    });
                });
            });
            return flat;
        }
        return data;
    };

    const fetchVoiceMakerVoices = async (keyID?: string) => {
        const id = keyID || settings?.voiceoverVoiceMakerKeyID;
        if (!id) {
            setVoiceMakerVoices(normalizeVoices(voicemakerVoicesData || []));
            return;
        }

        setLoadingTemplates(true);
        try {
            const keyObj = voiceMakerKeys.find((k: any) => k.id === id);
            if (keyObj) {
                const results = await GetVoiceMakerVoices(keyObj.key);
                if (results && results.length > 0) {
                    setVoiceMakerVoices(normalizeVoices(results));
                } else {
                    setVoiceMakerVoices(normalizeVoices(voicemakerVoicesData || []));
                }
            } else {
                setVoiceMakerVoices(normalizeVoices(voicemakerVoicesData || []));
            }
        } finally {
            setLoadingTemplates(false);
        }
    };

    const fetchEdgeTTSVoices = async () => {
        setLoadingTemplates(true);
        try {
            const results = await GetEdgeTTSVoices();
            if (results && results.length > 0) {
                setEdgeTTSVoices(results);
            }
        } catch (err) {
            console.error("Failed to fetch Edge TTS voices:", err);
        } finally {
            setLoadingTemplates(false);
        }
    };

    const fetchPollinationsModels = async () => {
        setLoadingPollinationsModels(true);
        try {
            const results = await GetPollinationsImageModels();
            if (results && results.length > 0) {
                setPollinationsModels(results);
                await SavePollinationsModels(results);
            }
        } catch (err) {
            console.error("Failed to fetch Pollinations models:", err);
            const saved = await GetPollinationsSavedModels();
            if (saved && saved.length > 0) {
                setPollinationsModels(saved);
            }
        } finally {
            setLoadingPollinationsModels(false);
        }
    };

    const estimatedChunks = useMemo(() => {
        if (!settings || !content) return 0;
        const method = settings.imageGenerationMethod || 'lines';
        const group = settings.imageGroupSentences;
        const limit = settings.imageSentenceLimit ?? 1000;

        if (method === 'lines') {
            return content.split('\n').map(l => l.trim()).filter(l => l).length;
        } else {
            const matches = content.match(/[^.!?]+[.!?]*/g) || [];
            const sentences = matches.map(s => s.trim()).filter(s => s);
            if (!group) return sentences.length;

            let chunksNum = 0;
            let currentLen = 0;

            for (const sentence of sentences) {
                if (currentLen === 0) {
                    currentLen = sentence.length;
                } else if (currentLen + 1 + sentence.length <= limit) {
                    currentLen += 1 + sentence.length;
                } else {
                    chunksNum++;
                    currentLen = sentence.length;
                }
            }
            if (currentLen > 0) chunksNum++;
            return chunksNum;
        }
    }, [content, settings?.imageGenerationMethod, settings?.imageGroupSentences, settings?.imageSentenceLimit]);

    useEffect(() => {
        if (!settings) return;

        // Fetch ElevenLabs Bot templates if service is active and key exists
        if (settings.voiceoverService === 'elevenlabsbot' && settings.voiceoverElevenLabsBotKeyID && elevenLabsBotKeys.length > 0) {
            fetchVoiceTemplates(settings.voiceoverElevenLabsBotKeyID);
        }

        // Fetch VoiceMaker voices if service is active and key exists
        if (settings.voiceoverService === 'voicemaker' && settings.voiceoverVoiceMakerKeyID && voiceMakerKeys.length > 0) {
            fetchVoiceMakerVoices(settings.voiceoverVoiceMakerKeyID);
        }
    }, [settings?.voiceoverService, settings?.voiceoverElevenLabsBotKeyID, settings?.voiceoverVoiceMakerKeyID, elevenLabsBotKeys, voiceMakerKeys]);

    useEffect(() => {
        const init = async () => {
            try {
                const orModels = await GetOpenRouterSavedModels();
                const pModels = await GetPollinationsSavedModels();
                const s = await GetPipelineSettings();

                setModels(orModels || []);
                if (pModels && pModels.length > 0) setPollinationsModels(pModels);

                let updated = false;
                const modelList = orModels || [];

                if (modelList.length > 0) {
                    if (s.translateModel === "") { s.translateModel = modelList[0]; updated = true; }
                    if (s.rewriteModel === "") { s.rewriteModel = modelList[0]; updated = true; }
                }

                if (openRouterKeys.length > 0) {
                    if (!s.translateOpenRouterKeyID) { s.translateOpenRouterKeyID = openRouterKeys[0].id; updated = true; }
                    if (!s.rewriteOpenRouterKeyID) { s.rewriteOpenRouterKeyID = openRouterKeys[0].id; updated = true; }
                }

                if (elevenLabsBotKeys.length > 0 && !s.voiceoverElevenLabsBotKeyID) {
                    s.voiceoverElevenLabsBotKeyID = elevenLabsBotKeys[0].id;
                    updated = true;
                }

                if (voiceMakerKeys.length > 0 && !s.voiceoverVoiceMakerKeyID) {
                    s.voiceoverVoiceMakerKeyID = voiceMakerKeys[0].id;
                    updated = true;
                }

                if (elevenLabsUAKeys.length > 0 && !s.voiceoverElevenLabsUAKeyID) {
                    s.voiceoverElevenLabsUAKeyID = elevenLabsUAKeys[0].id;
                    updated = true;
                }

                if (pollinationsKeys.length > 0 && !s.imagePollinationsKeyID) {
                    s.imagePollinationsKeyID = pollinationsKeys[0].id;
                    updated = true;
                }

                if (!s.rewriteEnabled) { s.rewriteEnabled = true; updated = true; }
                if (s.voiceoverEnabled === undefined) { s.voiceoverEnabled = false; updated = true; }

                if (!s.translateOutputPath || !s.rewriteOutputPath || !s.voiceoverOutputPath || !s.imageOutputPath) {
                    const def = await GetDefaultVideosPath();
                    if (def) {
                        if (!s.translateOutputPath) s.translateOutputPath = s.outputPath || def;
                        if (!s.rewriteOutputPath) s.rewriteOutputPath = s.outputPath || def;
                        if (!s.voiceoverOutputPath) s.voiceoverOutputPath = s.outputPath || def;
                        if (!s.imageOutputPath) s.imageOutputPath = s.outputPath || def;
                        updated = true;
                    }
                }

                // Initial UI states
                s.apiCollapsed = true;
                s.pathCollapsed = true;
                if (s.translateTemplatesCollapsed === undefined) s.translateTemplatesCollapsed = true;
                if (s.rewriteTemplatesCollapsed === undefined) s.rewriteTemplatesCollapsed = true;
                if (s.voiceoverTemplatesCollapsed === undefined) s.voiceoverTemplatesCollapsed = true;
                if (s.imageTemplatesCollapsed === undefined) s.imageTemplatesCollapsed = true;
                if (s.controlCollapsed === undefined) s.controlCollapsed = true;
                if (s.translateControlEnabled === undefined) { s.translateControlEnabled = false; updated = true; }
                if (s.imageControlEnabled === undefined) { s.imageControlEnabled = false; updated = true; }
                if (s.subtitleCollapsed === undefined) s.subtitleCollapsed = true;
                if (s.imageCollapsed === undefined) s.imageCollapsed = true;
                if (s.montageCollapsed === undefined) s.montageCollapsed = true;
                if (s.montageEnabled === undefined) { s.montageEnabled = false; updated = true; }

                if (s.translateTemperature === undefined) s.translateTemperature = 0.7;
                if (s.rewriteTemperature === undefined) s.rewriteTemperature = 0.7;
                if (s.translateMaxTokens === undefined) s.translateMaxTokens = 0;
                if (s.rewriteMaxTokens === undefined) s.rewriteMaxTokens = 0;

                if (s.imagePromptModel === undefined) s.imagePromptModel = modelList.length > 0 ? modelList[0] : "";
                if (s.imagePromptTemperature === undefined) s.imagePromptTemperature = 0.7;
                if (s.imagePromptMaxTokens === undefined) s.imagePromptMaxTokens = 0;

                if (s.imageWidth === undefined) s.imageWidth = 1920;
                if (s.imageHeight === undefined) s.imageHeight = 1080;
                if (s.imageNoLogo === undefined) s.imageNoLogo = true;
                if (s.imageEnhance === undefined) s.imageEnhance = false;
                if (s.imagePrompt === undefined) s.imagePrompt = "";
                if (s.imageService === undefined) { s.imageService = "pollinations"; updated = true; }
                if (s.imageGooglerModel === undefined) { s.imageGooglerModel = "whisk"; updated = true; }
                if (s.imageGooglerVideoModel === undefined) { s.imageGooglerVideoModel = "whisk"; updated = true; }
                if (s.imageGooglerVideoUpscale === undefined) { s.imageGooglerVideoUpscale = false; updated = true; }
                if (s.imageGooglerVideoEnabled === undefined) { s.imageGooglerVideoEnabled = false; updated = true; }

                if (s.elevenLabsUnlimStability === undefined) s.elevenLabsUnlimStability = 0.5;
                if (s.elevenLabsUnlimSimilarity === undefined) s.elevenLabsUnlimSimilarity = 0.75;
                if (s.elevenLabsUnlimStyle === undefined) s.elevenLabsUnlimStyle = 0.0;
                if (s.elevenLabsUnlimSpeakerBoost === undefined) s.elevenLabsUnlimSpeakerBoost = true;

                if (s.elevenLabsUAStability === undefined) s.elevenLabsUAStability = 0.5;
                if (s.elevenLabsUASimilarity === undefined) s.elevenLabsUASimilarity = 0.75;
                if (s.elevenLabsUAStyle === undefined) s.elevenLabsUAStyle = 0.0;
                if (s.elevenLabsUASpeakerBoost === undefined) s.elevenLabsUASpeakerBoost = true;
                if (s.elevenLabsUAModel === undefined) s.elevenLabsUAModel = 'eleven_multilingual_v2';

                if (!s.subtitleService) { s.subtitleService = 'standard'; updated = true; }
                if (!s.subtitleModel) { s.subtitleModel = 'base'; updated = true; }
                if (s.subtitleEnabled === undefined) { s.subtitleEnabled = false; updated = true; }

                if (!s.voiceoverService) { s.voiceoverService = 'elevenlabsbot'; updated = true; }

                if (updated) await SavePipelineSettings(s);
                setSettings(s);
                lastSavedRef.current = JSON.stringify(s);

            } catch (err) {
                console.error("Failed to initialize sidebar:", err);
            }
        };
        init();
    }, [type]);

    useEffect(() => {
        document.documentElement.style.setProperty('--sidebar-toggle-width', '36px');
        return () => { document.documentElement.style.setProperty('--sidebar-toggle-width', '0px'); };
    }, []);

    useEffect(() => {
        const width = isOpen ? (settings?.sidebarWidth || 320) : 0;
        document.documentElement.style.setProperty('--pipeline-sidebar-width', `${width}px`);
        return () => { document.documentElement.style.setProperty('--pipeline-sidebar-width', '0px'); };
    }, [settings?.sidebarWidth, isOpen]);

    const handleChange = (field: string, value: any) => {
        setSettings((prev: any) => ({ ...prev, [field]: value }));
    };

    const handleSaveTemplate = async () => {
        const name = type === 'translate' ? settings.translatePipelineName : (type === 'rewrite' ? settings.rewritePipelineName : settings.voiceoverPipelineName);
        const textSet: any = {};
        const voiceSet: any = {};
        const commonSet: any = {};

        Object.keys(settings).forEach(key => {
            if (key.startsWith(type)) {
                if (key.endsWith('Collapsed') || key.endsWith('OutputPath') || key.endsWith('PipelineName') || key === 'translateControlEnabled') return;
                textSet[key] = settings[key];
            }
        });

        const voiceoverFields = [
            'voiceoverEnabled', 'voiceoverService', 'voiceoverTemplate',
            'voiceoverElevenLabsBotKeyID', 'voiceoverElevenLabsUnlimKeyID', 'voiceoverElevenLabsUAKeyID',
            'elevenLabsUnlimVoiceID', 'elevenLabsUnlimStability', 'elevenLabsUnlimSimilarity', 'elevenLabsUnlimStyle', 'elevenLabsUnlimSpeakerBoost',
            'elevenLabsUAVoiceID', 'elevenLabsUAStability', 'elevenLabsUASimilarity', 'elevenLabsUAStyle', 'elevenLabsUASpeakerBoost', 'elevenLabsUAModel',
            'voiceoverVoiceMakerKeyID', 'voiceMakerVoiceID', 'voiceMakerLanguageCode', 'voiceMakerCharLimit'
        ];
        voiceoverFields.forEach(field => { if (settings[field] !== undefined) voiceSet[field] = settings[field]; });

        const subtitleFields = [
            'subtitleEnabled', 'subtitleService', 'subtitleModel'
        ];
        subtitleFields.forEach(field => { if (settings[field] !== undefined) commonSet[field] = settings[field]; });

        const imageFields = [
            'imageEnabled', 'imageService', 'imageModel', 'imageWidth', 'imageHeight', 'imageNoLogo', 'imageEnhance', 'imagePrompt', 'imagePollinationsKeyID',
            'imageGenerationMethod', 'imageGroupSentences', 'imageSentenceLimit', 'imagePromptModel', 'imagePromptTemperature', 'imagePromptMaxTokens',
            'elevenLabsImageKeyID', 'elevenLabsImageAspectRatio', 'elevenLabsImagePortrait',
            'imageGooglerModel', 'imageGooglerAspectRatio', 'imageGooglerRemixEnabled', 'imageGooglerReferenceImage', 'imageGooglerRemixStrictMode',
            'imageGooglerVideoEnabled', 'imageGooglerVideoModel', 'imageGooglerVideoMode', 'imageGooglerVideoCount', 'imageGooglerVideoUpscale'
        ];
        imageFields.forEach(field => { if (settings[field] !== undefined) commonSet[field] = settings[field]; });

        if (settings.translateControlEnabled !== undefined) commonSet.translateControlEnabled = settings.translateControlEnabled;
        if (settings.imageControlEnabled !== undefined) commonSet.imageControlEnabled = settings.imageControlEnabled;

        await saveTemplate(type, name, { text: textSet, voiceover: voiceSet, common: commonSet });
    };

    const handleConfirmDelete = async () => {
        if (templateToDelete) {
            await removeTemplate(templateToDelete.id);
            setSelectedTemplateIds(prev => prev.filter(id => id !== templateToDelete.id));
            setTemplateToDelete(null);
        }
    };

    const handleAddTask = (taskName: string) => {
        const relevantTemplateIds = selectedTemplateIds.filter(id => templates.find(t => t.id === id)?.type === type);
        if (relevantTemplateIds.length === 0) {
            addTask(type, content, settings, taskName);
        } else {
            const tasksData = relevantTemplateIds.map(id => {
                const template = templates.find(t => t.id === id);
                let tplSettings = template?.settings;
                if (tplSettings && (tplSettings.text || tplSettings.voiceover || tplSettings.common)) {
                    tplSettings = { ...(tplSettings.text || {}), ...(tplSettings.voiceover || {}), ...(tplSettings.common || {}) };
                }
                return { settings: tplSettings, subName: template?.name };
            }).filter(d => d.settings);
            addTasks(type, content, tasksData as any, taskName);
            setSelectedTemplateIds([]);
        }
        setIsModalOpen(false);
    };

    const applyTemplate = (tpl: any) => {
        let applied = tpl.settings;
        if (tpl.settings && (tpl.settings.text || tpl.settings.voiceover || tpl.settings.common)) {
            applied = { ...(tpl.settings.text || {}), ...(tpl.settings.voiceover || {}), ...(tpl.settings.common || {}) };
        }
        setSettings((prev: any) => ({
            ...prev, ...applied,
            sidebarWidth: prev.sidebarWidth, translateCollapsed: prev.translateCollapsed, rewriteCollapsed: prev.rewriteCollapsed,
            apiCollapsed: prev.apiCollapsed, pathCollapsed: prev.pathCollapsed, templatesCollapsed: prev.templatesCollapsed,
            translateTemplatesCollapsed: prev.translateTemplatesCollapsed, rewriteTemplatesCollapsed: prev.rewriteTemplatesCollapsed,
            voiceoverTemplatesCollapsed: prev.voiceoverTemplatesCollapsed, controlCollapsed: prev.controlCollapsed,
        }));
    };

    const resize = useCallback((e: MouseEvent) => {
        if (isResizing && sidebarRef.current) {
            const newWidth = window.innerWidth - e.pageX;
            if (newWidth >= 250 && newWidth <= 600) handleChange('sidebarWidth', newWidth);
        }
    }, [isResizing]);

    useEffect(() => {
        if (!settings) return;
        const currentString = JSON.stringify(settings);
        if (currentString !== lastSavedRef.current) {
            const timer = setTimeout(async () => {
                // Fetch LATEST from file to avoid overwriting Performance tab settings
                const saved = await GetPipelineSettings();

                // Fields managed BY THE SIDEBAR (we only update these)
                const sidebarManagedSettings = {
                    ...settings
                };

                // Fields managed BY THE PERFORMANCE TAB (we strictly PRESERVE these from 'saved')
                const performanceFields = [
                    'montageVideoCodec',
                    'montageThreadsPerProcess',
                    'montageProcessPriority',
                    'montageCPUCores'
                ];

                const merged = { ...sidebarManagedSettings } as any;
                performanceFields.forEach(field => {
                    if ((saved as any)[field] !== undefined) {
                        merged[field] = (saved as any)[field];
                    }
                });

                await SavePipelineSettings(merged);
                lastSavedRef.current = currentString;
            }, 500);
            return () => clearTimeout(timer);
        }
    }, [settings]);

    useEffect(() => {
        if (isResizing) {
            window.addEventListener('mousemove', resize);
            window.addEventListener('mouseup', () => setIsResizing(false));
        }
        return () => {
            window.removeEventListener('mousemove', resize);
            window.removeEventListener('mouseup', () => setIsResizing(false));
        };
    }, [isResizing, resize]);

    const handleSelectPath = async () => {
        try {
            const path = await SelectDirectory();
            if (path) {
                const field = type === 'translate' ? 'translateOutputPath' : (type === 'rewrite' ? 'rewriteOutputPath' : 'voiceoverOutputPath');
                handleChange(field, path);
            }
        } catch (err) { console.error("Failed to select path:", err); }
    };

    const renderValueOrInput = (field: string, value: number, isFloat: boolean) => {
        if (editingField === field) {
            return (
                <input
                    autoFocus className="settings-value-input" type="number" defaultValue={value} step={isFloat ? "0.1" : "500"}
                    onBlur={(e) => {
                        setEditingField(null);
                        let val = parseFloat(e.target.value);
                        if (isNaN(val)) val = value;
                        handleChange(field, val);
                    }}
                    onKeyDown={(e) => { if (e.key === 'Enter') (e.target as HTMLInputElement).blur(); }}
                />
            );
        }
        let displayValue: string | number = isFloat ? value.toFixed(1) : value;
        if (!isFloat && value === 0 && field.includes('MaxTokens')) displayValue = t('pipeline.max_tokens_unlimited');
        return (
            <span className="settings-slider-value" onClick={(e) => { e.stopPropagation(); setEditingField(field); }}
                style={!isFloat && value === 0 ? { minWidth: '80px', fontSize: '10px' } : {}}>
                {displayValue}
            </span>
        );
    };

    if (!settings) return null;

    const templatesCollapsedField = type === 'translate' ? 'translateTemplatesCollapsed' : (type === 'rewrite' ? 'rewriteTemplatesCollapsed' : 'voiceoverTemplatesCollapsed');

    return (
        <aside className="pipeline-sidebar" ref={sidebarRef} style={{ width: `${isOpen ? (settings.sidebarWidth || 320) : 0}px` }}>
            <div className={`sidebar-resizer ${isResizing ? 'is-resizing' : ''}`} onMouseDown={(e) => { setIsResizing(true); e.preventDefault(); }} />
            <div className="sidebar-clipper">
                <SidebarHeader type={type} settings={settings} handleChange={handleChange} handleSaveTemplate={handleSaveTemplate} />
                <div className="pipeline-sidebar-content">
                    <TemplatesSection
                        type={type} templates={templates} selectedTemplateIds={selectedTemplateIds}
                        toggleTemplate={(id) => setSelectedTemplateIds(prev => prev.includes(id) ? prev.filter(t => t !== id) : [...prev, id])}
                        applyTemplate={applyTemplate} setTemplateToDelete={setTemplateToDelete}
                        isCollapsed={settings[templatesCollapsedField]}
                        onToggleCollapse={(collapsed) => handleChange(templatesCollapsedField, collapsed)}
                        setCurrentPath={setCurrentPath}
                    />
                    <ControlSection settings={settings} handleChange={handleChange} />
                    <ApiSection
                        type={type} settings={settings} handleChange={handleChange}
                        openRouterKeys={openRouterKeys} elevenLabsBotKeys={elevenLabsBotKeys} elevenLabsUnlimKeys={elevenLabsUnlimKeys}
                        elevenLabsUAKeys={elevenLabsUAKeys} voiceMakerKeys={voiceMakerKeys} pollinationsKeys={pollinationsKeys}
                        elevenLabsImageKeys={elevenLabsImageKeys}
                        fetchVoiceTemplates={fetchVoiceTemplates} fetchVoiceMakerVoices={fetchVoiceMakerVoices} setCurrentPath={setCurrentPath}
                    />
                    <PathSection type={type} settings={settings} handleChange={handleChange} handleSelectPath={handleSelectPath} />

                    {(type === 'translate' || type === 'rewrite') && (
                        <TextSection type={type} settings={settings} handleChange={handleChange} models={models} renderValueOrInput={renderValueOrInput} setCurrentPath={setCurrentPath} />
                    )}

                    <VoiceoverSection
                        settings={settings} handleChange={handleChange} setSettings={setSettings}
                        fetchVoiceTemplates={fetchVoiceTemplates} fetchVoiceMakerVoices={fetchVoiceMakerVoices} fetchEdgeTTSVoices={fetchEdgeTTSVoices}
                        voiceTemplates={voiceTemplates} voiceMakerVoices={voiceMakerVoices} edgeTTSVoices={edgeTTSVoices} loadingTemplates={loadingTemplates}
                    />

                    <ImageSection
                        settings={settings} handleChange={handleChange} setSettings={setSettings}
                        fetchPollinationsModels={fetchPollinationsModels} pollinationsModels={pollinationsModels}
                        loadingPollinationsModels={loadingPollinationsModels} estimatedChunks={estimatedChunks}
                        content={content} models={models} renderValueOrInput={renderValueOrInput} setCurrentPath={setCurrentPath}
                    />

                    <SubtitleSection
                        settings={settings} handleChange={handleChange} setSettings={setSettings}
                    />

                    <MontageSection
                        settings={settings} handleChange={handleChange} setSettings={setSettings}
                    />

                </div>
                <SidebarFooter type={type} content={content} selectedTemplateIds={selectedTemplateIds} templates={templates} setIsModalOpen={setIsModalOpen} />
            </div>

            <TaskNameModal isOpen={isModalOpen} onClose={() => setIsModalOpen(false)} onConfirm={handleAddTask} />
            <button className={`sidebar-floating-toggle ${isOpen ? 'is-open' : ''}`} onClick={onToggle} title={isOpen ? t('pipeline.hide_settings') : t('pipeline.show_settings')}>
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" /></svg>
            </button>
            <ConfirmModal isOpen={!!templateToDelete} onClose={() => setTemplateToDelete(null)} onConfirm={handleConfirmDelete} title={t('common.delete')} message={t('templatesTab.delete_confirm')} />
        </aside>
    );
};
