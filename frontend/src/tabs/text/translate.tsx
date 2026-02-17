import { useState, useCallback, useRef } from 'react';
import { useI18n } from '../../contexts/I18nContext';

export const Translate = () => {
    const { t } = useI18n();
    const [text, setText] = useState("");
    const [isDragging, setIsDragging] = useState(false);
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
        // Just prevent default, don't update state to avoid flickering
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
        <div className="content-wrapper">
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
                </div>
            </div>
        </div>
    );
};
