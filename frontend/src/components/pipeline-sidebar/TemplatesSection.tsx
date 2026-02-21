import React from 'react';
import { useI18n } from '../../contexts/I18nContext';

interface TemplatesSectionProps {
    type: string;
    templates: any[];
    selectedTemplateIds: string[];
    toggleTemplate: (id: string) => void;
    applyTemplate: (tpl: any) => void;
    setTemplateToDelete: (tpl: any) => void;
    isCollapsed: boolean;
    onToggleCollapse: (collapsed: boolean) => void;
    setCurrentPath?: (path: string) => void;
}

const TemplatesIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <rect x="3" y="3" width="7" height="7" />
        <rect x="14" y="3" width="7" height="7" />
        <rect x="14" y="14" width="7" height="7" />
        <rect x="3" y="14" width="7" height="7" />
    </svg>
);

export const TemplatesSection: React.FC<TemplatesSectionProps> = ({
    type, templates, selectedTemplateIds, toggleTemplate, applyTemplate, setTemplateToDelete, isCollapsed, onToggleCollapse, setCurrentPath
}) => {
    const { t } = useI18n();

    return (
        <div className={`pipeline-stage-container ${isCollapsed ? 'is-collapsed' : ''}`}>
            <div
                className="pipeline-stage-header"
                onClick={() => onToggleCollapse(!isCollapsed)}
            >
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flex: 1 }}>
                    <svg
                        className={`stage-chevron ${isCollapsed ? 'rotated' : ''}`}
                        xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"
                    >
                        <path d="m6 9 6 6 6-6" />
                    </svg>
                    <div style={{
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        width: '28px',
                        height: '28px',
                        borderRadius: '8px',
                        background: 'rgba(var(--accent-rgb), 0.1)',
                        color: 'var(--accent-color)',
                        transition: 'all 0.3s'
                    }}>
                        <TemplatesIcon />
                    </div>
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
            <div className={`stage-settings-content ${isCollapsed ? 'collapsed' : ''}`}>
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
    );
};
