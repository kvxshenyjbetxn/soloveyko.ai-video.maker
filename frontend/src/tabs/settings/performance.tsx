import { useState, useEffect } from 'react';
import { useI18n } from '../../contexts/I18nContext';
import { GetSubtitleMaxConnections, SaveSubtitleMaxConnections } from '../../../wailsjs/go/main/App';
import './general.css';

export const Performance = () => {
    const { t } = useI18n();
    const [subtitleMax, setSubtitleMax] = useState(2);
    const [isSaving, setIsSaving] = useState(false);

    useEffect(() => {
        GetSubtitleMaxConnections().then(max => {
            setSubtitleMax(max || 2);
        });
    }, []);

    const handleSubtitleMaxChange = async (val: number) => {
        setSubtitleMax(val);
        setIsSaving(true);
        try {
            await SaveSubtitleMaxConnections(val);
        } catch (err) {
            console.error('Failed to save subtitle max connections', err);
        } finally {
            setIsSaving(false);
        }
    };

    const progress = ((subtitleMax - 1) / (5 - 1)) * 100;

    return (
        <div className="content-wrapper animate-fade">
            <div className="settings-container">
                <div className="settings-section">
                    <h3 className="section-title">{t('settings.performance')}</h3>
                    <p className="section-description">
                        {t('performanceTab.description')}
                    </p>
                </div>

                <div className="settings-section">
                    <h4 className="section-title" style={{
                        fontSize: '12px',
                        opacity: 0.6,
                        textTransform: 'uppercase',
                        letterSpacing: '0.05em',
                        marginBottom: '16px'
                    }}>
                        {t('performanceTab.subtitle_block')}
                    </h4>

                    <div style={{
                        background: 'rgba(255, 255, 255, 0.02)',
                        border: '1px solid var(--border-color)',
                        borderRadius: '12px',
                        padding: '20px'
                    }}>
                        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
                            <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                                <span style={{ fontSize: '14px', fontWeight: '600' }}>
                                    {t('performanceTab.subtitle_max_concurrency')}
                                </span>
                                <span style={{ fontSize: '12px', color: 'var(--text-tertiary)', maxWidth: '400px', lineHeight: '1.4' }}>
                                    {t('performanceTab.subtitle_max_concurrency_desc')}
                                </span>
                            </div>
                            <div style={{
                                fontSize: '24px',
                                fontWeight: '800',
                                color: 'var(--accent-primary)',
                                minWidth: '30px',
                                textAlign: 'right',
                                fontFamily: 'var(--font-mono)'
                            }}>
                                {subtitleMax}
                            </div>
                        </div>

                        <div className="settings-control" style={{ marginTop: '10px' }}>
                            <input
                                type="range"
                                min="1"
                                max="5"
                                className="settings-slider"
                                value={subtitleMax}
                                onChange={(e) => handleSubtitleMaxChange(parseInt(e.target.value))}
                                style={{
                                    width: '100%',
                                    margin: '0',
                                    '--range-progress': `${progress}%`
                                } as React.CSSProperties}
                            />
                            <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: '8px', padding: '0 2px' }}>
                                {[1, 2, 3, 4, 5].map(v => (
                                    <span key={v} style={{
                                        fontSize: '10px',
                                        color: subtitleMax === v ? 'var(--accent-primary)' : 'var(--text-tertiary)',
                                        fontWeight: subtitleMax === v ? '700' : '400',
                                        transition: 'all 0.2s ease'
                                    }}>
                                        {v}
                                    </span>
                                ))}
                            </div>
                        </div>
                    </div>
                </div>

                <div style={{
                    marginTop: 'auto',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '8px',
                    height: '20px'
                }}>
                    {isSaving && (
                        <>
                            <div className="spinner-tiny" style={{ borderTopColor: 'var(--accent-primary)' }} />
                            <span style={{
                                fontSize: '12px',
                                color: 'var(--text-tertiary)',
                                fontStyle: 'italic'
                            }}>
                                {t('performanceTab.saving')}
                            </span>
                        </>
                    )}
                </div>
            </div>
        </div>
    );
};
