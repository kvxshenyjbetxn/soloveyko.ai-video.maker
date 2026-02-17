import React, { useState, useEffect } from 'react';
import { useI18n } from '../../../../contexts/I18nContext';
import { useTheme } from '../../../../contexts/ThemeContext';
// @ts-ignore
import { GetElevenLabsUAAPIKey, SaveElevenLabsUAAPIKey } from '../../../../../wailsjs/go/main/App';
import '../../general.css';

export const ElevenLabsUA = () => {
    const { t } = useI18n();
    const { accentColor } = useTheme();

    const [apiKey, setApiKey] = useState('');
    const [isLoaded, setIsLoaded] = useState(false);
    const [statusMsg, setStatusMsg] = useState<{ type: 'success' | 'error', text: string } | null>(null);

    useEffect(() => {
        const loadKey = async () => {
            try {
                const key = await GetElevenLabsUAAPIKey();
                setApiKey(key || '');
            } catch (err) {
                console.error("Failed to load ElevenLabsUA API key:", err);
            } finally {
                setIsLoaded(true);
            }
        };
        loadKey();
    }, []);

    useEffect(() => {
        if (!isLoaded) return;
        const timer = setTimeout(async () => {
            try {
                await SaveElevenLabsUAAPIKey(apiKey);
                setStatusMsg({ type: 'success', text: 'Saved' });
                setTimeout(() => {
                    setStatusMsg(prev => prev?.text === 'Saved' ? null : prev);
                }, 2000);
            } catch (err) {
                setStatusMsg({ type: 'error', text: 'Error' });
            }
        }, 1000);
        return () => clearTimeout(timer);
    }, [apiKey, isLoaded]);

    return (
        <div className="content-wrapper animate-fade">
            <div className="settings-container" style={{ maxWidth: '1000px' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '30px' }}>
                    <h2 className="settings-title" style={{ margin: 0 }}>ElevenLabs UA</h2>
                </div>

                <div className="settings-section glass-panel" style={{ padding: '25px', borderRadius: '12px', background: 'rgba(255, 255, 255, 0.03)', border: '1px solid rgba(255, 255, 255, 0.05)', marginBottom: '30px' }}>
                    <h3 className="section-title" style={{ marginBottom: '20px', fontSize: '1.1em', opacity: 0.9 }}>{t('settings.voice.apiKey')}</h3>
                    <div style={{ position: 'relative' }}>
                        <input
                            type="password"
                            className="premium-input"
                            style={{
                                width: '100%',
                                padding: '12px 16px',
                                borderRadius: '8px',
                                border: `1px solid ${statusMsg?.type === 'success' && statusMsg.text === 'Saved' ? '#4caf5044' : 'rgba(255, 255, 255, 0.08)'}`,
                                background: 'rgba(0, 0, 0, 0.3)',
                                color: '#fff',
                                outline: 'none',
                                fontSize: '0.95em',
                                transition: 'all 0.3s ease',
                                boxSizing: 'border-box'
                            }}
                            value={apiKey}
                            onChange={(e) => {
                                setApiKey(e.target.value);
                                setStatusMsg({ type: 'success', text: 'Typing...' });
                            }}
                            placeholder="X-API-Key / xi-api-key..."
                        />
                        {statusMsg && (
                            <div style={{
                                position: 'absolute',
                                right: '12px',
                                top: '50%',
                                transform: 'translateY(-50%)',
                                color: statusMsg.type === 'success' ? '#4caf50' : '#ff5252',
                                fontSize: '0.75em',
                                fontWeight: '600',
                                opacity: 0.8,
                                pointerEvents: 'none',
                                display: 'flex',
                                alignItems: 'center',
                                gap: '5px'
                            }}>
                                {statusMsg.text === 'Saved' && <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>}
                                {statusMsg.text}
                            </div>
                        )}
                    </div>
                </div>

                <div className="stat-group glass-panel" style={{ padding: '25px', borderRadius: '12px', background: 'rgba(255, 255, 255, 0.02)', border: '1px solid rgba(255, 255, 255, 0.05)' }}>
                    <div style={{ display: 'flex', alignItems: 'flex-start', gap: '15px' }}>
                        <div style={{ width: '50px', height: '50px', borderRadius: '10px', background: 'rgba(255,255,255,0.05)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke={accentColor} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"></path><path d="M19 10v2a7 7 0 0 1-14 0v-2"></path><line x1="12" y1="19" x2="12" y2="23"></line><line x1="8" y1="23" x2="16" y2="23"></line></svg>
                        </div>
                        <div>
                            <div style={{ opacity: 0.5, fontSize: '0.8em', textTransform: 'uppercase' }}>Supported Models</div>
                            <div style={{ marginTop: '5px', display: 'flex', flexWrap: 'wrap', gap: '8px' }}>
                                <span style={{ padding: '4px 10px', borderRadius: '4px', background: 'rgba(255,255,255,0.05)', fontSize: '0.85em' }}>Multilingual v2</span>
                                <span style={{ padding: '4px 10px', borderRadius: '4px', background: 'rgba(255,255,255,0.05)', fontSize: '0.85em' }}>v3 (Emotions)</span>
                                <span style={{ padding: '4px 10px', borderRadius: '4px', background: 'rgba(255,255,255,0.05)', fontSize: '0.85em' }}>Flash v2.5</span>
                                <span style={{ padding: '4px 10px', borderRadius: '4px', background: 'rgba(255,255,255,0.05)', fontSize: '0.85em' }}>Turbo v2.5</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
};
