import React from 'react';
import './PipelineDashboard.css';
import { useI18n } from '../contexts/I18nContext';

// Import section components
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

interface PipelineDashboardProps {
    type: 'translate' | 'rewrite' | 'voiceover';
    isOpen: boolean;
    onClose: () => void;
    settings: any;
    handleChange: (field: string, value: any) => void;
    setSettings: React.Dispatch<React.SetStateAction<any>>;
    content: string;
    templates: any[];
    selectedTemplateIds: string[];
    setSelectedTemplateIds: React.Dispatch<React.SetStateAction<string[]>>;
    applyTemplate: (tpl: any) => void;
    handleSaveTemplate: () => void;
    setTemplateToDelete: (tpl: any) => void;
    handleAddTask: (taskName: string) => void;
    setIsModalOpen: (open: boolean) => void;
    setCurrentPath?: (path: string) => void;
    // Props for specific sections
    openRouterKeys: any[];
    elevenLabsBotKeys: any[];
    elevenLabsUnlimKeys: any[];
    elevenLabsUAKeys: any[];
    voiceMakerKeys: any[];
    pollinationsKeys: any[];
    elevenLabsImageKeys: any[];
    models: string[];
    voiceTemplates: any[];
    voiceMakerVoices: any[];
    edgeTTSVoices: any[];
    pollinationsModels: string[];
    loadingTemplates: boolean;
    loadingPollinationsModels: boolean;
    estimatedChunks: number;
    fetchVoiceTemplates: (key?: string) => void;
    fetchVoiceMakerVoices: (key?: string) => void;
    fetchEdgeTTSVoices: () => void;
    fetchPollinationsModels: () => void;
    handleSelectPath: () => void;
    renderValueOrInput: (field: string, value: any, isFloat: boolean) => React.ReactNode;
}

// Class Error Boundary to catch render errors in children
class DashboardErrorBoundary extends React.Component<{id: string, children: React.ReactNode}, {hasError: boolean, error: any}> {
    constructor(props: any) {
        super(props);
        this.state = { hasError: false, error: null };
    }
    static getDerivedStateFromError(error: any) {
        return { hasError: true, error };
    }
    componentDidCatch(error: any, errorInfo: any) {
        console.error(`[Dashboard] Component "${this.props.id}" crashed:`, error, errorInfo);
        // Attempt to log to Wails if possible
        if ((window as any).go?.main?.App?.LogFromUI) {
            (window as any).go.main.App.LogFromUI("ERROR", `Dashboard section ${this.props.id} crashed: ${error?.message || String(error)}`);
        }
    }
    render() {
        if (this.state.hasError) {
            return (
                <div className="dashboard-section-error" style={{
                    padding: '20px',
                    background: 'rgba(255, 0, 0, 0.05)',
                    border: '1px solid rgba(255, 0, 0, 0.2)',
                    borderRadius: '12px',
                    margin: '10px'
                }}>
                    <h3 style={{ color: '#ff4d4d', fontSize: '14px', marginBottom: '8px' }}>Error in {this.props.id}</h3>
                    <p style={{ fontSize: '12px', opacity: 0.8, marginBottom: '12px' }}>{this.state.error?.message || "Unknown error"}</p>
                    <button 
                        onClick={() => window.location.reload()}
                        style={{
                            background: 'rgba(255, 255, 255, 0.1)',
                            border: '1px solid rgba(255, 255, 255, 0.2)',
                            color: 'white',
                            padding: '4px 12px',
                            borderRadius: '4px',
                            cursor: 'pointer'
                        }}
                    >
                        Reload App
                    </button>
                    {this.props.id === 'global' && (
                        <button 
                            onClick={() => (window as any).forceCloseDashboard && (window as any).forceCloseDashboard()}
                            style={{
                                marginLeft: '8px',
                                background: 'rgba(255, 255, 255, 0.1)',
                                border: '1px solid rgba(255, 255, 255, 0.2)',
                                color: 'white',
                                padding: '4px 12px',
                                borderRadius: '4px',
                                cursor: 'pointer'
                            }}
                        >
                            Force Close
                        </button>
                    )}
                </div>
            );
        }
        return this.props.children;
    }
}

