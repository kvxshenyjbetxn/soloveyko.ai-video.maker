import React, { useState, useEffect } from 'react';
import { useI18n } from '../../../../contexts/I18nContext';
import { useTheme } from '../../../../contexts/ThemeContext';
import { useLogger } from '../../../../contexts/LoggerContext';
// @ts-ignore
import { GetElevenLabsImageAPIKey, SaveElevenLabsImageAPIKey } from '../../../../../wailsjs/go/main/App';
import '../../general.css';

export const ElevenLabsImage = () => {
    const { t } = useI18n();
    const { accentColor } = useTheme();
    const { addLog } = useLogger();

    const [apiKey, setApiKey] = useState('');
    const [isLoaded, setIsLoaded] = useState(false);
    const [statusMsg, setStatusMsg] = useState<{ type: 'success' | 'error', text: string } | null>(null);

    useEffect(() => {
        const loadKey = async () => {
            try {
                const key = await GetElevenLabsImageAPIKey();
                setApiKey(key || '');
            } catch (err) {
                console.error("Failed to load ElevenLabsImage API key:", err);
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
                await SaveElevenLabsImageAPIKey(apiKey);
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
                    <h2 className="settings-title" style={{ margin: 0 }}>ElevenLabs Image</h2>
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
                            placeholder="X-API-Key..."
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
                            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke={accentColor} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><circle cx="8.5" cy="8.5" r="1.5"></circle><polyline points="21 15 16 10 5 21"></polyline></svg>
                        </div>
                        <div>
                            <div style={{ opacity: 0.5, fontSize: '0.8em', textTransform: 'uppercase' }}>Available Methods</div>
                            <div style={{ marginTop: '5px', display: 'flex', flexWrap: 'wrap', gap: '8px' }}>
                                <span style={{ padding: '4px 10px', borderRadius: '4px', background: 'rgba(255,255,255,0.05)', fontSize: '0.85em' }}>Create</span>
                                <span style={{ padding: '4px 10px', borderRadius: '4px', background: 'rgba(255,255,255,0.05)', fontSize: '0.85em' }}>Edit</span>
                                <span style={{ padding: '4px 10px', borderRadius: '4px', background: 'rgba(255,255,255,0.05)', fontSize: '0.85em' }}>Remix</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
};
