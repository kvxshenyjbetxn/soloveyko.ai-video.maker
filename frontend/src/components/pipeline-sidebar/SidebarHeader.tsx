import React from 'react';
import { useI18n } from '../../contexts/I18nContext';

interface SidebarHeaderProps {
    type: 'translate' | 'rewrite' | 'voiceover';
    settings: any;
    handleChange: (field: string, value: any) => void;
    handleSaveTemplate: () => void;
}

export const SidebarHeader: React.FC<SidebarHeaderProps> = ({ type, settings, handleChange, handleSaveTemplate }) => {
    const { t } = useI18n();
    const isTranslate = type === 'translate';
    const isRewrite = type === 'rewrite';

    const pipelineName = (isTranslate ? settings.translatePipelineName : (isRewrite ? settings.rewritePipelineName : settings.voiceoverPipelineName)) || '';

    const onNameChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        let field = '';
        if (isTranslate) field = 'translatePipelineName';
        else if (isRewrite) field = 'rewritePipelineName';
        else field = 'voiceoverPipelineName';
        handleChange(field, e.target.value);
    };

    return (
        <div className="pipeline-sidebar-header" style={{ display: 'block', padding: '10px 12px', borderBottom: '1px solid var(--border-color)' }}>
            <div className="settings-control" style={{ marginBottom: 0 }}>
                <label className="settings-label" style={{ marginBottom: '4px', fontSize: '10px' }}>{t('pipeline.name')}</label>
                <div style={{ display: 'flex', gap: '8px' }}>
                    <input
                        className="settings-input"
                        value={pipelineName}
                        onChange={onNameChange}
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
    );
};