// Export the main component
export const PipelineDashboard: React.FC<PipelineDashboardProps> = (props) => {
    const { t } = useI18n();
    const { type, isOpen, onClose, settings, handleChange, setSettings, content, templates, selectedTemplateIds, setSelectedTemplateIds, applyTemplate, setTemplateToDelete, setIsModalOpen, setCurrentPath } = props;

    // Diagnostic logging
    React.useEffect(() => {
        if (isOpen) {
            console.log("[Dashboard] Mounted/Opened", { type, settingsPresent: !!settings });
        }
    }, [isOpen]);

    // Persisted order and collapse state for dashboard
    const [localCollapsed, setLocalCollapsed] = React.useState<Record<string, boolean>>(() => {
        try {
            const saved = localStorage.getItem('soloveyko_dashboard_collapsed');
            return saved ? JSON.parse(saved) : {};
        } catch (e) {
            console.error("Failed to parse dashboard collapsed state", e);
            return {};
        }
    });

    const [isLayoutMode, setIsLayoutMode] = React.useState(false);

    const [columns, setColumns] = React.useState<string[][]>(() => {
        const defaultText = [
            ['templates', 'path'],
            ['control', 'api'],
            ['text', 'customStages'],
            ['voiceover', 'image', 'subtitle', 'montage']
        ];
        const defaultVoice = [
            ['templates', 'path'],
            ['control', 'api'],
            ['voiceover', 'image'],
            ['subtitle', 'montage']
        ];
        const defaultOrder = type === 'voiceover' ? defaultVoice : defaultText;
        
        try {
            const saved = localStorage.getItem(`soloveyko_dashboard_columns_${type}`);
            if (saved) {
                const parsed = JSON.parse(saved);
                if (Array.isArray(parsed) && parsed.length === 4) {
                    // Validation: check if all sections are present
                    const flat = parsed.flat();
                    const mandatory = type === 'voiceover' 
                        ? ['templates', 'control', 'api', 'path', 'voiceover', 'image', 'subtitle', 'montage']
                        : ['templates', 'control', 'api', 'path', 'text', 'customStages', 'voiceover', 'image', 'subtitle', 'montage'];
                    
                    const missing = mandatory.filter(m => !flat.includes(m));
                    if (missing.length === 0) return parsed;
                    
                    // If something is missing, merge with default
                    const result = [...parsed];
                    missing.forEach(m => result[0].push(m));
                    return result;
                }
            }
        } catch (e) {
            console.error("Failed to parse dashboard columns", e);
        }
        return defaultOrder;
    });

    React.useEffect(() => {
        localStorage.setItem('soloveyko_dashboard_collapsed', JSON.stringify(localCollapsed));
    }, [localCollapsed]);

    React.useEffect(() => {
        localStorage.setItem(`soloveyko_dashboard_columns_${type}`, JSON.stringify(columns));
    }, [columns, type]);

    const moveSection = (id: string, direction: 'up' | 'down' | 'left' | 'right') => {
        const newColumns = columns.map(col => [...col]);
        let colIdx = -1;
        let itemIdx = -1;

        for (let i = 0; i < 4; i++) {
            const idx = newColumns[i].indexOf(id);
            if (idx !== -1) {
                colIdx = i;
                itemIdx = idx;
                break;
            }
        }

        if (colIdx === -1) return;

        if (direction === 'up' && itemIdx > 0) {
            [newColumns[colIdx][itemIdx], newColumns[colIdx][itemIdx - 1]] = [newColumns[colIdx][itemIdx - 1], newColumns[colIdx][itemIdx]];
        } else if (direction === 'down' && itemIdx < newColumns[colIdx].length - 1) {
            [newColumns[colIdx][itemIdx], newColumns[colIdx][itemIdx + 1]] = [newColumns[colIdx][itemIdx + 1], newColumns[colIdx][itemIdx]];
        } else if (direction === 'left' && colIdx > 0) {
            newColumns[colIdx].splice(itemIdx, 1);
            newColumns[colIdx - 1].push(id);
        } else if (direction === 'right' && colIdx < 3) {
            newColumns[colIdx].splice(itemIdx, 1);
            newColumns[colIdx + 1].push(id);
        }

        setColumns(newColumns);
    };

    const isTranslate = type === 'translate';
    const isRewrite = type === 'rewrite';
    const pipelineName = (settings && (isTranslate ? settings.translatePipelineName : (isRewrite ? settings.rewritePipelineName : settings.voiceoverPipelineName))) || '';

    // Register a global escape hatch for the error boundary to use
    React.useEffect(() => {
        (window as any).forceCloseDashboard = onClose;
        return () => { delete (window as any).forceCloseDashboard; };
    }, [onClose]);

    const onNameChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        let field = '';
        if (isTranslate) field = 'translatePipelineName';
        else if (isRewrite) field = 'rewritePipelineName';
        else field = 'voiceoverPipelineName';
        handleChange(field, e.target.value);
    };

    const toggleLocalCollapse = (id: string, collapsed: boolean) => {
        setLocalCollapsed(prev => ({ ...prev, [id]: collapsed }));
    };

    // Drag and Drop Logic for Placeholders
    const [draggedId, setDraggedId] = React.useState<string | null>(null);

    const handleDragStart = (e: React.DragEvent, id: string) => {
        setDraggedId(id);
        if (e.dataTransfer) {
            e.dataTransfer.effectAllowed = 'move';
            // Set a transparent drag image or use the default
            e.dataTransfer.setData('text/plain', id);
        }
    };

    const handleDragOver = (e: React.DragEvent, targetColIdx: number, targetItemIdx: number) => {
        e.preventDefault();
        if (!draggedId) return;

        // Find current position
        let sourceColIdx = -1;
        let sourceItemIdx = -1;
        for (let i = 0; i < 4; i++) {
            const idx = columns[i].indexOf(draggedId);
            if (idx !== -1) {
                sourceColIdx = i;
                sourceItemIdx = idx;
                break;
            }
        }

        if (sourceColIdx === -1) return;
        if (sourceColIdx === targetColIdx && sourceItemIdx === targetItemIdx) return;

        const newColumns = columns.map(col => [...col]);
        newColumns[sourceColIdx].splice(sourceItemIdx, 1);
        newColumns[targetColIdx].splice(targetItemIdx, 0, draggedId);
        setColumns(newColumns);
    };

    const handleDrop = (e: React.DragEvent) => {
        e.preventDefault();
        setDraggedId(null);
    };

    const renderSection = (id: string) => {
        const isCollapsed = localCollapsed[id] ?? false;
        
        switch (id) {
            case 'templates':
                return (
                    <TemplatesSection
                        type={type} 
                        templates={templates || []} 
                        selectedTemplateIds={selectedTemplateIds || []}
                        toggleTemplate={(id) => setSelectedTemplateIds(prev => prev.includes(id) ? prev.filter(t => t !== id) : [...prev, id])}
                        applyTemplate={applyTemplate} setTemplateToDelete={setTemplateToDelete}
                        isCollapsed={isCollapsed}
                        onToggleCollapse={(collapsed) => toggleLocalCollapse('templates', collapsed)}
                        setCurrentPath={setCurrentPath}
                    />
                );
            case 'control':
                return (
                    <ControlSection 
                        settings={settings} 
                        handleChange={handleChange} 
                        isCollapsed={isCollapsed}
                        onToggleCollapse={(collapsed) => toggleLocalCollapse('control', collapsed)}
                    />
                );
            case 'api':
                return (
                    <ApiSection
                        type={type} settings={settings} handleChange={handleChange}
                        openRouterKeys={props.openRouterKeys || []} 
                        elevenLabsBotKeys={props.elevenLabsBotKeys || []} 
                        elevenLabsUnlimKeys={props.elevenLabsUnlimKeys || []}
                        elevenLabsUAKeys={props.elevenLabsUAKeys || []} 
                        voiceMakerKeys={props.voiceMakerKeys || []} 
                        pollinationsKeys={props.pollinationsKeys || []}
                        elevenLabsImageKeys={props.elevenLabsImageKeys || []}
                        fetchVoiceTemplates={props.fetchVoiceTemplates} 
                        fetchVoiceMakerVoices={props.fetchVoiceMakerVoices} 
                        setCurrentPath={setCurrentPath}
                        isCollapsed={isCollapsed}
                        onToggleCollapse={(collapsed) => toggleLocalCollapse('api', collapsed)}
                    />
                );
            case 'path':
                return (
                    <PathSection 
                        type={type} 
                        settings={settings} 
                        handleChange={handleChange} 
                        handleSelectPath={props.handleSelectPath} 
                        isCollapsed={isCollapsed}
                        onToggleCollapse={(collapsed) => toggleLocalCollapse('path', collapsed)}
                    />
                );
            case 'text':
                if (type !== 'translate' && type !== 'rewrite') return null;
                return (
                    <TextSection 
                        type={type} 
                        settings={settings} 
                        handleChange={handleChange} 
                        models={props.models || []} 
                        renderValueOrInput={props.renderValueOrInput} 
                        setCurrentPath={setCurrentPath} 
                        isCollapsed={isCollapsed}
                        onToggleCollapse={(collapsed) => toggleLocalCollapse('text', collapsed)}
                    />
                );
            case 'customStages':
                if (type !== 'translate' && type !== 'rewrite') return null;
                return (
                    <CustomStagesSection 
                        settings={settings} 
                        handleChange={handleChange} 
                        models={props.models || []} 
                        isCollapsed={isCollapsed}
                        onToggleCollapse={(collapsed) => toggleLocalCollapse('customStages', collapsed)}
                    />
                );
            case 'voiceover':
                return (
                    <VoiceoverSection
                        type={type}
                        settings={settings} handleChange={handleChange} setSettings={setSettings}
                        fetchVoiceTemplates={props.fetchVoiceTemplates} fetchVoiceMakerVoices={props.fetchVoiceMakerVoices} fetchEdgeTTSVoices={props.fetchEdgeTTSVoices}
                        voiceTemplates={props.voiceTemplates || []} 
                        voiceMakerVoices={props.voiceMakerVoices || []} 
                        edgeTTSVoices={props.edgeTTSVoices || []} 
                        loadingTemplates={props.loadingTemplates}
                        isCollapsed={isCollapsed}
                        onToggleCollapse={(collapsed) => toggleLocalCollapse('voiceover', collapsed)}
                    />
                );
            case 'image':
                return (
                    <ImageSection
                        settings={settings} handleChange={handleChange} setSettings={setSettings}
                        fetchPollinationsModels={props.fetchPollinationsModels} 
                        pollinationsModels={props.pollinationsModels || []}
                        loadingPollinationsModels={props.loadingPollinationsModels} 
                        estimatedChunks={props.estimatedChunks}
                        content={content} 
                        models={props.models || []}
                        renderValueOrInput={props.renderValueOrInput}
                        setCurrentPath={setCurrentPath}
                        elevenLabsImageKeys={props.elevenLabsImageKeys || []}
                        isCollapsed={isCollapsed}
                        onToggleCollapse={(collapsed) => toggleLocalCollapse('image', collapsed)}
                    />
                );
            case 'subtitle':
                return (
                    <SubtitleSection
                        settings={settings} 
                        handleChange={handleChange} 
                        setSettings={setSettings}
                        setCurrentPath={setCurrentPath}
                        isCollapsed={isCollapsed}
                        onToggleCollapse={(collapsed) => toggleLocalCollapse('subtitle', collapsed)}
                    />
                );
            case 'montage':
                return (
                    <MontageSection
                        settings={settings} 
                        handleChange={handleChange} 
                        setSettings={setSettings}
                        setCurrentPath={setCurrentPath}
                        isCollapsed={isCollapsed}
                        onToggleCollapse={(collapsed) => toggleLocalCollapse('montage', collapsed)}
                    />
                );
            default:
                return null;
        }
    };

    const scrollContainerRef = React.useRef<HTMLDivElement>(null);

    if (!isOpen) return null;
    
    // Safety check for settings - if null, we show a guarded loading overlay
    if (!settings) {
        return (
            <div className="pipeline-dashboard-overlay">
                <header className="dashboard-header">
                    <div className="dashboard-title-group">
                        <h1 className="dashboard-title">{t('pipeline.dashboard_title') || "Pipeline Dashboard"}</h1>
                        <span className="dashboard-subtitle">{type.toUpperCase()} PREPARATION</span>
                    </div>
                    <div className="dashboard-actions">
                        <button className="dashboard-close-btn" onClick={onClose}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="18" x2="18" y2="6"></line></svg>
                        </button>
                    </div>
                </header>
                <div className="dashboard-loading" style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'white' }}>
                    {t('common.loading') || "Loading settings..."}
                </div>
            </div>
        );
    }

    return (
        <DashboardErrorBoundary id="global">
            <div className="pipeline-dashboard-overlay" onClick={(e) => {
            if (e.target === e.currentTarget) onClose();
        }}>
            <header className="dashboard-header">
                <div className="dashboard-title-group">
                    <h1 className="dashboard-title">{t('pipeline.dashboard_title') || 'Pipeline Dashboard'}</h1>
                    <p className="dashboard-subtitle">{type === 'translate' ? t('text.translate') : (type === 'rewrite' ? t('text.rewrite') : t('tabs.voiceover'))}</p>
                </div>

                <div className="dashboard-template-controls">
                    <div className="template-name-input-group">
                        <label className="template-label">{t('pipeline.name')}</label>
                        <input
                            className="template-name-input"
                            value={pipelineName}
                            onChange={onNameChange}
                            placeholder={t('pipeline.name_placeholder') || "Назва пайплайну..."}
                        />
                    </div>
                    <button
                        className="dashboard-save-template-btn"
                        onClick={props.handleSaveTemplate}
                        title={t('pipeline.save_template')}
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"></path>
                            <polyline points="17 21 17 13 7 13 7 21"></polyline>
                            <polyline points="7 3 7 8 15 8"></polyline>
                        </svg>
                        <span>{t('common.save')}</span>
                    </button>
                </div>

                <div className="dashboard-actions">
                    <button 
                        className={`dashboard-layout-btn ${isLayoutMode ? 'is-active' : ''}`}
                        onClick={() => setIsLayoutMode(!isLayoutMode)}
                        title="Налаштувати дашборд"
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            <circle cx="12" cy="12" r="3"></circle>
                            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
                        </svg>
                    </button>
                    <button className="dashboard-close-btn" onClick={onClose} title={t('common.close')}>
                        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            <line x1="18" y1="6" x2="6" y2="18"></line>
                            <line x1="6" y1="6" x2="18" y2="18"></line>
                        </svg>
                    </button>
                </div>
            </header>

            <main 
                ref={scrollContainerRef}
                className="dashboard-content"
            >
                <div className="dashboard-masonry">
                    {columns.map((col, colIdx) => (
                        <div key={colIdx} className="dashboard-column">
                            {col.map((id, itemIdx) => (
                                <DashboardErrorBoundary id={id} key={id}>
                                    <div 
                                        className={`dashboard-item ${isLayoutMode ? 'is-layout-editing' : ''} ${draggedId === id ? 'is-dragging' : ''}`}
                                        draggable={isLayoutMode}
                                        onDragStart={isLayoutMode ? (e) => handleDragStart(e, id) : undefined}
                                        onDragOver={isLayoutMode ? (e) => handleDragOver(e, colIdx, itemIdx) : undefined}
                                        onDrop={isLayoutMode ? handleDrop : undefined}
                                    >
                                        {isLayoutMode ? (
                                            <div className="dashboard-item-placeholder">
                                                <div className="placeholder-title">
                                                    {id === 'templates' && t('pipeline.group.templates')}
                                                    {id === 'control' && t('pipeline.group.control')}
                                                    {id === 'api' && t('pipeline.group.api')}
                                                    {id === 'path' && t('pipeline.group.path')}
                                                    {id === 'text' && t('pipeline.group.text')}
                                                    {id === 'customStages' && t('pipeline.group.custom_stages')}
                                                    {id === 'voiceover' && t('pipeline.stage.voiceover')}
                                                    {id === 'image' && t('pipeline.stage.image')}
                                                    {id === 'subtitle' && t('pipeline.stage.subtitle')}
                                                    {id === 'montage' && t('pipeline.stage.montage')}
                                                </div>
                                                <div className="dashboard-item-actions-static">
                                                    <button className="item-action-btn" onClick={(e) => { e.stopPropagation(); moveSection(id, 'left'); }} disabled={colIdx === 0} title="Move Left">
                                                        <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" strokeWidth="3" fill="none"><path d="M15 18l-6-6 6-6"/></svg>
                                                    </button>
                                                    <div className="item-action-vertical">
                                                        <button className="item-action-btn" onClick={(e) => { e.stopPropagation(); moveSection(id, 'up'); }} disabled={itemIdx === 0} title="Move Up">
                                                            <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" strokeWidth="3" fill="none"><path d="M18 15l-6-6-6 6"/></svg>
                                                        </button>
                                                        <button className="item-action-btn" onClick={(e) => { e.stopPropagation(); moveSection(id, 'down'); }} disabled={itemIdx === col.length - 1} title="Move Down">
                                                            <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" strokeWidth="3" fill="none"><path d="M6 9l6 6 6-6"/></svg>
                                                        </button>
                                                    </div>
                                                    <button className="item-action-btn" onClick={(e) => { e.stopPropagation(); moveSection(id, 'right'); }} disabled={colIdx === 3} title="Move Right">
                                                        <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" strokeWidth="3" fill="none"><path d="M9 18l6-6-6-6"/></svg>
                                                    </button>
                                                </div>
                                            </div>
                                        ) : renderSection(id)}
                                    </div>
                                </DashboardErrorBoundary>
                            ))}
                            {isLayoutMode && col.length === 0 && (
                                <div 
                                    className="dashboard-column-drop-zone"
                                    onDragOver={(e) => handleDragOver(e, colIdx, 0)}
                                    onDrop={handleDrop}
                                >
                                    DROP HERE
                                </div>
                            )}
                        </div>
                    ))}
                </div>
            </main>

            <footer className="dashboard-footer">
                <button 
                    className="add-to-queue-btn dashboard-add-btn" 
                    onClick={() => setIsModalOpen(true)}
                >
                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                        <line x1="12" y1="5" x2="12" y2="19"></line>
                        <line x1="5" y1="12" x2="19" y2="12"></line>
                    </svg>
                    {t('pipeline.add_to_queue')}
                </button>
            </footer>
        </div>
        </DashboardErrorBoundary>
    );
};

