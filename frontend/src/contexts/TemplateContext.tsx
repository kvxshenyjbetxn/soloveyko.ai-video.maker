import React, { createContext, useContext, useState, useEffect, useCallback, ReactNode } from 'react';
// @ts-ignore
import { GetTemplates, AddTemplate, DeleteTemplate, UpdateTemplate } from '../../wailsjs/go/main/App';
import { useToast } from './ToastContext';

export interface PipelineSettings {
    translateModel: string;
    translatePrompt: string;
    translateTemperature: number;
    translateMaxTokens: number;
    translateCollapsed: boolean;
    translateOpenRouterKeyID: string;
    translateElevenLabsBotKeyID: string;
    rewriteModel: string;
    rewritePrompt: string;
    rewriteTemperature: number;
    rewriteMaxTokens: number;
    rewriteCollapsed: boolean;
    rewriteOpenRouterKeyID: string;
    rewriteElevenLabsBotKeyID: string;
    sidebarWidth: number;
    translateEnabled: boolean;
    rewriteEnabled: boolean;
    apiCollapsed: boolean;
    translateOutputPath: string;
    rewriteOutputPath: string;
    outputPath: string;
    pathCollapsed: boolean;
    translatePipelineName: string;
    rewritePipelineName: string;
    voiceoverModel: string;
    voiceoverPrompt: string;
    voiceoverTemperature: number;
    voiceoverMaxTokens: number;
    voiceoverCollapsed: boolean;
    voiceoverOpenRouterKeyID: string;
    voiceoverElevenLabsBotKeyID: string;
    voiceoverEnabled: boolean;
    voiceoverOutputPath: string;
    voiceoverPipelineName: string;
    templatesCollapsed: boolean;
    translateTemplatesCollapsed: boolean;
    rewriteTemplatesCollapsed: boolean;
    voiceoverTemplatesCollapsed: boolean;
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
