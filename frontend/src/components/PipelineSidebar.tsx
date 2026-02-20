import { useState, useEffect, useRef, useCallback, useMemo } from 'react';
import './PipelineSidebar.css';
import { useI18n } from '../contexts/I18nContext';
import { useQueue } from '../contexts/QueueContext';
import { useServices } from '../contexts/ServiceContext';
import { useTemplates, PipelineSettings as TemplatePipelineSettings } from '../contexts/TemplateContext';
// @ts-ignore
import { GetPipelineSettings, SavePipelineSettings, GetOpenRouterSavedModels, SelectDirectory, GetDefaultVideosPath, GetElevenLabsBotVoiceTemplates, GetVoiceMakerVoices, GetPollinationsImageModels, GetPollinationsSavedModels, SavePollinationsModels } from '../../wailsjs/go/main/App';
import voicemakerVoicesData from '../assets/voicemaker_voices.json';

import { TaskNameModal } from './TaskNameModal';
import { ConfirmModal } from './ConfirmModal';
import SearchableSelect from './SearchableSelect';

interface PipelineSidebarProps {
    type: 'translate' | 'rewrite' | 'voiceover';
    isOpen: boolean;
    onToggle: () => void;
    content: string;
    setCurrentPath?: (path: string) => void;
}

export const PipelineSidebar: React.FC<PipelineSidebarProps> = ({ type, isOpen, onToggle, content, setCurrentPath }) => {
    const { t } = useI18n();
    const { addTask, addTasks } = useQueue();
    const { openRouterKeys, elevenLabsBotKeys, elevenLabsUnlimKeys, elevenLabsUAKeys, voiceMakerKeys, pollinationsKeys } = useServices();
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

    const fetchVoiceTemplates = async (keyID?: string) => {
        const id = keyID || settings?.voiceoverElevenLabsBotKeyID;
        if (!id) return;

        setLoadingTemplates(true);
        try {
            const keyObj = elevenLabsBotKeys.find((k: any) => k.id === id);
            if (keyObj) {
                const results = await GetElevenLabsBotVoiceTemplates(keyObj.key);
                setVoiceTemplates(results || []);
            }
        } catch (err) {
            console.error("Failed to fetch templates:", err);
        } finally {
            setLoadingTemplates(false);
        }
    };

    const normalizeVoices = (data: any[]) => {
        if (!data || data.length === 0) return [];
        // Check if it's the new grouped format
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
        } catch (err) {
            console.error("Failed to fetch VoiceMaker voices:", err);
            setVoiceMakerVoices(normalizeVoices(voicemakerVoicesData || []));
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
            // fallback to saved models
            const saved = await GetPollinationsSavedModels();
            if (saved && saved.length > 0) {
                setPollinationsModels(saved);
            }
        } finally {
            setLoadingPollinationsModels(false);
        }
    };

    const sidebarRef = useRef<HTMLDivElement>(null);
    const lastSavedRef = useRef<string>("");

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
        const init = async () => {
            try {
                const orModels = await GetOpenRouterSavedModels();
                const pModels = await GetPollinationsSavedModels();
                const s = await GetPipelineSettings();

                const modelList = orModels || [];
                setModels(modelList);

                if (pModels && pModels.length > 0) {
                    setPollinationsModels(pModels);
                }

                let updated = false;
                if (modelList.length > 0) {
                    if (s.translateModel === "") {
                        s.translateModel = modelList[0];
                        updated = true;
                    }
                    if (s.rewriteModel === "") {
                        s.rewriteModel = modelList[0];
                        updated = true;
                    }
                }

                if (openRouterKeys.length > 0) {
                    if (!s.translateOpenRouterKeyID) {
                        s.translateOpenRouterKeyID = openRouterKeys[0].id;
                        updated = true;
                    }
                    if (!s.rewriteOpenRouterKeyID) {
                        s.rewriteOpenRouterKeyID = openRouterKeys[0].id;
                        updated = true;
                    }
                }

                if (elevenLabsBotKeys.length > 0) {
                    if (!s.voiceoverElevenLabsBotKeyID) {
                        s.voiceoverElevenLabsBotKeyID = elevenLabsBotKeys[0].id;
                        updated = true;
                    }
                }

                // Завантажуємо ключі VoiceMaker
                if (voiceMakerKeys.length > 0) {
                    if (!s.voiceoverVoiceMakerKeyID) {
                        s.voiceoverVoiceMakerKeyID = voiceMakerKeys[0].id;
                        updated = true;
                    }
                }

                if (elevenLabsUAKeys.length > 0) {
                    if (!s.voiceoverElevenLabsUAKeyID) {
                        s.voiceoverElevenLabsUAKeyID = elevenLabsUAKeys[0].id;
                        updated = true;
                    }
                }

                if (pollinationsKeys.length > 0) {
                    if (!s.imagePollinationsKeyID) {
                        s.imagePollinationsKeyID = pollinationsKeys[0].id;
                        updated = true;
                    }
                }

                if (!s.rewriteEnabled) {
                    s.rewriteEnabled = true;
                    updated = true;
                }
                if (s.voiceoverEnabled === undefined) {
                    s.voiceoverEnabled = false;
                    updated = true;
                }

                if (!s.translateOutputPath || !s.rewriteOutputPath || !s.voiceoverOutputPath || !s.imageOutputPath) {
                    try {
                        const def = await GetDefaultVideosPath();
                        if (def) {
                            if (!s.translateOutputPath) s.translateOutputPath = s.outputPath || def;
                            if (!s.rewriteOutputPath) s.rewriteOutputPath = s.outputPath || def;
                            if (!s.voiceoverOutputPath) s.voiceoverOutputPath = s.outputPath || def;
                            if (!s.imageOutputPath) s.imageOutputPath = s.outputPath || def;
                            updated = true;
                        }
                    } catch (e) {
                        console.error("Failed to get default path:", e);
                    }
                }

                // Завжди згортаємо блок API та Шлях при ініціалізації
                s.apiCollapsed = true;
                s.pathCollapsed = true;

                // Для шаблонів встановлюємо дефолт, якщо ще немає
                if (s.translateTemplatesCollapsed === undefined) s.translateTemplatesCollapsed = true;
                if (s.rewriteTemplatesCollapsed === undefined) s.rewriteTemplatesCollapsed = true;
                if (s.voiceoverTemplatesCollapsed === undefined) s.voiceoverTemplatesCollapsed = true;
                if (s.imageTemplatesCollapsed === undefined) s.imageTemplatesCollapsed = true;
                if (s.controlCollapsed === undefined) s.controlCollapsed = true;
                if (s.imageCollapsed === undefined) s.imageCollapsed = true;

                // Забезпечуємо наявність значень для повзунків
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
                if (s.imageService === undefined) s.imageService = "pollinations";

                if (s.elevenLabsUnlimStability === undefined) s.elevenLabsUnlimStability = 0.5;
                if (s.elevenLabsUnlimSimilarity === undefined) s.elevenLabsUnlimSimilarity = 0.75;
                if (s.elevenLabsUnlimStyle === undefined) s.elevenLabsUnlimStyle = 0.0;
                if (s.elevenLabsUnlimSpeakerBoost === undefined) s.elevenLabsUnlimSpeakerBoost = true;

                if (s.elevenLabsUAStability === undefined) s.elevenLabsUAStability = 0.5;
                if (s.elevenLabsUASimilarity === undefined) s.elevenLabsUASimilarity = 0.75;
                if (s.elevenLabsUAStyle === undefined) s.elevenLabsUAStyle = 0.0;
                if (s.elevenLabsUASpeakerBoost === undefined) s.elevenLabsUASpeakerBoost = true;
                if (s.elevenLabsUAModel === undefined) s.elevenLabsUAModel = 'eleven_multilingual_v2';

                if (!s.voiceoverService) {
                    s.voiceoverService = 'elevenlabsbot';
                    updated = true;
                }

                // Оновлюємо налаштування на сервері
                if (updated) {
                    await SavePipelineSettings(s);
                }

                setSettings(s);
                lastSavedRef.current = JSON.stringify(s);

                // Завантажуємо шаблони голосів, якщо обрано ElevenLabs Bot або VoiceMaker
                if (s.voiceoverService === 'elevenlabsbot' && s.voiceoverElevenLabsBotKeyID) {
                    setTimeout(() => fetchVoiceTemplates(s.voiceoverElevenLabsBotKeyID), 0);
                }
                if (s.voiceoverService === 'voicemaker') {
                    setTimeout(() => fetchVoiceMakerVoices(s.voiceoverVoiceMakerKeyID), 0);
                }
            } catch (err) {
                console.error("Failed to initialize sidebar:", err);
            }
        };

        init();
    }, [type]);

    useEffect(() => {
        document.documentElement.style.setProperty('--sidebar-toggle-width', '36px');
        return () => {
            document.documentElement.style.setProperty('--sidebar-toggle-width', '0px');
        };
    }, []);

    useEffect(() => {
        const width = isOpen ? (settings?.sidebarWidth || 320) : 0;
        document.documentElement.style.setProperty('--pipeline-sidebar-width', `${width}px`);
        return () => {
            document.documentElement.style.setProperty('--pipeline-sidebar-width', '0px');
        };
    }, [settings?.sidebarWidth, isOpen]);

    const toggleTemplate = (id: string) => {
        setSelectedTemplateIds(prev =>
            prev.includes(id) ? prev.filter(t => t !== id) : [...prev, id]
        );
    };

    const handleSaveTemplate = async () => {
        const name = isTranslate ? settings.translatePipelineName : (isRewrite ? settings.rewritePipelineName : settings.voiceoverPipelineName);

        const textSet: any = {};
        const voiceSet: any = {};
        const commonSet: any = {};

        // 1. Копіюємо налаштування тексту (тільки того типу, який ми зараз зберігаємо)
        Object.keys(settings).forEach(key => {
            if (key.startsWith(type)) {
                // Виключаємо UI стани, шляхи та контроль (він іде в common)
                if (key.endsWith('Collapsed') || key.endsWith('OutputPath') || key.endsWith('PipelineName') || key === 'translateControlEnabled') {
                    return;
                }
                textSet[key] = settings[key];
            }
        });

        // 2. Копіюємо озвучку
        const voiceoverFields = [
            'voiceoverEnabled', 'voiceoverService', 'voiceoverTemplate',
            'voiceoverElevenLabsBotKeyID', 'voiceoverElevenLabsUnlimKeyID',
            'voiceoverElevenLabsUAKeyID',
            'elevenLabsUnlimVoiceID', 'elevenLabsUnlimStability', 'elevenLabsUnlimSimilarity',
            'elevenLabsUnlimStyle', 'elevenLabsUnlimSpeakerBoost',
            'elevenLabsUAVoiceID', 'elevenLabsUAStability', 'elevenLabsUASimilarity',
            'elevenLabsUAStyle', 'elevenLabsUASpeakerBoost', 'elevenLabsUAModel',
            'voiceoverVoiceMakerKeyID', 'voiceMakerVoiceID', 'voiceMakerLanguageCode', 'voiceMakerCharLimit'
        ];

        voiceoverFields.forEach(field => {
            if (settings[field] !== undefined) {
                voiceSet[field] = settings[field];
            }
        });

        // 3. Контроль та загальні
        if (settings.translateControlEnabled !== undefined) {
            commonSet.translateControlEnabled = settings.translateControlEnabled;
        }

        const imageFields = [
            'imageEnabled', 'imageService', 'imageModel', 'imageWidth', 'imageHeight',
            'imageNoLogo', 'imageEnhance', 'imagePrompt', 'imagePollinationsKeyID',
            'imageGenerationMethod', 'imageGroupSentences', 'imageSentenceLimit',
            'imagePromptModel', 'imagePromptTemperature', 'imagePromptMaxTokens'
        ];

        imageFields.forEach(field => {
            if (settings[field] !== undefined) {
                commonSet[field] = settings[field];
            }
        });

        const templateData = {
            text: textSet,
            voiceover: voiceSet,
            common: commonSet
        };

        await saveTemplate(type, name, templateData);
    };

    const handleConfirmDelete = async () => {
        if (templateToDelete) {
            await removeTemplate(templateToDelete.id);
            setSelectedTemplateIds(prev => prev.filter(id => id !== templateToDelete.id));
            setTemplateToDelete(null);
        }
    };

    const handleAddTask = (taskName: string) => {
        const relevantTemplateIds = selectedTemplateIds.filter(id => {
            const tpl = templates.find(t => t.id === id);
            return tpl && tpl.type === type;
        });

        if (relevantTemplateIds.length === 0) {
            // No templates of current type selected - use current sidebar settings
            addTask(type, content, settings, taskName);
        } else {
            // Create tasks for each selected template of current type using batch add
            const tasksData = relevantTemplateIds.map(id => {
                const template = templates.find(t => t.id === id);
                let tplSettings = template?.settings;

                // Flatten if nested
                if (tplSettings && (tplSettings.text || tplSettings.voiceover || tplSettings.common)) {
                    tplSettings = {
                        ...(tplSettings.text || {}),
                        ...(tplSettings.voiceover || {}),
                        ...(tplSettings.common || {})
                    };
                }

                return {
                    settings: tplSettings,
                    subName: template?.name
                };
            }).filter(d => d.settings);

            addTasks(type, content, tasksData as any, taskName);

            // Очищуємо виділення після додавання, щоб не було плутанини
            setSelectedTemplateIds([]);
        }
        setIsModalOpen(false);
    };

    const applyTemplate = (tpl: any) => {
        let appliedSettings = tpl.settings;

        // Перевіряємо, чи це нова вкладена структура, і розгортаємо її
        if (tpl.settings && (tpl.settings.text || tpl.settings.voiceover || tpl.settings.common)) {
            appliedSettings = {
                ...(tpl.settings.text || {}),
                ...(tpl.settings.voiceover || {}),
                ...(tpl.settings.common || {})
            };
        }

        setSettings((prev: any) => ({
            ...prev,
            ...appliedSettings,
            // Завжди зберігаємо поточний стан інтерфейсу, незалежно від того, що в шаблоні
            sidebarWidth: prev.sidebarWidth,
            translateCollapsed: prev.translateCollapsed,
            rewriteCollapsed: prev.rewriteCollapsed,
            apiCollapsed: prev.apiCollapsed,
            pathCollapsed: prev.pathCollapsed,
            templatesCollapsed: prev.templatesCollapsed,
            translateTemplatesCollapsed: prev.translateTemplatesCollapsed,
            rewriteTemplatesCollapsed: prev.rewriteTemplatesCollapsed,
            voiceoverTemplatesCollapsed: prev.voiceoverTemplatesCollapsed,
            controlCollapsed: prev.controlCollapsed,
        }));
    };

    const startResizing = useCallback((e: React.MouseEvent) => {
        setIsResizing(true);
        e.preventDefault();
    }, []);

    const stopResizing = useCallback(() => {
        setIsResizing(false);
    }, []);

    const resize = useCallback((e: MouseEvent) => {
        if (isResizing && sidebarRef.current) {
            const newWidth = window.innerWidth - e.pageX;
            if (newWidth >= 250 && newWidth <= 600) {
                setSettings((prev: any) => ({ ...prev, sidebarWidth: newWidth }));
            }
        }
    }, [isResizing]);

    useEffect(() => {
        if (!settings) return;

        const currentString = JSON.stringify(settings);
        if (currentString !== lastSavedRef.current) {
            const timer = setTimeout(() => {
                SavePipelineSettings(settings);
                lastSavedRef.current = currentString;
            }, 500);

            return () => clearTimeout(timer);
        }
    }, [settings]);

    useEffect(() => {
        if (isResizing) {
            window.addEventListener('mousemove', resize);
            window.addEventListener('mouseup', stopResizing);
        } else {
            window.removeEventListener('mousemove', resize);
            window.removeEventListener('mouseup', stopResizing);
        }

        return () => {
            window.removeEventListener('mousemove', resize);
            window.removeEventListener('mouseup', stopResizing);
        };
    }, [isResizing, resize, stopResizing]);

    const handleChange = (field: string, value: any) => {
        setSettings((prev: any) => ({
            ...prev,
            [field]: value
        }));
    };

    if (!settings) return null;

    const isTranslate = type === 'translate';
    const isRewrite = type === 'rewrite';
    const isVoiceover = type === 'voiceover';

    const isEnabled = isTranslate ? settings.translateEnabled : (isRewrite ? settings.rewriteEnabled : settings.voiceoverEnabled);
    const isCollapsed = isTranslate ? settings.translateCollapsed : (isRewrite ? settings.rewriteCollapsed : settings.voiceoverCollapsed);
    const templatesCollapsedField = isTranslate ? 'translateTemplatesCollapsed' : (isRewrite ? 'rewriteTemplatesCollapsed' : 'voiceoverTemplatesCollapsed');

    const isTemplatesCollapsed = settings[templatesCollapsedField];
    const isApiCollapsed = settings.apiCollapsed;
    const isPathCollapsed = settings.pathCollapsed;

    const modelValue = isTranslate ? settings.translateModel : (isRewrite ? settings.rewriteModel : '');
    const tempValue = (isTranslate ? settings.translateTemperature : (isRewrite ? settings.rewriteTemperature : 0.7)) ?? 0;
    const tokensValue = (isTranslate ? settings.translateMaxTokens : (isRewrite ? settings.rewriteMaxTokens : 0)) ?? 0;
    const promptValue = isTranslate ? settings.translatePrompt : (isRewrite ? settings.rewritePrompt : '');
    const selectedApiKeyID = isTranslate ? settings.translateOpenRouterKeyID : (isRewrite ? settings.rewriteOpenRouterKeyID : '');
    const selectedElevenLabsBotKeyID = isTranslate ? settings.translateElevenLabsBotKeyID : (isRewrite ? settings.rewriteElevenLabsBotKeyID : settings.voiceoverElevenLabsBotKeyID);


    const toggleCollapse = () => {
        const field = isTranslate ? 'translateCollapsed' : (isRewrite ? 'rewriteCollapsed' : 'voiceoverCollapsed');
        handleChange(field, !isCollapsed);
    };

    const handleToggleEnable = (e: React.ChangeEvent<HTMLInputElement>) => {
        const val = e.target.checked;
        const field = isTranslate ? 'translateEnabled' : (isRewrite ? 'rewriteEnabled' : 'voiceoverEnabled');
        const collapsedField = isTranslate ? 'translateCollapsed' : (isRewrite ? 'rewriteCollapsed' : 'voiceoverCollapsed');
        const newSettings = {
            ...settings,
            [field]: val
        };
        if (!val) {
            newSettings[collapsedField] = true;
        }
        setSettings(newSettings);
    };

    const handleSelectPath = async () => {
        try {
            const path = await SelectDirectory();
            if (path) {
                let field = '';
                if (isTranslate) field = 'translateOutputPath';
                else if (isRewrite) field = 'rewriteOutputPath';
                else field = 'voiceoverOutputPath';
                handleChange(field, path);
            }
        } catch (err) {
            console.error("Failed to select path:", err);
        }
    };

    const renderValueOrInput = (field: string, value: number, isFloat: boolean) => {
        if (editingField === field) {
            return (
                <input
                    autoFocus
                    className="settings-value-input"
                    type="number"
                    defaultValue={value}
                    step={isFloat ? "0.1" : "500"}
                    onBlur={(e) => {
                        setEditingField(null);
                        let val = parseFloat(e.target.value);
                        if (isNaN(val)) val = value;
                        handleChange(field, val);
                    }}
                    onKeyDown={(e) => {
                        if (e.key === 'Enter') {
                            (e.target as HTMLInputElement).blur();
                        }
                    }}
                />
            );
        }

        let displayValue: string | number = isFloat ? value.toFixed(1) : value;
        if (!isFloat && value === 0 && field.includes('MaxTokens')) {
            displayValue = t('pipeline.max_tokens_unlimited');
        }

        return (
            <span
                className="settings-slider-value"
                onClick={(e) => {
                    e.stopPropagation();
                    setEditingField(field);
                }}
                style={!isFloat && value === 0 ? { minWidth: '80px', fontSize: '10px' } : {}}
            >
                {displayValue}
            </span>
        );
    };

    return (
        <aside
            className="pipeline-sidebar"
            ref={sidebarRef}
            style={{ width: `${isOpen ? (settings.sidebarWidth || 320) : 0}px` }}
        >
            <div
                className={`sidebar-resizer ${isResizing ? 'is-resizing' : ''}`}
                onMouseDown={startResizing}
            />

            <div className="sidebar-clipper">
                <div className="pipeline-sidebar-header" style={{ display: 'block', padding: '10px 12px', borderBottom: '1px solid var(--border-color)' }}>
                    <div className="settings-control" style={{ marginBottom: 0 }}>
                        <label className="settings-label" style={{ marginBottom: '4px', fontSize: '10px' }}>{t('pipeline.name')}</label>
                        <div style={{ display: 'flex', gap: '8px' }}>
                            <input
                                className="settings-input"
                                value={(isTranslate ? settings.translatePipelineName : (isRewrite ? settings.rewritePipelineName : settings.voiceoverPipelineName)) || ''}
                                onChange={(e) => {
                                    let field = '';
                                    if (isTranslate) field = 'translatePipelineName';
                                    else if (isRewrite) field = 'rewritePipelineName';
                                    else field = 'voiceoverPipelineName';
                                    handleChange(field, e.target.value);
                                }}
                                placeholder="Назва пайплайну..."
                                style={{ flex: 1, height: '32px' }}
                            />
                            <button
                                className="save-template-btn"
                                onClick={handleSaveTemplate}
                                title={t('pipeline.save_template')}
                                style={{ marginTop: 0, width: '32px', height: '32px', flexShrink: 0 }}
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                                    <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"></path>
                                    <polyline points="17 21 17 13 7 13 7 21"></polyline>
                                    <polyline points="7 3 7 8 15 8"></polyline>
                                </svg>
                            </button>
                        </div>
                    </div>
                </div>

                <div className="pipeline-sidebar-content">

                    {/* Templates Section */}
                    <div className={`pipeline-stage-container ${isTemplatesCollapsed ? 'is-collapsed' : ''}`}>
                        <div
                            className="pipeline-stage-header"
                            onClick={() => handleChange(templatesCollapsedField, !isTemplatesCollapsed)}
                        >
                            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flex: 1 }}>
                                <svg
                                    className={`stage-chevron ${isTemplatesCollapsed ? 'rotated' : ''}`}
                                    xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"
                                >
                                    <path d="m6 9 6 6 6-6" />
                                </svg>
                                <div style={{ display: 'flex', flexDirection: 'column' }}>
                                    <span className="pipeline-stage-title">{t('pipeline.templates')}</span>
                                </div>
                                {setCurrentPath && (
                                    <button
                                        className="templates-settings-link"
                                        onClick={(e) => {
                                            e.stopPropagation();
                                            setCurrentPath('settings.templates');
                                        }}
                                        title={t('settings.templates')}
                                    >
                                        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                                            <circle cx="12" cy="12" r="3" />
                                            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
                                        </svg>
                                    </button>
                                )}
                            </div>
                        </div>
                        <div className={`stage-settings-content ${isTemplatesCollapsed ? 'collapsed' : ''}`}>
                            {templates.filter(t => t.type === type).length === 0 ? (
                                <div className="no-templates-text">{t('pipeline.no_templates')}</div>
                            ) : (
                                <div className="templates-list">
                                    {templates.filter(t => t.type === type).map(tpl => (
                                        <div
                                            key={tpl.id}
                                            className={`template-item ${selectedTemplateIds.includes(tpl.id) ? 'selected' : ''}`}
                                            onClick={() => toggleTemplate(tpl.id)}
                                        >
                                            <div className="template-checkbox">
                                                {selectedTemplateIds.includes(tpl.id) && (
                                                    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>
                                                )}
                                            </div>
                                            <span className="template-name">{tpl.name}</span>
                                            <button
                                                className="template-apply-btn"
                                                onClick={(e) => {
                                                    e.stopPropagation();
                                                    applyTemplate(tpl);
                                                }}
                                                title={t('common.load')}
                                            >
                                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                                                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                                                    <polyline points="7 10 12 15 17 10" />
                                                    <line x1="12" y1="15" x2="12" y2="3" />
                                                </svg>
                                            </button>
                                            <button
                                                className="template-delete-btn"
                                                onClick={(e) => {
                                                    e.stopPropagation();
                                                    setTemplateToDelete(tpl);
                                                }}
                                                title={t('common.delete')}
                                            >
                                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                                                    <line x1="18" y1="6" x2="6" y2="18"></line>
                                                    <line x1="6" y1="6" x2="18" y2="18"></line>
                                                </svg>
                                            </button>
                                        </div>
                                    ))}
                                </div>
                            )}
                        </div>
                    </div>

                    {/* Control Section */}
                    <div className={`pipeline-stage-container ${settings.controlCollapsed ? 'is-collapsed' : ''}`}>
                        <div
                            className="pipeline-stage-header"
                            onClick={() => handleChange('controlCollapsed', !settings.controlCollapsed)}
                        >
                            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flex: 1 }}>
                                <svg
                                    className={`stage-chevron ${settings.controlCollapsed ? 'rotated' : ''}`}
                                    xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"
                                >
                                    <path d="m6 9 6 6 6-6" />
                                </svg>
                                <div style={{ display: 'flex', flexDirection: 'column' }}>
                                    <span className="pipeline-stage-title">{t('pipeline.control') || 'Контроль'}</span>
                                </div>
                            </div>
                        </div>
                        <div className={`stage-settings-content ${settings.controlCollapsed ? 'collapsed' : ''}`}>
                            <div className="settings-group">
                                <div className="settings-control">
                                    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                                        <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.translate_control') || 'Контроль перекладу'}</label>
                                        <label className="stage-switch small">
                                            <input
                                                type="checkbox"
                                                checked={settings.translateControlEnabled}
                                                onChange={(e) => handleChange('translateControlEnabled', e.target.checked)}
                                            />
                                            <span className="stage-slider"></span>
                                        </label>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>

                    {/* API Settings Section */}
                    <div className={`pipeline-stage-container ${isApiCollapsed ? 'is-collapsed' : ''}`}>
                        <div
                            className="pipeline-stage-header"
                            onClick={() => handleChange('apiCollapsed', !isApiCollapsed)}
                        >
                            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flex: 1 }}>
                                <svg
                                    className={`stage-chevron ${isApiCollapsed ? 'rotated' : ''}`}
                                    xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"
                                >
                                    <path d="m6 9 6 6 6-6" />
                                </svg>
                                <div style={{ display: 'flex', flexDirection: 'column' }}>
                                    <span className="pipeline-stage-title">{t('pipeline.group.api')}</span>
                                </div>
                            </div>
                        </div>

                        <div className={`stage-settings-content ${isApiCollapsed ? 'collapsed' : ''}`}>
                            <div className="settings-group">
                                <div className="settings-control">
                                    <label className="settings-label">OpenRouter Key</label>
                                    <select
                                        className="settings-select"
                                        value={selectedApiKeyID}
                                        onChange={(e) => {
                                            const val = e.target.value;
                                            if (val === "MANAGE_KEYS") {
                                                if (setCurrentPath) {
                                                    setCurrentPath('settings.api.openrouter');
                                                }
                                                return;
                                            }
                                            let field = '';
                                            if (isTranslate) field = 'translateOpenRouterKeyID';
                                            else if (isRewrite) field = 'rewriteOpenRouterKeyID';
                                            if (field) handleChange(field, val);
                                        }}
                                    >
                                        {openRouterKeys.length === 0 ? (
                                            <option value="">{t('api.openrouterSettings.noKeys')}</option>
                                        ) : (
                                            openRouterKeys.map(k => (
                                                <option key={k.id} value={k.id}>{k.name}</option>
                                            ))
                                        )}
                                        <option value="MANAGE_KEYS" style={{ color: 'var(--accent-primary)', fontWeight: 'bold' }}>
                                            ⚙️ {t('tabs.settings')}
                                        </option>
                                    </select>
                                </div>

                                <div className="settings-control">
                                    <label className="settings-label">ElevenLabs Bot Key</label>
                                    <select
                                        className="settings-select"
                                        value={selectedElevenLabsBotKeyID}
                                        onChange={(e) => {
                                            const val = e.target.value;
                                            if (val === "MANAGE_KEYS") {
                                                if (setCurrentPath) {
                                                    setCurrentPath('settings.api.voice.elevenlabsbot');
                                                }
                                                return;
                                            }
                                            let field = '';
                                            if (isTranslate) field = 'translateElevenLabsBotKeyID';
                                            else if (isRewrite) field = 'rewriteElevenLabsBotKeyID';
                                            else field = 'voiceoverElevenLabsBotKeyID';
                                            handleChange(field, val);
                                            if (val !== "MANAGE_KEYS" && settings.voiceoverService === 'elevenlabsbot') {
                                                fetchVoiceTemplates(val);
                                            }
                                        }}
                                    >
                                        {elevenLabsBotKeys.length === 0 ? (
                                            <option value="">{t('api.openrouterSettings.noKeys')}</option>
                                        ) : (
                                            elevenLabsBotKeys.map(k => (
                                                <option key={k.id} value={k.id}>{k.name}</option>
                                            ))
                                        )}
                                        <option value="MANAGE_KEYS" style={{ color: 'var(--accent-primary)', fontWeight: 'bold' }}>
                                            ⚙️ {t('tabs.settings')}
                                        </option>
                                    </select>
                                </div>

                                <div className="settings-control">
                                    <label className="settings-label">ElevenLabs Unlim Key</label>
                                    <select
                                        className="settings-select"
                                        value={settings.voiceoverElevenLabsUnlimKeyID}
                                        onChange={(e) => {
                                            const val = e.target.value;
                                            if (val === "MANAGE_KEYS") {
                                                if (setCurrentPath) {
                                                    setCurrentPath('settings.api.voice.elevenlabsunlim');
                                                }
                                                return;
                                            }
                                            handleChange('voiceoverElevenLabsUnlimKeyID', val);
                                        }}
                                    >
                                        {elevenLabsUnlimKeys.length === 0 ? (
                                            <option value="">{t('api.openrouterSettings.noKeys')}</option>
                                        ) : (
                                            elevenLabsUnlimKeys.map(k => (
                                                <option key={k.id} value={k.id}>{k.name}</option>
                                            ))
                                        )}
                                        <option value="MANAGE_KEYS" style={{ color: 'var(--accent-primary)', fontWeight: 'bold' }}>
                                            ⚙️ {t('tabs.settings')}
                                        </option>
                                    </select>
                                </div>

                                <div className="settings-control">
                                    <label className="settings-label">ElevenLabs UA Key</label>
                                    <select
                                        className="settings-select"
                                        value={settings.voiceoverElevenLabsUAKeyID}
                                        onChange={(e) => {
                                            const val = e.target.value;
                                            if (val === "MANAGE_KEYS") {
                                                if (setCurrentPath) {
                                                    setCurrentPath('settings.api.voice.elevenlabsua');
                                                }
                                                return;
                                            }
                                            handleChange('voiceoverElevenLabsUAKeyID', val);
                                        }}
                                    >
                                        {elevenLabsUAKeys.length === 0 ? (
                                            <option value="">{t('api.openrouterSettings.noKeys')}</option>
                                        ) : (
                                            elevenLabsUAKeys.map(k => (
                                                <option key={k.id} value={k.id}>{k.name}</option>
                                            ))
                                        )}
                                        <option value="MANAGE_KEYS" style={{ color: 'var(--accent-primary)', fontWeight: 'bold' }}>
                                            ⚙️ {t('tabs.settings')}
                                        </option>
                                    </select>
                                </div>

                                <div className="settings-control">
                                    <label className="settings-label">VoiceMaker Key</label>
                                    <select
                                        className="settings-select"
                                        value={settings.voiceoverVoiceMakerKeyID}
                                        onChange={(e) => {
                                            const val = e.target.value;
                                            if (val === "MANAGE_KEYS") {
                                                if (setCurrentPath) {
                                                    setCurrentPath('settings.api.voice.voicemaker');
                                                }
                                                return;
                                            }
                                            handleChange('voiceoverVoiceMakerKeyID', val);
                                            if (settings.voiceoverService === 'voicemaker') {
                                                fetchVoiceMakerVoices(val);
                                            }
                                        }}
                                    >
                                        {voiceMakerKeys.length === 0 ? (
                                            <option value="">{t('api.openrouterSettings.noKeys')}</option>
                                        ) : (
                                            voiceMakerKeys.map(k => (
                                                <option key={k.id} value={k.id}>{k.name}</option>
                                            ))
                                        )}
                                        <option value="MANAGE_KEYS" style={{ color: 'var(--accent-primary)', fontWeight: 'bold' }}>
                                            ⚙️ {t('tabs.settings')}
                                        </option>
                                    </select>
                                </div>

                                <div className="settings-control">
                                    <label className="settings-label">Pollinations Key</label>
                                    <select
                                        className="settings-select"
                                        value={settings.imagePollinationsKeyID}
                                        onChange={(e) => {
                                            const val = e.target.value;
                                            if (val === "MANAGE_KEYS") {
                                                if (setCurrentPath) {
                                                    setCurrentPath('settings.api.image.pollinationsai');
                                                }
                                                return;
                                            }
                                            handleChange('imagePollinationsKeyID', val);
                                        }}
                                    >
                                        {pollinationsKeys.length === 0 ? (
                                            <option value="">{t('api.openrouterSettings.noKeys')}</option>
                                        ) : (
                                            pollinationsKeys.map(k => (
                                                <option key={k.id} value={k.id}>{k.name}</option>
                                            ))
                                        )}
                                        <option value="MANAGE_KEYS" style={{ color: 'var(--accent-primary)', fontWeight: 'bold' }}>
                                            ⚙️ {t('tabs.settings')}
                                        </option>
                                    </select>
                                </div>
                            </div>
                        </div>
                    </div>

                    {/* Save Path Section */}
                    <div className={`pipeline-stage-container ${isPathCollapsed ? 'is-collapsed' : ''}`}>
                        <div
                            className="pipeline-stage-header"
                            onClick={() => handleChange('pathCollapsed', !isPathCollapsed)}
                        >
                            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flex: 1 }}>
                                <svg
                                    className={`stage-chevron ${isPathCollapsed ? 'rotated' : ''}`}
                                    xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"
                                >
                                    <path d="m6 9 6 6 6-6" />
                                </svg>
                                <div style={{ display: 'flex', flexDirection: 'column' }}>
                                    <span className="pipeline-stage-title">{t('pipeline.stage.path')}</span>
                                </div>
                            </div>
                        </div>

                        <div className={`stage-settings-content ${isPathCollapsed ? 'collapsed' : ''}`}>
                            <div className="settings-group">
                                <div className="settings-control">
                                    <label className="settings-label">{t('pipeline.group.path')}</label>
                                    <div className="settings-row">
                                        <input
                                            className="settings-input"
                                            style={{ flex: 1, textOverflow: 'ellipsis' }}
                                            value={isTranslate ? settings?.translateOutputPath : (isRewrite ? settings?.rewriteOutputPath : (type === 'voiceover' ? settings?.voiceoverOutputPath : settings?.imageOutputPath)) || ''}
                                            readOnly
                                            placeholder="Виберіть папку..."
                                        />
                                        <button
                                            className="settings-button"
                                            onClick={handleSelectPath}
                                        >
                                            Обзор
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div className={`pipeline-stage-container ${isCollapsed || !isEnabled ? 'is-collapsed' : ''}`}>
                        <div
                            className="pipeline-stage-header"
                            onClick={toggleCollapse}
                        >
                            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flex: 1 }}>
                                <svg
                                    className={`stage-chevron ${isCollapsed || !isEnabled ? 'rotated' : ''}`}
                                    xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"
                                >
                                    <path d="m6 9 6 6 6-6" />
                                </svg>
                                <div style={{ display: 'flex', flexDirection: 'column' }}>
                                    <span className="pipeline-stage-title">{t(`pipeline.stage.${type}`)}</span>
                                    <span className="stage-status-text">
                                        {isEnabled ? t('pipeline.stage.enabled') : t('pipeline.stage.disabled')}
                                    </span>
                                </div>
                            </div>
                            <label className="stage-switch" onClick={(e) => e.stopPropagation()}>
                                <input
                                    type="checkbox"
                                    checked={isEnabled}
                                    onChange={handleToggleEnable}
                                />
                                <span className="stage-slider"></span>
                            </label>
                        </div>

                        <div className={`stage-settings-content ${isCollapsed || !isEnabled ? 'collapsed' : ''}`}>
                            <div className="settings-group">
                                <div className="settings-group-title">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12 2a10 10 0 1 0 10 10H12V2z" /><path d="M12 12 2.1 12a10.05 10.05 0 0 1 9.9-10v10z" /><path d="m9 16.5 3-3" /></svg>
                                    {t('pipeline.group.ai')}
                                </div>

                                <div className="settings-control">
                                    <label className="settings-label">{t('pipeline.model')}</label>
                                    <select
                                        className="settings-select"
                                        value={modelValue}
                                        onChange={(e) => {
                                            const val = e.target.value;
                                            if (val === "ADD_NEW_MODEL") {
                                                if (setCurrentPath) {
                                                    setCurrentPath('settings.api.openrouter');
                                                }
                                                return;
                                            }
                                            let field = '';
                                            if (isTranslate) field = 'translateModel';
                                            else if (isRewrite) field = 'rewriteModel';
                                            if (field) handleChange(field, val);
                                        }}
                                    >
                                        {models.map(m => <option key={m} value={m}>{m}</option>)}
                                        <option value="ADD_NEW_MODEL" style={{ color: 'var(--accent-primary)', fontWeight: 'bold' }}>
                                            + {t('pipeline.add_model')}
                                        </option>
                                    </select>
                                </div>

                                <div className="settings-control">
                                    <label className="settings-label">{t('pipeline.temperature')}</label>
                                    <div className="settings-slider-container">
                                        <input
                                            type="range"
                                            className="settings-slider"
                                            min="0"
                                            max="2"
                                            step="0.1"
                                            value={tempValue}
                                            style={{ '--range-progress': `${(tempValue / 2) * 100}%` } as React.CSSProperties}
                                            onChange={(e) => {
                                                let field = '';
                                                if (isTranslate) field = 'translateTemperature';
                                                else if (isRewrite) field = 'rewriteTemperature';
                                                if (field) handleChange(field, parseFloat(e.target.value));
                                            }}
                                        />
                                        {renderValueOrInput(isTranslate ? 'translateTemperature' : 'rewriteTemperature', tempValue, true)}
                                    </div>
                                </div>

                                <div className="settings-control">
                                    <label className="settings-label">{t('pipeline.max_tokens')}</label>
                                    <div className="settings-slider-container">
                                        <input
                                            type="range"
                                            className="settings-slider"
                                            min="0"
                                            max="128000"
                                            step="500"
                                            value={tokensValue}
                                            style={{ '--range-progress': `${(tokensValue / 128000) * 100}%` } as React.CSSProperties}
                                            onChange={(e) => {
                                                let field = '';
                                                if (isTranslate) field = 'translateMaxTokens';
                                                else if (isRewrite) field = 'rewriteMaxTokens';
                                                if (field) handleChange(field, parseInt(e.target.value));
                                            }}
                                        />
                                        {renderValueOrInput(isTranslate ? 'translateMaxTokens' : 'rewriteMaxTokens', tokensValue, false)}
                                    </div>
                                </div>
                            </div>

                            <div className="settings-group">
                                <div className="settings-group-title">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" /></svg>
                                    {t('pipeline.group.prompt')}
                                </div>
                                <div className="settings-control">
                                    <label className="settings-label">{t('pipeline.system_prompt')}</label>
                                    <textarea
                                        className="settings-textarea"
                                        value={promptValue}
                                        onChange={(e) => {
                                            let field = '';
                                            if (isTranslate) field = 'translatePrompt';
                                            else if (isRewrite) field = 'rewritePrompt';
                                            if (field) handleChange(field, e.target.value);
                                        }}
                                        placeholder={t(`pipeline.${type}.prompt_placeholder`)}
                                    />
                                </div>
                            </div>
                        </div>
                    </div>

                    {/* Stage 2.A: Voiceover */}
                    <div className={`pipeline-stage-container ${settings.voiceoverCollapsed || !settings.voiceoverEnabled ? 'is-collapsed' : ''}`}>
                        <div
                            className="pipeline-stage-header"
                            onClick={() => handleChange('voiceoverCollapsed', !settings.voiceoverCollapsed)}
                        >
                            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flex: 1 }}>
                                <svg
                                    className={`stage-chevron ${settings.voiceoverCollapsed || !settings.voiceoverEnabled ? 'rotated' : ''}`}
                                    xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"
                                >
                                    <path d="m6 9 6 6 6-6" />
                                </svg>
                                <div style={{ display: 'flex', flexDirection: 'column' }}>
                                    <span className="pipeline-stage-title">{t('pipeline.stage.voiceover')}</span>
                                    <span className="stage-status-text">
                                        {settings.voiceoverEnabled ? t('pipeline.stage.enabled') : t('pipeline.stage.disabled_simple')}
                                    </span>
                                </div>
                            </div>
                            <label className="stage-switch" onClick={(e) => e.stopPropagation()}>
                                <input
                                    type="checkbox"
                                    checked={settings.voiceoverEnabled}
                                    onChange={(e) => {
                                        const val = e.target.checked;
                                        setSettings((prev: any) => ({
                                            ...prev,
                                            voiceoverEnabled: val,
                                            voiceoverCollapsed: !val ? true : prev.voiceoverCollapsed
                                        }));
                                    }}
                                />
                                <span className="stage-slider"></span>
                            </label>
                        </div>

                        <div className={`stage-settings-content ${settings.voiceoverCollapsed || !settings.voiceoverEnabled ? 'collapsed' : ''}`}>
                            <div className="settings-group">
                                <div className="settings-control">
                                    <label className="settings-label">{t('pipeline.voiceover.service') || 'Сервіс озвучки'}</label>
                                    <select
                                        className="settings-select"
                                        value={settings.voiceoverService}
                                        onChange={(e) => {
                                            const val = e.target.value;
                                            handleChange('voiceoverService', val);
                                            if (val === 'elevenlabsbot') {
                                                fetchVoiceTemplates();
                                            } else if (val === 'voicemaker') {
                                                fetchVoiceMakerVoices();
                                            }
                                        }}
                                    >
                                        <option value="elevenlabsbot">{t('pipeline.voiceover.services.elevenlabsbot') || 'ElevenLabs Bot'}</option>
                                        <option value="elevenlabsunlim">{t('pipeline.voiceover.services.elevenlabsunlim') || 'ElevenLabs Unlim'}</option>
                                        <option value="elevenlabsua">{t('pipeline.voiceover.services.elevenlabsua') || 'ElevenLabs UA'}</option>
                                        <option value="voicemaker">{t('pipeline.voiceover.services.voicemaker') || 'VoiceMaker'}</option>
                                    </select>
                                </div>

                                {settings.voiceoverService === 'elevenlabsbot' && (
                                    <div className="settings-control">
                                        <label className="settings-label">{t('pipeline.voiceover.template') || 'Шаблон голосу'}</label>
                                        <div style={{ display: 'flex', gap: '8px' }}>
                                            <select
                                                className="settings-select"
                                                style={{ flex: 1 }}
                                                value={settings.voiceoverTemplate}
                                                onChange={(e) => handleChange('voiceoverTemplate', e.target.value)}
                                                disabled={loadingTemplates}
                                            >
                                                <option value="">{loadingTemplates ? (t('common.loading') || 'Loading...') : (t('common.select_template') || 'Select template...')}</option>
                                                {voiceTemplates.map(tpl => (
                                                    <option key={tpl} value={tpl}>{tpl}</option>
                                                ))}
                                            </select>
                                            <button
                                                className="premium-btn-sm"
                                                style={{ padding: '0 10px', height: '32px', minWidth: 'auto', display: 'flex', alignItems: 'center', justifyContent: 'center' }}
                                                onClick={() => fetchVoiceTemplates()}
                                                disabled={loadingTemplates}
                                                title={t('common.refresh') || 'Refresh'}
                                            >
                                                <svg
                                                    className={loadingTemplates ? 'animate-spin' : ''}
                                                    xmlns="http://www.w3.org/2000/svg"
                                                    width="14" height="14"
                                                    viewBox="0 0 24 24"
                                                    fill="none"
                                                    stroke="currentColor"
                                                    strokeWidth="2.5"
                                                    strokeLinecap="round"
                                                    strokeLinejoin="round"
                                                >
                                                    <path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.85.83 6.72 2.24" />
                                                    <polyline points="21 3 21 9 15 9" />
                                                </svg>
                                            </button>
                                        </div>
                                    </div>
                                )}

                                {settings.voiceoverService === 'elevenlabsunlim' && (
                                    <>
                                        <div className="settings-control">
                                            <label className="settings-label">Voice ID</label>
                                            <input
                                                className="settings-input"
                                                value={settings.elevenLabsUnlimVoiceID || ''}
                                                onChange={(e) => handleChange('elevenLabsUnlimVoiceID', e.target.value)}
                                                placeholder="AB9XsbSA..."
                                            />
                                        </div>

                                        <div className="settings-control">
                                            <label className="settings-label">{t('pipeline.voiceover.settings.stability') || 'Stability'}</label>
                                            <div className="settings-slider-container">
                                                <input
                                                    type="range"
                                                    className="settings-slider"
                                                    min="0"
                                                    max="1"
                                                    step="0.01"
                                                    value={settings.elevenLabsUnlimStability ?? 0.5}
                                                    style={{ '--range-progress': `${(settings.elevenLabsUnlimStability ?? 0.5) * 100}%` } as React.CSSProperties}
                                                    onChange={(e) => handleChange('elevenLabsUnlimStability', parseFloat(e.target.value))}
                                                />
                                                <span className="settings-slider-value">{(settings.elevenLabsUnlimStability ?? 0.5).toFixed(2)}</span>
                                            </div>
                                        </div>

                                        <div className="settings-control">
                                            <label className="settings-label">{t('pipeline.voiceover.settings.similarity') || 'Similarity'}</label>
                                            <div className="settings-slider-container">
                                                <input
                                                    type="range"
                                                    className="settings-slider"
                                                    min="0"
                                                    max="1"
                                                    step="0.01"
                                                    value={settings.elevenLabsUnlimSimilarity ?? 0.75}
                                                    style={{ '--range-progress': `${(settings.elevenLabsUnlimSimilarity ?? 0.75) * 100}%` } as React.CSSProperties}
                                                    onChange={(e) => handleChange('elevenLabsUnlimSimilarity', parseFloat(e.target.value))}
                                                />
                                                <span className="settings-slider-value">{(settings.elevenLabsUnlimSimilarity ?? 0.75).toFixed(2)}</span>
                                            </div>
                                        </div>

                                        <div className="settings-control">
                                            <label className="settings-label">{t('pipeline.voiceover.settings.style') || 'Style Exaggeration'}</label>
                                            <div className="settings-slider-container">
                                                <input
                                                    type="range"
                                                    className="settings-slider"
                                                    min="0"
                                                    max="1"
                                                    step="0.01"
                                                    value={settings.elevenLabsUnlimStyle ?? 0}
                                                    style={{ '--range-progress': `${(settings.elevenLabsUnlimStyle ?? 0) * 100}%` } as React.CSSProperties}
                                                    onChange={(e) => handleChange('elevenLabsUnlimStyle', parseFloat(e.target.value))}
                                                />
                                                <span className="settings-slider-value">{(settings.elevenLabsUnlimStyle ?? 0).toFixed(2)}</span>
                                            </div>
                                        </div>

                                        <div className="settings-control">
                                            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                                                <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.voiceover.settings.speaker_boost') || 'Speaker Boost'}</label>
                                                <label className="stage-switch small">
                                                    <input
                                                        type="checkbox"
                                                        checked={settings.elevenLabsUnlimSpeakerBoost}
                                                        onChange={(e) => handleChange('elevenLabsUnlimSpeakerBoost', e.target.checked)}
                                                    />
                                                    <span className="stage-slider"></span>
                                                </label>
                                            </div>
                                        </div>
                                    </>
                                )}

                                {settings.voiceoverService === 'elevenlabsua' && (
                                    <>
                                        <div className="settings-control">
                                            <label className="settings-label">Voice ID</label>
                                            <input
                                                className="settings-input"
                                                value={settings.elevenLabsUAVoiceID || ''}
                                                onChange={(e) => handleChange('elevenLabsUAVoiceID', e.target.value)}
                                                placeholder="eBthAb30..."
                                            />
                                        </div>

                                        <div className="settings-control">
                                            <label className="settings-label">Model ID</label>
                                            <select
                                                className="settings-select"
                                                value={settings.elevenLabsUAModel || 'eleven_multilingual_v2'}
                                                onChange={(e) => handleChange('elevenLabsUAModel', e.target.value)}
                                            >
                                                <option value="eleven_multilingual_v2">Multilingual v2</option>
                                                <option value="eleven_flash_v2_5">Flash v2.5</option>
                                                <option value="eleven_turbo_v2_5">Turbo v2.5</option>
                                                <option value="eleven_multilingual_v3">v3 (Emotions)</option>
                                            </select>
                                        </div>

                                        <div className="settings-control">
                                            <label className="settings-label">{t('pipeline.voiceover.settings.stability') || 'Stability'}</label>
                                            <div className="settings-slider-container">
                                                <input
                                                    type="range"
                                                    className="settings-slider"
                                                    min="0"
                                                    max="1"
                                                    step="0.01"
                                                    value={settings.elevenLabsUAStability ?? 0.5}
                                                    style={{ '--range-progress': `${(settings.elevenLabsUAStability ?? 0.5) * 100}%` } as React.CSSProperties}
                                                    onChange={(e) => handleChange('elevenLabsUAStability', parseFloat(e.target.value))}
                                                />
                                                <span className="settings-slider-value">{(settings.elevenLabsUAStability ?? 0.5).toFixed(2)}</span>
                                            </div>
                                        </div>

                                        <div className="settings-control">
                                            <label className="settings-label">{t('pipeline.voiceover.settings.similarity') || 'Similarity'}</label>
                                            <div className="settings-slider-container">
                                                <input
                                                    type="range"
                                                    className="settings-slider"
                                                    min="0"
                                                    max="1"
                                                    step="0.01"
                                                    value={settings.elevenLabsUASimilarity ?? 0.75}
                                                    style={{ '--range-progress': `${(settings.elevenLabsUASimilarity ?? 0.75) * 100}%` } as React.CSSProperties}
                                                    onChange={(e) => handleChange('elevenLabsUASimilarity', parseFloat(e.target.value))}
                                                />
                                                <span className="settings-slider-value">{(settings.elevenLabsUASimilarity ?? 0.75).toFixed(2)}</span>
                                            </div>
                                        </div>

                                        <div className="settings-control">
                                            <label className="settings-label">{t('pipeline.voiceover.settings.style') || 'Style Exaggeration'}</label>
                                            <div className="settings-slider-container">
                                                <input
                                                    type="range"
                                                    className="settings-slider"
                                                    min="0"
                                                    max="1"
                                                    step="0.01"
                                                    value={settings.elevenLabsUAStyle ?? 0}
                                                    style={{ '--range-progress': `${(settings.elevenLabsUAStyle ?? 0) * 100}%` } as React.CSSProperties}
                                                    onChange={(e) => handleChange('elevenLabsUAStyle', parseFloat(e.target.value))}
                                                />
                                                <span className="settings-slider-value">{(settings.elevenLabsUAStyle ?? 0).toFixed(2)}</span>
                                            </div>
                                        </div>

                                        <div className="settings-control">
                                            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                                                <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.voiceover.settings.speaker_boost') || 'Speaker Boost'}</label>
                                                <label className="stage-switch small">
                                                    <input
                                                        type="checkbox"
                                                        checked={settings.elevenLabsUASpeakerBoost}
                                                        onChange={(e) => handleChange('elevenLabsUASpeakerBoost', e.target.checked)}
                                                    />
                                                    <span className="stage-slider"></span>
                                                </label>
                                            </div>
                                        </div>
                                    </>
                                )}

                                {settings.voiceoverService === 'voicemaker' && (
                                    <>
                                        <div className="settings-control">
                                            <label className="settings-label">{t('pipeline.voiceover.template') || 'Голос'}</label>
                                            <div style={{ display: 'flex', gap: '8px' }}>
                                                <SearchableSelect
                                                    options={(voiceMakerVoices || []).map(v => ({
                                                        value: v.VoiceId,
                                                        label: v.VoiceWebname,
                                                        subLabel: v.LanguageName
                                                    }))}
                                                    value={settings.voiceMakerVoiceID}
                                                    onChange={(val) => {
                                                        const vInfo = voiceMakerVoices.find(v => v.VoiceId === val);
                                                        setSettings((prev: any) => ({
                                                            ...prev,
                                                            voiceMakerVoiceID: val,
                                                            voiceMakerLanguageCode: vInfo?.LanguageCode || 'multi-lang'
                                                        }));
                                                    }}
                                                    loading={loadingTemplates}
                                                    placeholder="Виберіть голос..."
                                                    searchPlaceholder={t('common.search') || 'Пошук...'}
                                                />
                                                <button
                                                    className="premium-btn-sm"
                                                    style={{ padding: '0 10px', height: '32px', minWidth: 'auto', display: 'flex', alignItems: 'center', justifyContent: 'center' }}
                                                    onClick={() => fetchVoiceMakerVoices()}
                                                    disabled={loadingTemplates}
                                                >
                                                    <svg
                                                        className={loadingTemplates ? 'animate-spin' : ''}
                                                        xmlns="http://www.w3.org/2000/svg"
                                                        width="14" height="14"
                                                        viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"
                                                    >
                                                        <path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.85.83 6.72 2.24" />
                                                        <polyline points="21 3 21 9 15 9" />
                                                    </svg>
                                                </button>
                                            </div>
                                        </div>

                                        <div className="settings-control">
                                            <label className="settings-label">Max Chars per Request</label>
                                            <div className="settings-slider-container">
                                                <input
                                                    type="range"
                                                    className="settings-slider"
                                                    min="500"
                                                    max="10000"
                                                    step="100"
                                                    value={settings.voiceMakerCharLimit ?? 3000}
                                                    style={{ '--range-progress': `${((settings.voiceMakerCharLimit ?? 3000) - 500) / 9500 * 100}%` } as React.CSSProperties}
                                                    onChange={(e) => handleChange('voiceMakerCharLimit', parseInt(e.target.value))}
                                                />
                                                <span className="settings-slider-value">{settings.voiceMakerCharLimit ?? 3000}</span>
                                            </div>
                                        </div>
                                    </>
                                )}
                            </div>
                        </div>
                    </div>
                    {/* Stage 2.B: Images */}
                    <div className={`pipeline-stage-container ${settings.imageCollapsed || !settings.imageEnabled ? 'is-collapsed' : ''}`}>
                        <div
                            className="pipeline-stage-header"
                            onClick={() => handleChange('imageCollapsed', !settings.imageCollapsed)}
                        >
                            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flex: 1 }}>
                                <svg
                                    className={`stage-chevron ${settings.imageCollapsed || !settings.imageEnabled ? 'rotated' : ''}`}
                                    xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"
                                >
                                    <path d="m6 9 6 6 6-6" />
                                </svg>
                                <div style={{ display: 'flex', flexDirection: 'column' }}>
                                    <span className="pipeline-stage-title">{t('pipeline.stage.image')}</span>
                                    <span className="stage-status-text">
                                        {settings.imageEnabled ? t('pipeline.stage.enabled') : t('pipeline.stage.disabled_simple')}
                                    </span>
                                </div>
                            </div>
                            <label className="stage-switch" onClick={(e) => e.stopPropagation()}>
                                <input
                                    type="checkbox"
                                    checked={settings.imageEnabled}
                                    onChange={(e) => {
                                        const val = e.target.checked;
                                        setSettings((prev: any) => ({
                                            ...prev,
                                            imageEnabled: val,
                                            imageCollapsed: !val ? true : prev.imageCollapsed
                                        }));
                                    }}
                                />
                                <span className="stage-slider"></span>
                            </label>
                        </div>

                        <div className={`stage-settings-content ${settings.imageCollapsed || !settings.imageEnabled ? 'collapsed' : ''}`}>
                            <div className="settings-group">
                                <div className="settings-group-title">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg>
                                    {t('pipeline.group.prompt')}
                                </div>
                                <div className="settings-control">
                                    <label className="settings-label">{t('pipeline.image.generation_method') || 'Метод генерации задач'}</label>
                                    <div className="settings-description" style={{ fontSize: '11px', color: 'var(--text-secondary)', marginBottom: '8px' }}>
                                        {t('pipeline.image.generation_desc') || 'Выберите как разбить текст на отдельные промпты'}
                                    </div>
                                    <div style={{ display: 'flex', background: 'var(--bg-tertiary)', borderRadius: '8px', padding: '4px', gap: '4px' }}>
                                        <button
                                            className={`method-toggle-btn ${settings.imageGenerationMethod === 'lines' ? 'active' : ''}`}
                                            onClick={() => handleChange('imageGenerationMethod', 'lines')}
                                            style={{
                                                flex: 1, padding: '6px', borderRadius: '6px', border: 'none',
                                                background: settings.imageGenerationMethod === 'lines' ? 'var(--bg-primary)' : 'transparent',
                                                color: settings.imageGenerationMethod === 'lines' ? 'var(--text-primary)' : 'var(--text-secondary)',
                                                cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '6px',
                                                fontSize: '12px', fontWeight: settings.imageGenerationMethod === 'lines' ? 500 : 400,
                                                boxShadow: settings.imageGenerationMethod === 'lines' ? '0 2px 4px rgba(0,0,0,0.1)' : 'none',
                                                transition: 'all 0.2s'
                                            }}
                                        >
                                            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><line x1="8" y1="6" x2="21" y2="6"></line><line x1="8" y1="12" x2="21" y2="12"></line><line x1="8" y1="18" x2="21" y2="18"></line><line x1="3" y1="6" x2="3.01" y2="6"></line><line x1="3" y1="12" x2="3.01" y2="12"></line><line x1="3" y1="18" x2="3.01" y2="18"></line></svg>
                                            {t('pipeline.image.lines') || 'Строки'}
                                        </button>
                                        <button
                                            className={`method-toggle-btn ${settings.imageGenerationMethod !== 'lines' ? 'active' : ''}`}
                                            onClick={() => handleChange('imageGenerationMethod', 'sentences')}
                                            style={{
                                                flex: 1, padding: '6px', borderRadius: '6px', border: 'none',
                                                background: settings.imageGenerationMethod !== 'lines' ? 'var(--bg-primary)' : 'transparent',
                                                color: settings.imageGenerationMethod !== 'lines' ? 'var(--text-primary)' : 'var(--text-secondary)',
                                                cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '6px',
                                                fontSize: '12px', fontWeight: settings.imageGenerationMethod !== 'lines' ? 500 : 400,
                                                boxShadow: settings.imageGenerationMethod !== 'lines' ? '0 2px 4px rgba(0,0,0,0.1)' : 'none',
                                                transition: 'all 0.2s'
                                            }}
                                        >
                                            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="4 7 4 4 20 4 20 7"></polyline><line x1="9" y1="20" x2="15" y2="20"></line><line x1="12" y1="4" x2="12" y2="20"></line></svg>
                                            {t('pipeline.image.sentences') || 'Предложения'}
                                        </button>
                                    </div>
                                </div>

                                {settings.imageGenerationMethod !== 'lines' && (
                                    <div className="settings-control">
                                        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                                            <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.image.group_limit') || 'Группировать по лимиту символов'}</label>
                                            <label className="stage-switch small">
                                                <input
                                                    type="checkbox"
                                                    checked={settings.imageGroupSentences}
                                                    onChange={(e) => handleChange('imageGroupSentences', e.target.checked)}
                                                />
                                                <span className="stage-slider"></span>
                                            </label>
                                        </div>
                                        <div className="settings-description" style={{ fontSize: '11px', color: 'var(--text-secondary)', marginTop: '8px', marginBottom: '12px' }}>
                                            {settings.imageGroupSentences
                                                ? (t('pipeline.image.group_limit_desc') || 'Предложения будут объединены до достижения лимита символов')
                                                : (t('pipeline.image.group_limit_desc_off') || 'Каждое предложение будет разделено буквально как отдельный промпт')}
                                        </div>

                                        {settings.imageGroupSentences && (
                                            <div className="settings-slider-container" style={{ marginTop: '8px' }}>
                                                <span style={{ fontSize: '11px', color: 'var(--text-secondary)' }}>{t('pipeline.image.symbol_limit') || 'Лимит символов:'} {settings.imageSentenceLimit ?? 1000}</span>
                                                <input
                                                    type="range"
                                                    className="settings-slider"
                                                    min="100"
                                                    max="5000"
                                                    step="100"
                                                    value={settings.imageSentenceLimit ?? 1000}
                                                    style={{ '--range-progress': `${((settings.imageSentenceLimit ?? 1000) - 100) / 4900 * 100}%`, marginTop: '8px', width: '100%' } as React.CSSProperties}
                                                    onChange={(e) => handleChange('imageSentenceLimit', parseInt(e.target.value))}
                                                />
                                            </div>
                                        )}
                                    </div>
                                )}

                                <div className="settings-control">
                                    <label className="settings-label">{t('pipeline.image.prompt') || 'Промт для інструкцій'}</label>
                                    <textarea
                                        className="settings-textarea"
                                        style={{ height: '80px', resize: 'vertical' }}
                                        value={settings.imagePrompt || ''}
                                        onChange={(e) => handleChange('imagePrompt', e.target.value)}
                                        placeholder={t('pipeline.image.prompt_placeholder') || 'Введіть промт...'}
                                    />
                                    {content && content.trim() !== '' && (
                                        <div style={{ fontSize: '11px', color: 'var(--accent-primary)', marginTop: '8px', fontWeight: 500, display: 'flex', alignItems: 'center', gap: '6px' }}>
                                            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10"></circle><polyline points="12 6 12 12 16 14"></polyline></svg>
                                            {t('pipeline.image.estimated_chunks') || 'Орієнтовна кількість промптів: '} {estimatedChunks}
                                        </div>
                                    )}
                                </div>
                            </div>

                            <div className="settings-group">
                                <div className="settings-group-title">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12 2a10 10 0 1 0 10 10H12V2z" /><path d="M12 12 2.1 12a10.05 10.05 0 0 1 9.9-10v10z" /><path d="m9 16.5 3-3" /></svg>
                                    {t('pipeline.group.ai')}
                                </div>

                                <div className="settings-control">
                                    <label className="settings-label">{t('pipeline.model')}</label>
                                    <select
                                        className="settings-select"
                                        value={settings.imagePromptModel || ''}
                                        onChange={(e) => {
                                            const val = e.target.value;
                                            if (val === "ADD_NEW_MODEL") {
                                                if (setCurrentPath) setCurrentPath('settings.api.openrouter');
                                                return;
                                            }
                                            handleChange('imagePromptModel', val);
                                        }}
                                    >
                                        <option value="">{t('pipeline.model.default')}</option>
                                        {models.map(m => <option key={m} value={m}>{m}</option>)}
                                        <option value="ADD_NEW_MODEL" style={{ color: 'var(--accent-primary)', fontWeight: 'bold' }}>
                                            + {t('pipeline.add_model')}
                                        </option>
                                    </select>
                                </div>

                                <div className="settings-control">
                                    <label className="settings-label">{t('pipeline.temperature')}</label>
                                    <div className="settings-slider-container">
                                        <input
                                            type="range"
                                            className="settings-slider"
                                            min="0"
                                            max="2"
                                            step="0.1"
                                            value={settings.imagePromptTemperature ?? 0.7}
                                            style={{ '--range-progress': `${((settings.imagePromptTemperature ?? 0.7) / 2) * 100}%` } as React.CSSProperties}
                                            onChange={(e) => handleChange('imagePromptTemperature', parseFloat(e.target.value))}
                                        />
                                        {renderValueOrInput('imagePromptTemperature', settings.imagePromptTemperature ?? 0.7, true)}
                                    </div>
                                </div>

                                <div className="settings-control">
                                    <label className="settings-label">{t('pipeline.max_tokens')}</label>
                                    <div className="settings-slider-container">
                                        <input
                                            type="range"
                                            className="settings-slider"
                                            min="0"
                                            max="128000"
                                            step="500"
                                            value={settings.imagePromptMaxTokens ?? 0}
                                            style={{ '--range-progress': `${((settings.imagePromptMaxTokens ?? 0) / 128000) * 100}%` } as React.CSSProperties}
                                            onChange={(e) => handleChange('imagePromptMaxTokens', parseFloat(e.target.value))}
                                        />
                                        {renderValueOrInput('imagePromptMaxTokens', settings.imagePromptMaxTokens ?? 0, false)}
                                    </div>
                                </div>
                            </div>

                            <div className="settings-group">
                                <div className="settings-group-title">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polygon points="12 2 2 7 12 12 22 7 12 2"></polygon><polyline points="2 17 12 22 22 17"></polyline><polyline points="2 12 12 17 22 12"></polyline></svg>
                                    {t('pipeline.group.provider')}
                                </div>
                                <div className="settings-control">
                                    <label className="settings-label">{t('pipeline.image.service')}</label>
                                    <select
                                        className="settings-select"
                                        value={settings.imageService}
                                        onChange={(e) => {
                                            const val = e.target.value;
                                            handleChange('imageService', val);
                                            if (val === 'pollinations') {
                                                fetchPollinationsModels();
                                            }
                                        }}
                                    >
                                        <option value="pollinations">{t('image.pollinationsai') || 'Pollinations.ai'}</option>
                                        <option value="googler">{t('image.googler') || 'Googler'}</option>
                                        <option value="elevenlabsimage">{t('image.elevenlabsimage') || 'ElevenLabs Image'}</option>
                                    </select>
                                </div>

                                {settings.imageService === 'pollinations' && (
                                    <>
                                        <div className="settings-control">
                                            <label className="settings-label">{t('pipeline.image.model')}</label>
                                            <div style={{ display: 'flex', gap: '8px' }}>
                                                <select
                                                    className="settings-select"
                                                    style={{ flex: 1 }}
                                                    value={settings.imageModel}
                                                    onChange={(e) => handleChange('imageModel', e.target.value)}
                                                    onFocus={() => {
                                                        if (pollinationsModels.length === 0) fetchPollinationsModels();
                                                    }}
                                                >
                                                    <option value="">{loadingPollinationsModels ? t('common.loading') : t('pipeline.model.default')}</option>
                                                    {pollinationsModels.map(m => (
                                                        <option key={m} value={m}>{m}</option>
                                                    ))}
                                                </select>
                                                <button
                                                    className="premium-btn-sm"
                                                    style={{ padding: '0 10px', height: '32px', minWidth: 'auto', display: 'flex', alignItems: 'center', justifyContent: 'center' }}
                                                    onClick={() => fetchPollinationsModels()}
                                                    disabled={loadingPollinationsModels}
                                                >
                                                    <svg
                                                        className={loadingPollinationsModels ? 'animate-spin' : ''}
                                                        xmlns="http://www.w3.org/2000/svg"
                                                        width="14" height="14"
                                                        viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"
                                                    >
                                                        <path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.85.83 6.72 2.24" />
                                                        <polyline points="21 3 21 9 15 9" />
                                                    </svg>
                                                </button>
                                            </div>
                                        </div>

                                        <div className="settings-row" style={{ gap: '16px' }}>
                                            <div className="settings-control" style={{ flex: 1 }}>
                                                <label className="settings-label">{t('pipeline.image.width')}</label>
                                                <input
                                                    type="number"
                                                    className="settings-input"
                                                    value={settings.imageWidth || 1920}
                                                    onChange={(e) => handleChange('imageWidth', parseInt(e.target.value))}
                                                />
                                            </div>
                                            <div className="settings-control" style={{ flex: 1 }}>
                                                <label className="settings-label">{t('pipeline.image.height')}</label>
                                                <input
                                                    type="number"
                                                    className="settings-input"
                                                    value={settings.imageHeight || 1080}
                                                    onChange={(e) => handleChange('imageHeight', parseInt(e.target.value))}
                                                />
                                            </div>
                                        </div>

                                        <div className="settings-control">
                                            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                                                <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.image.nologo')}</label>
                                                <label className="stage-switch small">
                                                    <input
                                                        type="checkbox"
                                                        checked={settings.imageNoLogo}
                                                        onChange={(e) => handleChange('imageNoLogo', e.target.checked)}
                                                    />
                                                    <span className="stage-slider"></span>
                                                </label>
                                            </div>
                                        </div>

                                        <div className="settings-control">
                                            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                                                <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.image.enhance')}</label>
                                                <label className="stage-switch small">
                                                    <input
                                                        type="checkbox"
                                                        checked={settings.imageEnhance}
                                                        onChange={(e) => handleChange('imageEnhance', e.target.checked)}
                                                    />
                                                    <span className="stage-slider"></span>
                                                </label>
                                            </div>
                                        </div>
                                    </>
                                )}
                            </div>
                        </div>
                    </div>
                </div>

                <div className="pipeline-sidebar-footer">
                    <div className="footer-actions">
                        <button
                            className="add-to-queue-btn"
                            onClick={() => {
                                if (content.trim()) {
                                    setIsModalOpen(true);
                                }
                            }}
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                                <line x1="12" y1="5" x2="12" y2="19"></line>
                                <line x1="5" y1="12" x2="19" y2="12"></line>
                            </svg>
                            {selectedTemplateIds.filter(id => templates.find(t => t.id === id)?.type === type).length > 0
                                ? `${t('pipeline.add_to_queue')} (${selectedTemplateIds.filter(id => templates.find(t => t.id === id)?.type === type).length})`
                                : t('pipeline.add_to_queue')}
                        </button>
                    </div>
                </div>
            </div>

            <TaskNameModal
                isOpen={isModalOpen}
                onClose={() => setIsModalOpen(false)}
                onConfirm={handleAddTask}
            />

            <button
                className={`sidebar-floating-toggle ${isOpen ? 'is-open' : ''}`}
                onClick={onToggle}
                title={isOpen ? t('pipeline.hide_settings') : t('pipeline.show_settings')}
            >
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" /></svg>
            </button>
            <ConfirmModal
                isOpen={!!templateToDelete}
                onClose={() => setTemplateToDelete(null)}
                onConfirm={handleConfirmDelete}
                title={t('common.delete')}
                message={t('templatesTab.delete_confirm')}
            />
        </aside>
    );
};
