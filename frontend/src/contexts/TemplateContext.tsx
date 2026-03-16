import React, { createContext, useContext, useState, useEffect, useCallback, ReactNode } from 'react';
// @ts-ignore
import { GetTemplates, AddTemplate, DeleteTemplate, UpdateTemplate } from '../../wailsjs/go/main/App';
import { useToast } from './ToastContext';

export interface CustomStage {
    id: string;
    name: string;
    prompt: string;
    dataSource: string;
    enabled: boolean;
}

export interface PipelineSettings {
    translateModel: string;
    translatePrompt: string;
    translateTemperature: number;
    translateMaxTokens: number;
    translateCollapsed: boolean;
    translateOpenRouterKeyID: string;
    translateEnabled: boolean;
    translateOutputPath: string;
    translatePipelineName: string;
    translateTemplatesCollapsed: boolean;
    translateControlEnabled: boolean;

    rewriteModel: string;
    rewritePrompt: string;
    rewriteTemperature: number;
    rewriteMaxTokens: number;
    rewriteCollapsed: boolean;
    rewriteOpenRouterKeyID: string;
    rewriteEnabled: boolean;
    rewriteOutputPath: string;
    rewritePipelineName: string;
    rewriteTemplatesCollapsed: boolean;

    voiceoverEnabled: boolean;
    voiceoverService: string;
    voiceoverTemplate: string;
    voiceoverElevenLabsBotKeyID: string;
    voiceoverElevenLabsUnlimKeyID: string;
    voiceoverElevenLabsUAKeyID: string;
    voiceoverVoiceMakerKeyID: string;
    voiceoverOutputPath: string;
    voiceoverPipelineName: string;
    voiceoverTemplatesCollapsed: boolean;
    voiceoverCollapsed: boolean;

    voiceMakerVoiceID: string;
    voiceMakerLanguageCode: string;
    voiceMakerCharLimit: number;

    elevenLabsUnlimVoiceID: string;
    elevenLabsUnlimStability: number;
    elevenLabsUnlimSimilarity: number;
    elevenLabsUnlimStyle: number;
    elevenLabsUnlimSpeakerBoost: boolean;

    elevenLabsUAVoiceID: string;
    elevenLabsUAStability: number;
    elevenLabsUASimilarity: number;
    elevenLabsUAStyle: number;
    elevenLabsUASpeakerBoost: boolean;
    elevenLabsUAModel: string;

    imageGooglerVideoEnabled: boolean;
    imageGooglerVideoModel: string;
    imageGooglerVideoMode: string;
    imageGooglerVideoCount: number;
    imageGooglerVideoUpscale: boolean;

    imageService: string;
    imageModel: string;
    imageWidth: number;
    imageHeight: number;
    imageNoLogo: boolean;
    imageEnhance: boolean;
    imagePrompt: string;
    imagePollinationsKeyID: string;
    imageGooglerModel: string;
    imageGooglerAspectRatio: string;
    imageGooglerRemixEnabled: boolean;
    imageGooglerReferenceImage: string;
    imageGooglerRemixStrictMode: boolean;
    imageOutputPath: string;
    imagePipelineName: string;
    imageTemplatesCollapsed: boolean;
    imageCollapsed: boolean;
    imageGenerationMethod: string;
    imageGroupSentences: boolean;
    imageSentenceLimit: number;
    imagePromptModel: string;
    imagePromptTemperature: number;
    imagePromptMaxTokens: number;
    imageMode: string;
    imageMemoryType: string;
    imageMemoryChars: number;
    imageDetermineCharacters: boolean;
    imageDetermineCharactersMode: string;
    imageDetermineCharactersPrompt: string;
    imageDetermineCharactersStatic: string;
    elevenLabsImageKeyID: string;
    elevenLabsImageAspectRatio: string;

    subtitleEnabled: boolean;
    subtitleCollapsed: boolean;
    subtitleService: string;
    subtitleModel: string;
    subtitleAmdLanguage: string;
    subtitleMaxLen: number;
    subtitleColor: string;
    subtitleOutlineColor: string;
    subtitleOutlineWidth: number;
    subtitleShadowColor: string;
    subtitleShadowWidth: number;
    subtitleBlur: number;
    subtitleSize: number;
    subtitleFont: string;
    subtitleUppercase: boolean;
    subtitleKerning: number;
    subtitlePosition: 'bottom' | 'middle' | 'top';
    subtitleMarginV: number;
    subtitleBgEnabled: boolean;
    subtitleBgColor: string;
    subtitleAnimation: 'none' | 'slide-up';
    subtitleFadeEnabled: boolean;
    subtitleFadeIn: number;
    subtitleFadeOut: number;
    subtitleKaraokeEffect: boolean;
    subtitleKaraokeColor: string;
    subtitleKaraokeMode: 'fill' | 'highlight' | 'appear';
    subtitleKaraokeScale: number;
    subtitleWhisperxLanguage: string;

