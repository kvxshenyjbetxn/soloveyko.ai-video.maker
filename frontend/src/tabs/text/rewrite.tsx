import { useState, useCallback, useRef } from 'react';
import { useI18n } from '../../contexts/I18nContext';
import { PipelineSidebar } from '../../components/PipelineSidebar';

export const Rewrite = () => {
    const { t } = useI18n();
    const [text, setText] = useState("");
    const [isDragging, setIsDragging] = useState(false);
    const [showPipelineSettings, setShowPipelineSettings] = useState(true);
    const dragCounter = useRef(0);

    const updateText = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
        setText(e.target.value);
    };

    const handleDragEnter = useCallback((e: React.DragEvent) => {
        e.preventDefault();
        e.stopPropagation();
        dragCounter.current++;
        if (e.dataTransfer.items && e.dataTransfer.items.length > 0) {
            setIsDragging(true);
        }
    }, []);

    const handleDragLeave = useCallback((e: React.DragEvent) => {
        e.preventDefault();
        e.stopPropagation();
        dragCounter.current--;
        if (dragCounter.current === 0) {
            setIsDragging(false);
        }
    }, []);

    const handleDragOver = useCallback((e: React.DragEvent) => {
        e.preventDefault();
        e.stopPropagation();
    }, []);

    const handleDrop = useCallback((e: React.DragEvent) => {
        e.preventDefault();
        e.stopPropagation();
        setIsDragging(false);
        dragCounter.current = 0;

        const files = e.dataTransfer.files;
        if (files && files.length > 0) {
            const file = files[0];
            if (file.type === 'text/plain' || file.name.endsWith('.txt')) {
                const reader = new FileReader();
                reader.onload = (event) => {
                    const content = event.target?.result;
                    if (typeof content === 'string') {
                        setText(content);
                    }
                };
                reader.readAsText(file);
            }
        }
    }, []);

    const characterCount = text.length;
    const wordCount = text.trim() === "" ? 0 : text.trim().split(/\s+/).length;
    const paragraphCount = text.trim() === "" ? 0 : text.trim().split(/\n+/).length;

    return (
        <div className="content-wrapper" style={{ flexDirection: 'row', padding: 0 }}>
            <div className="main-editor-area" style={{ flex: 1, display: 'flex', flexDirection: 'column', padding: '24px', overflow: 'hidden' }}>
                <div
                    className={`script-editor-container ${isDragging ? 'drag-over' : ''}`}
                    onDragOver={handleDragOver}
                    onDragEnter={handleDragEnter}
                    onDragLeave={handleDragLeave}
                    onDrop={handleDrop}
                >
                    <textarea
                        className="script-input"
                        value={text}
                        onChange={updateText}
                        placeholder={t('text.placeholder')}
                        spellCheck={false}
                    />
                    <div className="stats-bar">
                        <div className="stat-item">
                            <span className="stat-label">{t('stats.characters')}</span>
                            <span className="stat-value">{characterCount}</span>
                        </div>
                        <div className="stat-separator"></div>
                        <div className="stat-item">
                            <span className="stat-label">{t('stats.words')}</span>
                            <span className="stat-value">{wordCount}</span>
                        </div>
                        <div className="stat-separator"></div>
                        <div className="stat-item">
                            <span className="stat-label">{t('stats.paragraphs')}</span>
                            <span className="stat-value">{paragraphCount}</span>
                        </div>
                        <div style={{ marginLeft: 'auto' }}>
                            <button
                                className="sidebar-toggle-btn"
                                onClick={() => setShowPipelineSettings(!showPipelineSettings)}
                                style={{
                                    background: 'transparent',
                                    border: 'none',
                                    color: 'var(--text-secondary)',
                                    cursor: 'pointer',
                                    display: 'flex',
                                    alignItems: 'center',
                                    gap: '6px',
                                    fontSize: '12px'
                                }}
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" /></svg>
                                {showPipelineSettings ? t('pipeline.hide_settings') : t('pipeline.show_settings')}
                            </button>
                        </div>
                    </div>
                </div>
            </div>
            {showPipelineSettings && <PipelineSidebar type="rewrite" />}
        </div>
    );
};
