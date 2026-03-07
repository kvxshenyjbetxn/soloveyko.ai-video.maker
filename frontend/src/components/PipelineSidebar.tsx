import React, { useState, useEffect, useRef, useCallback, useMemo } from 'react';
import './PipelineSidebar.css';
import { useI18n } from '../contexts/I18nContext';
import { useQueue } from '../contexts/QueueContext';
import { useToast } from '../contexts/ToastContext';
import { useServices } from '../contexts/ServiceContext';
import { useTemplates } from '../contexts/TemplateContext';
// @ts-ignore
import { GetPipelineSettings, SavePipelineSettings, GetOpenRouterSavedModels, SelectDirectory, GetDefaultVideosPath, GetElevenLabsBotVoiceTemplates, GetVoiceMakerVoices, GetPollinationsImageModels, GetPollinationsSavedModels, SavePollinationsModels, GetEdgeTTSVoices, CheckExistingTask, CheckExistingTasks, AddToHistory } from '../../wailsjs/go/main/App';
// @ts-ignore
import { EventsOn } from '../../wailsjs/runtime/runtime';
import voicemakerVoicesData from '../assets/voicemaker_voices.json';

import { TaskNameModal } from './TaskNameModal';
import { ConfirmModal } from './ConfirmModal';
import { ExistingFilesModal } from './ExistingFilesModal';

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
import { CustomStagesSection } from './pipeline-sidebar/CustomStagesSection';
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
    const { showToast } = useToast();
    const { addTasks, addTask, getNextTaskName } = useQueue();
    const { openRouterKeys, elevenLabsBotKeys, elevenLabsUnlimKeys, elevenLabsUAKeys, voiceMakerKeys, pollinationsKeys, elevenLabsImageKeys } = useServices();
    const [settings, setSettings] = useState<any>(null);
    const [models, setModels] = useState<string[]>([]);
    const [isResizing, setIsResizing] = useState(false);
    const [editingField, setEditingField] = useState<string | null>(null);
    const [isModalOpen, setIsModalOpen] = useState(false);
    const [templateToDelete, setTemplateToDelete] = useState<any | null>(null);
    const [existingFilesData, setExistingFilesData] = useState<any>(null);
    const [pendingTaskName, setPendingTaskName] = useState<string>("");
    const [pendingContent, setPendingContent] = useState<string>("");
    const [historyOverride, setHistoryOverride] = useState<{ content: string, templateIds: string[], taskName: string } | null>(null);
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
        const initialCount = settings.imageInitialSentenceCount ?? 0;

        let baseSegments: string[] = [];
        if (method === 'lines') {
            baseSegments = content.split('\n').map(l => l.trim()).filter(l => l);
        } else {
            // Updated splitter to match Go backend precisely
            const splitSentences = (text: string) => {
                const sentences: string[] = [];
                let current = "";
                const chars = [...text];
                for (let i = 0; i < chars.length; i++) {
                    current += chars[i];
                    if (chars[i] === '.' || chars[i] === '!' || chars[i] === '?') {
                        if (i + 1 === chars.length || " \n\t\r".includes(chars[i + 1])) {
                            sentences.push(current.trim());
                            current = "";
                        }
                    }
                }
                if (current.trim()) sentences.push(current.trim());
                return sentences.filter(s => s);
            };
            baseSegments = splitSentences(content);
        }

        if (baseSegments.length === 0) return 0;

        let chunksNum = 0;
        let currentBaseIdx = 0;

        // Backend initialCount logic with 50 chars constraint
        if (initialCount > 0) {
            for (let dynamicChunkIdx = 0; dynamicChunkIdx < initialCount && currentBaseIdx < baseSegments.length; dynamicChunkIdx++) {
                let currentChunk = baseSegments[currentBaseIdx];
                currentBaseIdx++;

                // Merge next segments until we hit 50 chars (hook/dynamic start constraint)
                while ([...currentChunk].length < 50 && currentBaseIdx < baseSegments.length) {
                    currentChunk += " " + baseSegments[currentBaseIdx];
                    currentBaseIdx++;
                }
                chunksNum++;
            }
        }

        const remaining = baseSegments.slice(currentBaseIdx);

        if (remaining.length > 0) {
            // For lines mode, if initial count is set, backend forces grouping for the rest
            const shouldGroup = (method === 'lines' && initialCount > 0) || (method === 'sentences' && group);

            if (!shouldGroup) {
                chunksNum += remaining.length;
            } else {
                let currentLen = 0;
                for (const segment of remaining) {
                    const segLen = [...segment].length;
                    if (currentLen === 0) {
                        currentLen = segLen;
                    } else if (currentLen + 1 + segLen <= limit) {
                        currentLen += 1 + segLen;
                    } else {
                        chunksNum++;
                        currentLen = segLen;
                    }
                }
                if (currentLen > 0) chunksNum++;
            }
        }

        return chunksNum;
    }, [content, settings?.imageGenerationMethod, settings?.imageGroupSentences, settings?.imageSentenceLimit, settings?.imageInitialSentenceCount]);

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

                if (elevenLabsImageKeys.length > 0 && !s.elevenLabsImageKeyID) {
                    s.elevenLabsImageKeyID = elevenLabsImageKeys[0].id;
                    updated = true;
                }

                if (elevenLabsUnlimKeys.length > 0 && !s.voiceoverElevenLabsUnlimKeyID) {
                    s.voiceoverElevenLabsUnlimKeyID = elevenLabsUnlimKeys[0].id;
                    updated = true;
                }

                if (!s.rewriteEnabled) { s.rewriteEnabled = true; updated = true; }
                if (s.voiceoverEnabled === undefined) { s.voiceoverEnabled = false; updated = true; }

                if (!s.translateOutputPath || !s.rewriteOutputPath) {
                    const def = await GetDefaultVideosPath();
                    if (def) {
                        if (!s.translateOutputPath) s.translateOutputPath = s.outputPath || def;
                        if (!s.rewriteOutputPath) s.rewriteOutputPath = s.outputPath || def;
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
                if (s.montageIntroVideoEnabled === undefined) { s.montageIntroVideoEnabled = false; updated = true; }
                if (s.montageIntroVideoPath === undefined) { s.montageIntroVideoPath = ""; updated = true; }
                if (s.montageWatermarkEnabled === undefined) { s.montageWatermarkEnabled = false; updated = true; }
                if (s.montageWatermarkPath === undefined) { s.montageWatermarkPath = ""; updated = true; }
                if (s.montageWatermarkPosition === undefined) { s.montageWatermarkPosition = "bottom-right"; updated = true; }
                if (s.montageWatermarkOpacity === undefined) { s.montageWatermarkOpacity = 0.8; updated = true; }
                if (s.montageWatermarkSize === undefined) { s.montageWatermarkSize = 15; updated = true; }
                if (s.montageWatermarkOnIntro === undefined) { s.montageWatermarkOnIntro = false; updated = true; }
                if (s.montageOverlayPath === undefined) { s.montageOverlayPath = ""; updated = true; }
                if (s.montageOverlayOnIntro === undefined) { s.montageOverlayOnIntro = false; updated = true; }
                if (s.montageOverlayTriggersEnabled === undefined) { s.montageOverlayTriggersEnabled = false; updated = true; }
                if (s.montageOverlayTriggers === undefined) { s.montageOverlayTriggers = []; updated = true; }

                if (s.customStages === undefined) { s.customStages = []; updated = true; }
                if (s.customStagesEnabled === undefined) { s.customStagesEnabled = true; updated = true; }
                if (s.customStagesCollapsed === undefined) { s.customStagesCollapsed = true; updated = true; }

                if (s.translateTemperature === undefined) s.translateTemperature = 0.7;
                if (s.rewriteTemperature === undefined) s.rewriteTemperature = 0.7;
                if (s.translateMaxTokens === undefined) s.translateMaxTokens = 0;
                if (s.rewriteMaxTokens === undefined) s.rewriteMaxTokens = 0;

                if (s.imagePromptModel === undefined) s.imagePromptModel = modelList.length > 0 ? modelList[0] : "";
                if (s.imagePromptTemperature === undefined) s.imagePromptTemperature = 0.7;
                if (s.imagePromptMaxTokens === undefined) s.imagePromptMaxTokens = 0;
                if (s.imageMode === undefined) s.imageMode = "normal";
                if (s.imageMemoryType === undefined) s.imageMemoryType = "primitive";
                if (s.imageMemoryChars === undefined) s.imageMemoryChars = 1000;
                if (s.imageDetermineCharacters === undefined) s.imageDetermineCharacters = false;
                if (s.imageDetermineCharactersPrompt === undefined) s.imageDetermineCharactersPrompt = "";

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
                if (s.subtitleMaxLen === undefined) s.subtitleMaxLen = 40;
                if (s.subtitleMaxWords === undefined) s.subtitleMaxWords = 10;
                if (s.subtitleAmdLanguage === undefined) s.subtitleAmdLanguage = 'uk';
                if (s.subtitleFont === undefined) s.subtitleFont = 'Arial';
                if (s.subtitleSize === undefined) s.subtitleSize = 24;
                if (s.subtitleColor === undefined) s.subtitleColor = '#ffffff';
                if (s.subtitleFadeEnabled === undefined) s.subtitleFadeEnabled = true;
                if (s.subtitleFadeIn === undefined) s.subtitleFadeIn = 300;
                if (s.subtitleFadeOut === undefined) s.subtitleFadeOut = 300;
                if (s.subtitleKaraokeEffect === undefined) s.subtitleKaraokeEffect = false;
                if (s.subtitleWhisperxLanguage === undefined) s.subtitleWhisperxLanguage = '';

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







    const handleChange = (field: string, value: any) => {
        setSettings((prev: any) => ({ ...prev, [field]: value }));
    };

    const handleSaveTemplate = async () => {
        const name = (type === 'translate' ? settings.translatePipelineName : (type === 'rewrite' ? settings.rewritePipelineName : settings.voiceoverPipelineName)) || `Template ${templates.length + 1}`;
        const templateData: any = {
            api: {},
            stages: {
                [type]: type === 'translate' ? settings.translateEnabled : settings.rewriteEnabled,
                voiceover: settings.voiceoverEnabled,
                image: settings.imageEnabled,
                subtitle: settings.subtitleEnabled,
                montage: settings.montageEnabled
            },
            control: {
                translate: settings.translateControlEnabled,
                image: settings.imageControlEnabled
            },
            text: {},
            voiceover: {
                services: {}
            },
            image: {
                services: {}
            },
            subtitle: {},
            montage: {},
            customStages: settings.customStages || [],
            customStagesEnabled: settings.customStagesEnabled || false,
            customStagesCollapsed: settings.customStagesCollapsed || false
        };

        // 0. API Keys Group
        const apiKeys = Object.keys(settings).filter(k => k.endsWith('KeyID'));
        apiKeys.forEach(k => {
            // Exclude irrelevant text pipeline keys
            if (type === 'translate' && k === 'rewriteOpenRouterKeyID') return;
            if (type === 'rewrite' && k === 'translateOpenRouterKeyID') return;

            if (settings[k] !== undefined) templateData.api[k] = settings[k];
        });

        // 1. Text Pipeline Settings (Translate/Rewrite)
        Object.keys(settings).forEach(key => {
            if (key.startsWith(type)) {
                // We EXCLUDE Collapsed UI state, but INCLUDE Enabled, OutputPath and PipelineName
                // so the search for existing files and loading works correctly.
                if (key.endsWith('Collapsed') || key.endsWith('KeyID')) return;
                templateData.text[key] = settings[key];
            }
        });

        // Also save major enabling flags and OUTPUT PATHS so the template "knows" what it does
        ['voiceoverEnabled', 'imageEnabled', 'subtitleEnabled', 'montageEnabled', 'translateControlEnabled', 'imageControlEnabled',
            'translateOutputPath', 'rewriteOutputPath'
        ].forEach(key => {
            if (settings[key] !== undefined) {
                templateData.text[key] = settings[key];
            }
        });

        // 2. Voiceover Settings
        const voiceoverBaseFields = ['voiceoverService'];
        voiceoverBaseFields.forEach(f => { if (settings[f] !== undefined) templateData.voiceover[f] = settings[f]; });

        // Voiceover Service Specific Groups
        const voServices: any = {
            elevenlabsbot: ['voiceoverTemplate'],
            elevenlabsunlim: ['elevenLabsUnlimVoiceID', 'elevenLabsUnlimStability', 'elevenLabsUnlimSimilarity', 'elevenLabsUnlimStyle', 'elevenLabsUnlimSpeakerBoost'],
            elevenlabsua: ['elevenLabsUAVoiceID', 'elevenLabsUAStability', 'elevenLabsUASimilarity', 'elevenLabsUAStyle', 'elevenLabsUASpeakerBoost', 'elevenLabsUAModel'],
            voicemaker: ['voiceMakerVoiceID', 'voiceMakerLanguageCode', 'voiceMakerCharLimit'],
            edgetts: ['edgeTTSVoiceID', 'edgeTTSRate', 'edgeTTSPitch', 'edgeTTSVolume']
        };

        Object.entries(voServices).forEach(([svc, fields]: [string, any]) => {
            templateData.voiceover.services[svc] = {};
            fields.forEach((f: string) => {
                if (settings[f] !== undefined) templateData.voiceover.services[svc][f] = settings[f];
            });
        });

        // 3. Image Settings
        const imageBaseFields = ['imageService', 'imageMode', 'imageMemoryType', 'imageMemoryChars', 'imageGenerationMethod', 'imageGroupSentences', 'imageSentenceLimit', 'imageInitialSentenceCount', 'imagePromptModel', 'imagePromptTemperature', 'imagePromptMaxTokens', 'imageDetermineCharacters', 'imageDetermineCharactersPrompt'];
        imageBaseFields.forEach(f => { if (settings[f] !== undefined) templateData.image[f] = settings[f]; });

        // Image Service Specific Groups
        const imgServices: any = {
            pollinations: ['imageModel', 'imageWidth', 'imageHeight', 'imageNoLogo', 'imageEnhance', 'imagePrompt'],
            googler: ['imageGooglerModel', 'imageGooglerAspectRatio', 'imageGooglerRemixEnabled', 'imageGooglerReferenceImage', 'imageGooglerRemixStrictMode', 'imageGooglerVideoEnabled', 'imageGooglerVideoModel', 'imageGooglerVideoMode', 'imageGooglerVideoCount', 'imageGooglerVideoUpscale'],
            elevenlabsimage: ['elevenLabsImageAspectRatio']
        };

        Object.entries(imgServices).forEach(([svc, fields]: [string, any]) => {
            templateData.image.services[svc] = {};
            fields.forEach((f: string) => {
                if (settings[f] !== undefined) templateData.image.services[svc][f] = settings[f];
            });
        });

        // 4. Subtitle Settings
        const subtitleFields = [
            'subtitleService', 'subtitleModel', 'subtitleAmdLanguage', 'subtitleMaxLen', 'subtitleMaxWords',
            'subtitleFont', 'subtitleSize', 'subtitleColor', 'subtitleFadeEnabled', 'subtitleFadeIn', 'subtitleFadeOut',
            'subtitleKaraokeEffect', 'subtitleWhisperxLanguage'
        ];
        subtitleFields.forEach(f => { if (settings[f] !== undefined) templateData.subtitle[f] = settings[f]; });

        // 6. Custom Stages
        templateData.customStages = settings.customStages || [];
        templateData.customStagesEnabled = settings.customStagesEnabled || false;
        templateData.customStagesCollapsed = settings.customStagesCollapsed || false;

        // 7. Montage Triggers
        templateData.montageOverlayTriggers = settings.montageOverlayTriggers || [];
        templateData.montageOverlayTriggersEnabled = settings.montageOverlayTriggersEnabled || false;

        // 5. Montage Settings
        const montageFields = [
            'montageResolution', 'montageFPS', 'montageSwayFactor', 'montageZoomFactor',
            'montageUpscaleFactor', 'montageTransitionDuration', 'montageTransitionEffect',
            'montageEncodingPreset', 'montageBitrate', 'montageIntroVideoEnabled', 'montageIntroVideoPath',
            'montageWatermarkEnabled', 'montageWatermarkPath', 'montageWatermarkPosition', 'montageWatermarkOpacity', 'montageWatermarkSize', 'montageWatermarkOnIntro',
            'montageOverlayEnabled', 'montageOverlayPath', 'montageOverlayOnIntro'
        ];
        montageFields.forEach(f => { if (settings[f] !== undefined) templateData.montage[f] = settings[f]; });

        // Cleanup empty objects
        if (Object.keys(templateData.api).length === 0) delete templateData.api;
        if (Object.keys(templateData.text).length === 0) delete templateData.text;
        if (Object.keys(templateData.subtitle).length === 0) delete templateData.subtitle;
        if (Object.keys(templateData.montage).length === 0) delete templateData.montage;

        await saveTemplate(type, name, templateData);
    };


    const handleConfirmDelete = async () => {
        if (templateToDelete) {
            await removeTemplate(templateToDelete.id);
            setSelectedTemplateIds(prev => prev.filter(id => id !== templateToDelete.id));
            setTemplateToDelete(null);
        }
    };

    /**
     * Recursively flattens a nested object into a flat key-value map.
     */
    const flattenSettings = (obj: any): any => {
        let result: any = {};
        if (!obj || typeof obj !== 'object') return result;

        // 1. Recursive flattening
        const flatten = (o: any) => {
            if (!o || typeof o !== 'object') return;
            for (const i in o) {
                const val = o[i];
                if (val !== null && typeof val === 'object' && !Array.isArray(val)) {
                    flatten(val);
                } else {
                    result[i] = val;
                }
            }
        };
        flatten(obj);

        // 2. Specialized mapping for "stages" group to "Enabled" fields used in the UI
        if (obj.stages && typeof obj.stages === 'object') {
            const s = obj.stages;
            if (s.voiceover !== undefined) result.voiceoverEnabled = s.voiceover;
            if (s.image !== undefined) result.imageEnabled = s.image;
            if (s.subtitle !== undefined) result.subtitleEnabled = s.subtitle;
            if (s.montage !== undefined) result.montageEnabled = s.montage;
            if (s.translate !== undefined) result.translateEnabled = s.translate;
            if (s.rewrite !== undefined) result.rewriteEnabled = s.rewrite;
        }

        // 3. Control section mapping
        if (obj.control && typeof obj.control === 'object') {
            if (obj.control.translate !== undefined) result.translateControlEnabled = obj.control.translate;
            if (obj.control.image !== undefined) result.imageControlEnabled = obj.control.image;
        }

        // Explicitly handle arrays (they should be arrays, not flattened)
        if (obj.customStages && Array.isArray(obj.customStages)) result.customStages = obj.customStages;
        if (obj.montageOverlayTriggers && Array.isArray(obj.montageOverlayTriggers)) {
            result.montageOverlayTriggers = obj.montageOverlayTriggers;
        } else if (obj.montage && typeof obj.montage === 'object' && Array.isArray(obj.montage.montageOverlayTriggers)) {
            // Compatibility for templates that saved them inside montage object
            result.montageOverlayTriggers = obj.montage.montageOverlayTriggers;
        }

        return result;
    };

    const handleAddTask = useCallback(async (taskName: string, overrideContent?: string, overrideTemplateIds?: string[], isDirectFromHistory: boolean = false) => {
        if (!settings) {
            showToast(t('common.error') || 'Settings not ready', 'error');
            return;
        }
        setIsModalOpen(false);
        const resolvedName = taskName.trim() || getNextTaskName();

        // Use override params if provided (direct call), otherwise use state (modal call from history), otherwise use defaults
        const finalContent = overrideContent !== undefined ? overrideContent : (historyOverride ? historyOverride.content : content);
        const finalTemplateIds = overrideTemplateIds !== undefined ? overrideTemplateIds : (historyOverride ? historyOverride.templateIds : selectedTemplateIds);
        const isFromHistory = isDirectFromHistory || !!historyOverride;

        // Clear history override for next time
        setHistoryOverride(null);

        // 1. Побудуємо список усіх завдань, які плануємо додати
        const relevantTemplateIds = finalTemplateIds.filter(id => templates.find(t => t.id === id)?.type === type);
        const tasksToCheck: any[] = [];

        if (relevantTemplateIds.length === 0) {
            tasksToCheck.push({ taskName: resolvedName, taskType: type, subName: "", settings: settings });
        } else {
            relevantTemplateIds.forEach(id => {
                const template = templates.find(t => t.id === id);
                if (template) {
                    const tplSettings = flattenSettings(template.settings);
                    tasksToCheck.push({ taskName: resolvedName, taskType: type, subName: template.name, settings: tplSettings });
                }
            });
        }

        // 2. Перевірка на вже існуючі файли для кожного завдання
        try {
            const results = await CheckExistingTasks(tasksToCheck);

            // Record to history if NOT applying from history itself
            if (!isFromHistory) {
                const templatesUsed = relevantTemplateIds.length === 0
                    ? [t('common.default') || "Default"]
                    : relevantTemplateIds.map(id => templates.find(tpl => tpl.id === id)?.name).filter(Boolean) as string[];

                try {
                    await AddToHistory(resolvedName, type, templatesUsed, finalContent);
                } catch (historyErr) {
                    console.error("History recording failed:", historyErr);
                }
            }

            if (results && results.length > 0) {
                setExistingFilesData(results); // Тепер це масив
                setPendingTaskName(resolvedName);
                setPendingContent(finalContent);
                return;
            }
        } catch (err) {
            console.error("CheckExistingTasks failed:", err);
        }

        proceedAddTask(resolvedName, [], undefined, overrideTemplateIds, finalContent);

        // Show success toast
        showToast(t('pipeline.task_added_success') || 'Task added to queue', 'success');
    }, [settings, type, content, templates, selectedTemplateIds, historyOverride, getNextTaskName, showToast, t]);

    const proceedAddTask = useCallback((taskName: string, skippedStages: string[], existingDataArray?: any[], overrideTemplateIds?: string[], overrideContent?: string, settingsOverrides?: any) => {
        const finalTemplateIds = overrideTemplateIds !== undefined ? overrideTemplateIds : selectedTemplateIds;
        const finalContent = overrideContent !== undefined ? overrideContent : content;
        const relevantTemplateIds = finalTemplateIds.filter(id => templates.find(t => t.id === id)?.type === type);

        if (relevantTemplateIds.length === 0) {
            const existing = existingDataArray?.find(d => d.id === "");
            const finalSettings = settingsOverrides ? { ...settings, ...settingsOverrides } : settings;
            addTask(type, finalContent, finalSettings, taskName, "", skippedStages, existing);
        } else {
            const tasksData = relevantTemplateIds.map(id => {
                const template = templates.find(t => t.id === id);
                if (!template) return null;
                const flattened = flattenSettings(template.settings);
                const tplSettings = settingsOverrides ? { ...flattened, ...settingsOverrides } : flattened;
                const existing = existingDataArray?.find(d => d.id === template.name);
                return { settings: tplSettings, subName: template.name, existing };
            }).filter(d => d && d.settings);

            addTasks(type, finalContent, tasksData as any, taskName, skippedStages);
            if (overrideTemplateIds === undefined) {
                setSelectedTemplateIds([]);
            }
        }
    }, [type, content, settings, templates, selectedTemplateIds, addTask, addTasks, setSelectedTemplateIds]);

    const applyTemplate = (tpl: any) => {
        if (!tpl || !tpl.settings) return;
        const applied = flattenSettings(tpl.settings);
        const nameField = type === 'translate' ? 'translatePipelineName' : (type === 'rewrite' ? 'rewritePipelineName' : 'voiceoverPipelineName');

        setSettings((prev: any) => {
            if (!prev) return { ...applied, [nameField]: tpl.name };

            // Filter: only keep primitives or arrays. Never objects (to prevent corruption)
            const cleanApplied: any = {};
            Object.keys(applied).forEach(k => {
                const v = applied[k];
                if (v === null || typeof v !== 'object' || Array.isArray(v)) {
                    cleanApplied[k] = v;
                }
            });

            return {
                ...prev,
                ...cleanApplied,
                [nameField]: tpl.name,
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
                // Ensure values are RESET to defaults if missing in template, not carried over from previous state
                customStages: cleanApplied.customStages ?? [],
                customStagesEnabled: cleanApplied.customStagesEnabled ?? false,
                montageOverlayTriggers: cleanApplied.montageOverlayTriggers ?? [],
                montageOverlayTriggersEnabled: cleanApplied.montageOverlayTriggersEnabled ?? false,
            };
        });
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

    useEffect(() => {
        // @ts-ignore
        const unsub = EventsOn("applyHistoryEntry", (entry: any) => {
            if (entry.type === type) {
                if (!settings) {
                    console.warn("[Sidebar] Settings not ready, cannot apply history entry");
                    return;
                }
                // Find template IDs by names
                const matchedIds: string[] = [];
                if (entry.templates && entry.templates.length > 0) {
                    entry.templates.forEach((tplName: string) => {
                        const tpl = templates.find(t => t.name === tplName && t.type === type);
                        if (tpl) matchedIds.push(tpl.id);
                    });
                }

                setSelectedTemplateIds(matchedIds);

                // Skip the modal and call handleAddTask directly
                handleAddTask(entry.taskName, entry.content, matchedIds, true);
            }
        });
        return () => {
            if (unsub) unsub();
        };
    }, [type, templates, settings, handleAddTask, setSelectedTemplateIds]);

    useEffect(() => {
        document.documentElement.style.setProperty('--sidebar-toggle-width', '36px');
        return () => { document.documentElement.style.setProperty('--sidebar-toggle-width', '0px'); };
    }, []);

    useEffect(() => {
        const width = isOpen ? (settings?.sidebarWidth || 320) : 0;
        document.documentElement.style.setProperty('--pipeline-sidebar-width', `${width}px`);
        return () => {
            document.documentElement.style.setProperty('--pipeline-sidebar-width', '0px');
        };
    }, [isOpen, settings?.sidebarWidth]);

    const handleSelectPath = async () => {
        try {
            const path = await SelectDirectory();
            if (path) {
                // Only two main paths allowed: translate and rewrite. 
                // Voiceover (if standalone) will use translate path by default.
                const field = type === 'rewrite' ? 'rewriteOutputPath' : 'translateOutputPath';
                handleChange(field, path);
            }
        } catch (err) { console.error("Failed to select path:", err); }
    };

    const renderValueOrInput = (field: string, value: any, isFloat: boolean) => {
        const safeVal = (val: any) => {
            if (typeof val === 'number') return val;
            const parsed = parseFloat(val);
            return isNaN(parsed) ? 0 : parsed;
        };

        const numValue = safeVal(value);
        if (editingField === field) {
            return (
                <input
                    autoFocus className="settings-value-input" type="number" defaultValue={numValue} step={isFloat ? "0.1" : "500"}
                    onBlur={(e) => {
                        setEditingField(null);
                        let val = parseFloat(e.target.value);
                        if (isNaN(val)) val = numValue;
                        handleChange(field, val);
                    }}
                    onKeyDown={(e) => { if (e.key === 'Enter') (e.target as HTMLInputElement).blur(); }}
                />
            );
        }
        let displayValue: string | number = isFloat ? numValue.toFixed(1) : numValue;
        if (!isFloat && numValue === 0 && field.includes('MaxTokens')) displayValue = t('pipeline.max_tokens_unlimited');
        return (
            <span className="settings-slider-value" onClick={(e) => { e.stopPropagation(); setEditingField(field); }}
                style={!isFloat && numValue === 0 ? { minWidth: '80px', fontSize: '10px' } : {}}>
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
                        <>
                            <TextSection type={type} settings={settings} handleChange={handleChange} models={models} renderValueOrInput={renderValueOrInput} setCurrentPath={setCurrentPath} />
                            <CustomStagesSection settings={settings} handleChange={handleChange} models={models} />
                        </>
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
                        elevenLabsImageKeys={elevenLabsImageKeys}
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

            <TaskNameModal
                isOpen={isModalOpen}
                onClose={() => {
                    setIsModalOpen(false);
                    setHistoryOverride(null);
                }}
                onConfirm={handleAddTask}
                defaultName={historyOverride?.taskName}
            />
            {existingFilesData && (
                <ExistingFilesModal
                    isOpen={true}
                    data={existingFilesData}
                    onConfirm={(skip: string[]) => {
                        // If some found stages are NOT in the final 'skip' list,
                        // it means the user wants to regenerate them.
                        const allFound = new Set<string>();
                        existingFilesData.forEach((d: any) => (d.foundStages || []).forEach((s: string) => allFound.add(s)));

                        const regenOverrides: Record<string, any> = {};
                        if (allFound.has('voice') && !skip.includes('voice')) regenOverrides.voiceoverRegenerate = true;
                        if (allFound.has('image') && !skip.includes('image')) {
                            regenOverrides.imageRegeneratePrompts = true;
                            regenOverrides.imageGooglerRegenerateImages = true;
                            regenOverrides.imageElevenLabsImageRegenerate = true;
                        }
                        if (allFound.has('subtitle') && !skip.includes('subtitle')) regenOverrides.subtitleRegenerate = true;

                        proceedAddTask(pendingTaskName, skip, existingFilesData, undefined, pendingContent, regenOverrides);
                        setExistingFilesData(null);
                    }}
                    onCancel={() => {
                        const regenOverrides = {
                            imageRegeneratePrompts: true,
                            imageGooglerRegenerateImages: true,
                            voiceoverRegenerate: true,
                            subtitleRegenerate: true,
                            imageElevenLabsImageRegenerate: true
                        };
                        proceedAddTask(pendingTaskName, [], existingFilesData, undefined, pendingContent, regenOverrides);
                        setExistingFilesData(null);
                    }}
                />
            )}
            <button className={`sidebar-floating-toggle ${isOpen ? 'is-open' : ''}`} onClick={onToggle} title={isOpen ? t('pipeline.hide_settings') : t('pipeline.show_settings')}>
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" /></svg>
            </button>
            <ConfirmModal isOpen={!!templateToDelete} onClose={() => setTemplateToDelete(null)} onConfirm={handleConfirmDelete} title={t('common.delete')} message={t('templatesTab.delete_confirm')} />
        </aside>
    );
};