    montageEnabled: boolean;
    montageCollapsed: boolean;
    montageResolution: string;
    montageFPS: number;
    montageSwayFactor: number;
    montageZoomFactor: number;
    montageUpscaleFactor: number;
    montageTransitionDuration: number;
    montageTransitionEffect: string;
    montageEncodingPreset: string;
    montageBitrate: number;

    sidebarWidth: number;
    apiCollapsed: boolean;
    pathCollapsed: boolean;
    templatesCollapsed: boolean;
    controlCollapsed: boolean;
    outputPath: string;
    imageControlEnabled: boolean;
    montageControlEnabled: boolean;
    edgeTTSVoiceID: string;
    edgeTTSRate: string;
    edgeTTSPitch: string;
    edgeTTSVolume: string;
    customStages: CustomStage[];
    customStagesEnabled: boolean;
    customStagesCollapsed: boolean;
    imageShortVideoFillMode: string;
    montageMetadataSimulation: string;
}

export interface PipelineTemplate {
    id: string;
    type: 'translate' | 'rewrite' | 'voiceover';
    name: string;
    createdAt: number;
    settings: any;
}

interface TemplateContextType {
    templates: PipelineTemplate[];
    loadTemplates: () => Promise<void>;
    saveTemplate: (tplType: 'translate' | 'rewrite' | 'voiceover', name: string, data: any) => Promise<void>;
    removeTemplate: (id: string) => Promise<void>;
    updateTemplate: (id: string, name: string, data: any) => Promise<void>;
    isLoading: boolean;
    selectedTemplateIds: string[];
    setSelectedTemplateIds: React.Dispatch<React.SetStateAction<string[]>>;
}

const TemplateContext = createContext<TemplateContextType | undefined>(undefined);

export const TemplateProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
    const [templates, setTemplates] = useState<PipelineTemplate[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [selectedTemplateIds, setSelectedTemplateIds] = useState<string[]>([]);
    const { showToast } = useToast();

    const loadTemplates = useCallback(async () => {
        setIsLoading(true);
        try {
            const temps = await GetTemplates();
            setTemplates((temps as PipelineTemplate[]) || []);
        } catch (error) {
            console.error('Failed to load templates:', error);
            showToast('Помилка завантаження шаблонів', 'error');
        } finally {
            setIsLoading(false);
        }
    }, [showToast]);

    const saveTemplate = useCallback(async (tplType: 'translate' | 'rewrite' | 'voiceover', name: string, data: PipelineSettings) => {
        try {
            await AddTemplate(tplType, name, data);
            await loadTemplates();
            showToast(`Шаблон "${name}" збережено`, 'success');
        } catch (error) {
            console.error('Failed to save template:', error);
            showToast('Помилка збереження шаблону', 'error');
        }
    }, [loadTemplates, showToast]);

    const removeTemplate = useCallback(async (id: string) => {
        try {
            await DeleteTemplate(id);
            await loadTemplates();
            showToast('Шаблон видалено', 'success');
        } catch (error) {
            console.error('Failed to delete template:', error);
            showToast('Помилка видалення шаблону', 'error');
        }
    }, [loadTemplates, showToast]);

    const updateTemplate = useCallback(async (id: string, name: string, data: PipelineSettings) => {
        try {
            await UpdateTemplate(id, name, data);
            await loadTemplates();
            showToast('Шаблон оновлено', 'success');
        } catch (error) {
            console.error('Failed to update template:', error);
            showToast('Помилка оновлення шаблону', 'error');
        }
    }, [loadTemplates, showToast]);

    useEffect(() => {
        loadTemplates();
    }, [loadTemplates]);

    return (
        <TemplateContext.Provider value={{
            templates,
            loadTemplates,
            saveTemplate,
            removeTemplate,
            updateTemplate,
            isLoading,
            selectedTemplateIds,
            setSelectedTemplateIds
        }}>
            {children}
        </TemplateContext.Provider>
    );
};

export const useTemplates = () => {
    const context = useContext(TemplateContext);
    if (!context) {
        throw new Error('useTemplates must be used within a TemplateProvider');
    }
    return context;
};
