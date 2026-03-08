import React from 'react';
import { useI18n } from '../../contexts/I18nContext';

interface ImageSectionProps {
    settings: any;
    handleChange: (field: string, value: any) => void;
    setSettings: React.Dispatch<React.SetStateAction<any>>;
    fetchPollinationsModels: () => void;
    pollinationsModels: string[];
    loadingPollinationsModels: boolean;
    estimatedChunks: number;
    content: string;
    models: string[];
    renderValueOrInput: (field: string, value: number, isFloat: boolean) => React.ReactNode;
    setCurrentPath?: (path: string) => void;
    elevenLabsImageKeys?: any[];
}

const ImageIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
        <circle cx="8.5" cy="8.5" r="1.5" />
        <polyline points="21 15 16 10 5 21" />
    </svg>
);

export const ImageSection: React.FC<ImageSectionProps> = ({
    settings, handleChange, setSettings, fetchPollinationsModels, pollinationsModels, loadingPollinationsModels, estimatedChunks, content, models, renderValueOrInput, setCurrentPath, elevenLabsImageKeys
}) => {
    const { t } = useI18n();
    const [previewUrl, setPreviewUrl] = React.useState<string | null>(null);

    React.useEffect(() => {
        if (settings.imageGooglerReferenceImage) {
            // Load preview
            const loadPreview = async () => {
                try {
                    const b64 = await (window as any).go.main.App.GetImageAsBase64(settings.imageGooglerReferenceImage);
                    if (b64) setPreviewUrl(b64);
                } catch (err) {
                    console.error("Failed to load preview:", err);
                    setPreviewUrl(null);
                }
            };
            loadPreview();
        } else {
            setPreviewUrl(null);
        }
    }, [settings.imageGooglerReferenceImage]);

    return (
        <div className={`pipeline-stage-container ${settings.imageCollapsed ? 'is-collapsed' : ''}`} >
            <div
                className="pipeline-stage-header"
                onClick={() => handleChange('imageCollapsed', !settings.imageCollapsed)}
            >
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flex: 1 }}>
                    <svg
                        className={`stage-chevron ${settings.imageCollapsed ? 'rotated' : ''}`}
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
                        background: settings.imageEnabled ? 'rgba(var(--accent-rgb), 0.1)' : 'var(--bg-tertiary)',
                        color: settings.imageEnabled ? 'var(--accent-color)' : 'var(--text-tertiary)',
                        transition: 'all 0.3s'
                    }}>
                        <ImageIcon />
                    </div>
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
                                imageEnabled: val
                            }));
                        }}
                    />
                    <span className="stage-slider"></span>
                </label>
            </div>

            <div className={`stage-settings-content ${settings.imageCollapsed ? 'collapsed' : ''}`}>
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

                    {settings.imageGenerationMethod === 'sentences' && (
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
                                    <span style={{ fontSize: '11px', color: 'var(--text-secondary)' }}>{t('pipeline.image.symbol_limit') || 'Ліміт символів:'} {settings.imageSentenceLimit ?? 1000}</span>
                                    <input
                                        type="range"
                                        className="settings-slider"
                                        min="50"
                                        max="5000"
                                        step="50"
                                        value={settings.imageSentenceLimit ?? 1000}
                                        style={{ '--range-progress': `${((settings.imageSentenceLimit ?? 1000) - 50) / 4950 * 100}%`, marginTop: '8px', width: '100%' } as React.CSSProperties}
                                        onChange={(e) => handleChange('imageSentenceLimit', parseInt(e.target.value))}
                                    />
                                </div>
                            )}
                        </div>
                    )}

                    {(settings.imageGenerationMethod === 'lines' || (settings.imageGenerationMethod === 'sentences' && settings.imageGroupSentences)) && (
                        <div className="settings-control" style={{ marginTop: '12px' }}>
                            <label className="settings-label">{t('pipeline.image.initial_sentences') || 'Динамічний початок (речень)'}</label>
                            <div className="settings-description" style={{ fontSize: '11px', color: 'var(--text-secondary)', marginBottom: '8px' }}>
                                {t('pipeline.image.initial_sentences_desc') || 'Вкажіть кількість перших частин, які будуть нарізані по одному (без групування)'}
                            </div>
                            <div className="settings-slider-container">
                                <input
                                    type="range"
                                    className="settings-slider"
                                    min="0"
                                    max="100"
                                    step="1"
                                    value={settings.imageInitialSentenceCount ?? 0}
                                    style={{ '--range-progress': `${((settings.imageInitialSentenceCount ?? 0) / 100) * 100}%` } as React.CSSProperties}
                                    onChange={(e) => handleChange('imageInitialSentenceCount', parseInt(e.target.value))}
                                />
                                <span style={{ fontSize: '12px', minWidth: '24px', textAlign: 'right', fontWeight: 500 }}>{settings.imageInitialSentenceCount ?? 0}</span>
                            </div>
                        </div>
                    )}

                    <div className="settings-control" style={{ borderTop: '1px solid var(--border-color)', marginTop: '12px', paddingTop: '12px' }}>
                        <label className="settings-label">{t('pipeline.image.mode') || 'Режим'}</label>
                        <div style={{ display: 'flex', background: 'var(--bg-tertiary)', borderRadius: '8px', padding: '4px', gap: '4px', marginBottom: '12px' }}>
                            <button
                                className={`method-toggle-btn ${(settings.imageMode || 'normal') === 'normal' ? 'active' : ''}`}
                                onClick={() => handleChange('imageMode', 'normal')}
                                style={{
                                    flex: 1, padding: '6px', borderRadius: '6px', border: 'none',
                                    background: (settings.imageMode || 'normal') === 'normal' ? 'var(--bg-primary)' : 'transparent',
                                    color: (settings.imageMode || 'normal') === 'normal' ? 'var(--text-primary)' : 'var(--text-secondary)',
                                    cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '6px',
                                    fontSize: '12px', fontWeight: (settings.imageMode || 'normal') === 'normal' ? 500 : 400,
                                    boxShadow: (settings.imageMode || 'normal') === 'normal' ? '0 2px 4px rgba(0,0,0,0.1)' : 'none',
                                    transition: 'all 0.2s'
                                }}
                            >
                                {t('pipeline.image.mode_normal') || 'Звичайний'}
                            </button>
                            <button
                                className={`method-toggle-btn ${settings.imageMode === 'memory' ? 'active' : ''}`}
                                onClick={() => handleChange('imageMode', 'memory')}
                                style={{
                                    flex: 1, padding: '6px', borderRadius: '6px', border: 'none',
                                    background: settings.imageMode === 'memory' ? 'var(--bg-primary)' : 'transparent',
                                    color: settings.imageMode === 'memory' ? 'var(--text-primary)' : 'var(--text-secondary)',
                                    cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '6px',
                                    fontSize: '12px', fontWeight: settings.imageMode === 'memory' ? 500 : 400,
                                    boxShadow: settings.imageMode === 'memory' ? '0 2px 4px rgba(0,0,0,0.1)' : 'none',
                                    transition: 'all 0.2s'
                                }}
                            >
                                {t('pipeline.image.mode_memory') || 'Пам\'ять'}
                            </button>
                        </div>

                        <div className="settings-control">
                            {settings.imageMode === 'memory' && (
                                <div style={{ display: 'flex', background: 'var(--bg-tertiary)', borderRadius: '8px', padding: '4px', gap: '4px', marginBottom: '12px' }}>
                                    <button
                                        className={`method-toggle-btn ${(settings.imageMemoryType || 'primitive') === 'primitive' ? 'active' : ''}`}
                                        onClick={() => handleChange('imageMemoryType', 'primitive')}
                                        style={{
                                            flex: 1, padding: '6px', borderRadius: '6px', border: 'none',
                                            background: (settings.imageMemoryType || 'primitive') === 'primitive' ? 'var(--bg-primary)' : 'transparent',
                                            color: (settings.imageMemoryType || 'primitive') === 'primitive' ? 'var(--text-primary)' : 'var(--text-secondary)',
                                            cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '6px',
                                            fontSize: '12px', fontWeight: (settings.imageMemoryType || 'primitive') === 'primitive' ? 500 : 400,
                                            boxShadow: (settings.imageMemoryType || 'primitive') === 'primitive' ? '0 2px 4px rgba(0,0,0,0.1)' : 'none',
                                            transition: 'all 0.2s'
                                        }}
                                    >
                                        {t('pipeline.image.memory_type_primitive') || 'Примітивно'}
                                    </button>
                                    <button
                                        className={`method-toggle-btn ${settings.imageMemoryType === 'external' ? 'active' : ''}`}
                                        onClick={() => handleChange('imageMemoryType', 'external')}
                                        style={{
                                            flex: 1, padding: '6px', borderRadius: '6px', border: 'none',
                                            background: settings.imageMemoryType === 'external' ? 'var(--bg-primary)' : 'transparent',
                                            color: settings.imageMemoryType === 'external' ? 'var(--text-primary)' : 'var(--text-secondary)',
                                            cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '6px',
                                            fontSize: '12px', fontWeight: settings.imageMemoryType === 'external' ? 500 : 400,
                                            boxShadow: settings.imageMemoryType === 'external' ? '0 2px 4px rgba(0,0,0,0.1)' : 'none',
                                            transition: 'all 0.2s'
                                        }}
                                    >
                                        {t('pipeline.image.memory_type_external') || 'Сторонні сервіси'}
                                    </button>
                                </div>
                            )}

                            {((settings.imageMode || 'normal') === 'normal' || ((settings.imageMode === 'memory') && (settings.imageMemoryType || 'primitive') === 'primitive')) && (
                                <>
                                    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%', marginTop: settings.imageMode === 'memory' ? '4px' : '0' }}>
                                        <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.image.determine_characters') || 'Визначити персонажів'}</label>
                                        <label className="stage-switch small">
                                            <input
                                                type="checkbox"
                                                checked={settings.imageDetermineCharacters || false}
                                                onChange={(e) => handleChange('imageDetermineCharacters', e.target.checked)}
                                            />
                                            <span className="stage-slider"></span>
                                        </label>
                                    </div>
                                    <div className="settings-description" style={{ fontSize: '11px', color: 'var(--text-secondary)', marginTop: '8px' }}>
                                        {t('pipeline.image.determine_characters_desc') || 'Автоматично визначити опис персонажів із тексту для використання в промптах'}
                                    </div>

                                    {settings.imageDetermineCharacters && (
                                        <div className="settings-control" style={{ marginTop: '12px' }}>
                                            <label className="settings-label">{t('pipeline.image.determine_characters_prompt') || 'Промт для визначення персонажів'}</label>
                                            <textarea
                                                className="settings-textarea"
                                                style={{ height: '80px', resize: 'vertical' }}
                                                value={settings.imageDetermineCharactersPrompt || ''}
                                                onChange={(e) => handleChange('imageDetermineCharactersPrompt', e.target.value)}
                                                placeholder={t('pipeline.image.determine_characters_prompt_desc') || 'Введіть промт для визначення персонажів...'}
                                            />
                                        </div>
                                    )}

                                    {settings.imageMode === 'memory' && (
                                        <div className="settings-slider-container" style={{ marginTop: '16px', paddingTop: '16px', borderTop: '1px solid var(--border-color)' }}>
                                            <span style={{ fontSize: '11px', color: 'var(--text-secondary)' }}>{t('pipeline.image.memory_chars') || "Кількість символів пам'яті:"} {settings.imageMemoryChars ?? 1000}</span>
                                            <input
                                                type="range"
                                                className="settings-slider"
                                                min="500"
                                                max="5000"
                                                step="100"
                                                value={settings.imageMemoryChars ?? 1000}
                                                style={{ '--range-progress': `${((settings.imageMemoryChars ?? 1000) - 500) / 4500 * 100}%`, marginTop: '8px', width: '100%' } as React.CSSProperties}
                                                onChange={(e) => handleChange('imageMemoryChars', parseInt(e.target.value))}
                                            />
                                        </div>
                                    )}
                                </>
                            )}
                        </div>
                    </div>

                    <div className="settings-control">

                        <label className="settings-label">{t('pipeline.image.prompt') || 'Промт для інструкцій'}</label>
                        <textarea
                            className="settings-textarea"
                            style={{ height: '80px', resize: 'vertical' }}
                            value={settings.imagePrompt || ''}
                            onChange={(e) => handleChange('imagePrompt', e.target.value)}
                            placeholder={t('pipeline.image.prompt_placeholder') || 'Введіть промт...'}
                        />
                        <div className="settings-description" style={{ fontSize: '11px', color: 'var(--text-secondary)', marginTop: '4px' }}>
                            {t('pipeline.image.prompt_placeholder')?.split('...').pop()?.trim() || 'Використовуйте {{content}} для вставки тексту'}
                        </div>

                        {content && content.trim() !== '' && (
                            <div style={{ fontSize: '11px', color: 'var(--accent-primary)', marginTop: '8px', fontWeight: 500, display: 'flex', alignItems: 'center', gap: '6px' }}>
                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10"></circle><polyline points="12 6 12 12 16 14"></polyline></svg>
                                {t('pipeline.image.estimated_chunks') || 'Орієнтовна кількість промптів: '} {estimatedChunks}
                            </div>
                        )}
                    </div>

                    <div className="settings-control" style={{ borderTop: '1px solid var(--border-color)', paddingTop: '12px' }}>
                        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                            <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.image.sync_enabled') || 'Синхронний режим (SRT)'}</label>
                            <label className="stage-switch small">
                                <input
                                    type="checkbox"
                                    checked={settings.imageSyncEnabled || false}
                                    onChange={(e) => handleChange('imageSyncEnabled', e.target.checked)}
                                />
                                <span className="stage-slider"></span>
                            </label>
                        </div>
                        <div className="settings-description" style={{ fontSize: '11px', color: 'var(--text-secondary)', marginTop: '8px' }}>
                            {t('pipeline.image.sync_desc') || 'Зміна картинок відбуватиметься синхронно з вимовою тексту (через SRT)'}
                        </div>
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

                    {settings.imageService === 'googler' && (
                        <>
                            <div className="settings-control">
                                <label className="settings-label">{t('pipeline.image.model')}</label>
                                <select
                                    className="settings-select"
                                    value={settings.imageGooglerModel || 'whisk'}
                                    onChange={(e) => handleChange('imageGooglerModel', e.target.value)}
                                >
                                    <option value="whisk">Whisk (v4)</option>
                                    <option value="flow">Flow (v4)</option>
                                    <option value="grok">Grok (v4)</option>
                                    <option value="gemini">Gemini (v4)</option>
                                </select>
                            </div>

                            <div className="settings-control">
                                <label className="settings-label">{t('pipeline.image.aspect_ratio') || 'Співвідношення сторін'}</label>
                                <select
                                    className="settings-select"
                                    value={settings.imageGooglerAspectRatio || 'IMAGE_ASPECT_RATIO_LANDSCAPE'}
                                    onChange={(e) => handleChange('imageGooglerAspectRatio', e.target.value)}
                                >
                                    <option value="IMAGE_ASPECT_RATIO_PORTRAIT">{t('pipeline.image.aspect_ratio_portrait') || 'Портрет (9:16)'}</option>
                                    <option value="IMAGE_ASPECT_RATIO_LANDSCAPE">{t('pipeline.image.aspect_ratio_landscape') || 'Ландшафт (16:9)'}</option>
                                </select>
                            </div>

                            {settings.imageGooglerModel === 'whisk' && (
                                <>
                                    <div className="settings-control">
                                        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                                            <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.image.googler.remix_enabled')}</label>
                                            <label className="stage-switch small">
                                                <input
                                                    type="checkbox"
                                                    checked={settings.imageGooglerRemixEnabled || false}
                                                    onChange={(e) => handleChange('imageGooglerRemixEnabled', e.target.checked)}
                                                />
                                                <span className="stage-slider"></span>
                                            </label>
                                        </div>
                                    </div>

                                    {settings.imageGooglerRemixEnabled && (
                                        <>
                                            <div className="settings-control">
                                                <div
                                                    onClick={async () => {
                                                        try {
                                                            const path = await (window as any).go.main.App.SelectImage();
                                                            if (path) {
                                                                handleChange('imageGooglerReferenceImage', path);
                                                            }
                                                        } catch (err) {
                                                            console.error(err);
                                                        }
                                                    }}
                                                    style={{
                                                        width: '100%',
                                                        padding: '16px',
                                                        borderRadius: '12px',
                                                        border: settings.imageGooglerReferenceImage ? '1px solid var(--accent-color)' : '1px dashed var(--bg-tertiary)',
                                                        backgroundColor: settings.imageGooglerReferenceImage ? 'rgba(var(--accent-rgb), 0.05)' : 'var(--bg-secondary)',
                                                        backgroundImage: previewUrl ? `url(${previewUrl})` : 'none',
                                                        backgroundSize: 'contain',
                                                        backgroundRepeat: 'no-repeat',
                                                        backgroundPosition: 'center',
                                                        display: 'flex',
                                                        flexDirection: 'column',
                                                        alignItems: 'center',
                                                        justifyContent: 'center',
                                                        gap: '8px',
                                                        cursor: 'pointer',
                                                        transition: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
                                                        position: 'relative',
                                                        overflow: 'hidden',
                                                        minHeight: '100px'
                                                    }}
                                                    className="image-remix-dropzone"
                                                >
                                                    {previewUrl && (
                                                        <div style={{
                                                            position: 'absolute',
                                                            inset: 0,
                                                            backgroundColor: 'rgba(0,0,0,0.4)',
                                                            zIndex: 1
                                                        }} />
                                                    )}

                                                    <div style={{
                                                        fontSize: '24px',
                                                        opacity: settings.imageGooglerReferenceImage ? 1 : 0.5,
                                                        filter: settings.imageGooglerReferenceImage ? 'drop-shadow(0 0 8px var(--accent-color))' : 'none',
                                                        position: 'relative',
                                                        zIndex: 2
                                                    }}>
                                                        {settings.imageGooglerReferenceImage ? '🖼️' : '📁'}
                                                    </div>
                                                    <div style={{
                                                        fontSize: '11px',
                                                        fontWeight: '600',
                                                        color: settings.imageGooglerReferenceImage ? '#fff' : 'var(--text-secondary)',
                                                        textAlign: 'center',
                                                        position: 'relative',
                                                        zIndex: 2,
                                                        textShadow: previewUrl ? '0 1px 4px rgba(0,0,0,0.8)' : 'none'
                                                    }}>
                                                        {settings.imageGooglerReferenceImage
                                                            ? t('pipeline.image.googler.remix_change')
                                                            : t('pipeline.image.googler.remix_select')}
                                                    </div>
                                                    {settings.imageGooglerReferenceImage && (
                                                        <div style={{
                                                            fontSize: '9px',
                                                            color: '#ddd',
                                                            maxWidth: '100%',
                                                            overflow: 'hidden',
                                                            textOverflow: 'ellipsis',
                                                            whiteSpace: 'nowrap',
                                                            opacity: 0.9,
                                                            position: 'relative',
                                                            zIndex: 2,
                                                            textShadow: '0 1px 2px rgba(0,0,0,0.8)'
                                                        }}>
                                                            {settings.imageGooglerReferenceImage.split(/[\\/]/).pop()}
                                                        </div>
                                                    )}

                                                    {!settings.imageGooglerReferenceImage && (
                                                        <div style={{
                                                            position: 'absolute',
                                                            bottom: '4px',
                                                            fontSize: '8px',
                                                            color: 'var(--text-tertiary)',
                                                            opacity: 0.3
                                                        }}>
                                                            JPG, PNG, WEBP
                                                        </div>
                                                    )}
                                                </div>
                                            </div>

                                            <div className="settings-control">
                                                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                                                    <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.image.googler.strict_mode')}</label>
                                                    <label className="stage-switch small">
                                                        <input
                                                            type="checkbox"
                                                            checked={settings.imageGooglerRemixStrictMode || false}
                                                            onChange={(e) => handleChange('imageGooglerRemixStrictMode', e.target.checked)}
                                                        />
                                                        <span className="stage-slider"></span>
                                                    </label>
                                                </div>
                                            </div>
                                        </>
                                    )}
                                </>
                            )}

                            <div className="settings-control" style={{ marginTop: '16px', paddingTop: '16px', borderTop: '1px solid var(--border-color)' }}>
                                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                                    <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.image.googler.video_enabled') || 'Анімація картинок'}</label>
                                    <label className="stage-switch small">
                                        <input
                                            type="checkbox"
                                            checked={settings.imageGooglerVideoEnabled || false}
                                            onChange={(e) => handleChange('imageGooglerVideoEnabled', e.target.checked)}
                                        />
                                        <span className="stage-slider"></span>
                                    </label>
                                </div>
                            </div>

                            {settings.imageGooglerVideoEnabled && (
                                <>
                                    <div className="settings-control">
                                        <label className="settings-label">{t('pipeline.image.googler.video_model') || 'Модель відео'}</label>
                                        <select
                                            className="settings-select"
                                            value={settings.imageGooglerVideoModel || 'whisk'}
                                            onChange={(e) => handleChange('imageGooglerVideoModel', e.target.value)}
                                        >
                                            <option value="whisk">Whisk</option>
                                            <option value="flow">Flow</option>
                                            <option value="grok">Grok</option>
                                            <option value="gemini">Gemini</option>
                                        </select>
                                    </div>
                                    <div className="settings-control">
                                        <label className="settings-label">{t('pipeline.image.googler.video_mode') || 'Джерело анімації'}</label>
                                        <select
                                            className="settings-select"
                                            value={settings.imageGooglerVideoMode || 'text'}
                                            onChange={(e) => handleChange('imageGooglerVideoMode', e.target.value)}
                                        >
                                            <option value="text">{t('pipeline.image.googler.video_mode_text') || 'З тексту (промту)'}</option>
                                            <option value="image">{t('pipeline.image.googler.video_mode_image') || 'З згенерованого зображення'}</option>
                                        </select>
                                    </div>
                                    <div className="settings-control">
                                        <label className="settings-label">{t('pipeline.image.googler.video_count') || 'Кількість відео'}</label>
                                        <input
                                            type="number"
                                            className="settings-input"
                                            value={settings.imageGooglerVideoCount ?? 1}
                                            onChange={(e) => handleChange('imageGooglerVideoCount', parseInt(e.target.value))}
                                            min="1"
                                        />
                                    </div>
                                    {settings.imageGooglerVideoModel === 'grok' && (
                                        <div className="settings-control">
                                            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
                                                <label className="settings-label" style={{ marginBottom: 0 }}>{t('pipeline.image.googler.video_upscale') || 'Upscale відео (Grok)'}</label>
                                                <label className="stage-switch small">
                                                    <input
                                                        type="checkbox"
                                                        checked={settings.imageGooglerVideoUpscale || false}
                                                        onChange={(e) => handleChange('imageGooglerVideoUpscale', e.target.checked)}
                                                    />
                                                    <span className="stage-slider"></span>
                                                </label>
                                            </div>
                                        </div>
                                    )}
                                </>
                            )}
                        </>
                    )}

                    {settings.imageService === 'elevenlabsimage' && (
                        <>
                            <div className="settings-control">
                                <label className="settings-label">{t('pipeline.image.elevenlabsimage.aspect_ratio') || 'Співвідношення сторін'}</label>
                                <select
                                    className="settings-select"
                                    value={settings.elevenLabsImageAspectRatio || '16:9'}
                                    onChange={(e) => handleChange('elevenLabsImageAspectRatio', e.target.value)}
                                >
                                    <option value="16:9">16:9</option>
                                    <option value="9:16">9:16</option>
                                </select>
                            </div>
                        </>
                    )}
                </div>
            </div>
        </div >
    );
};
