import { useState } from 'react';
import { useI18n } from '../../contexts/I18nContext';

export const Rewrite = () => {
    const { t } = useI18n();
    const [text, setText] = useState("");

    const updateText = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
        setText(e.target.value);
    };

    const characterCount = text.length;
    const wordCount = text.trim() === "" ? 0 : text.trim().split(/\s+/).length;
    const paragraphCount = text.trim() === "" ? 0 : text.trim().split(/\n+/).length;

    return (
        <div className="content-wrapper">
            <div className="script-editor-container">
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
