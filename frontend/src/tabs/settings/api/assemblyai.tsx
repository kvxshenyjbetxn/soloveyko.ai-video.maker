import React, { useState, useEffect } from 'react';
import { useI18n } from '../../../contexts/I18nContext';
import { useTheme } from '../../../contexts/ThemeContext';
// @ts-ignore
import { GetAssemblyAIAPIKey, SaveAssemblyAIAPIKey, CheckAssemblyAIConnection } from '../../../../wailsjs/go/main/App';
import '../general.css';

export const AssemblyAI = () => {
    const { t } = useI18n();
    const { accentColor } = useTheme();

    const [apiKey, setApiKey] = useState('');
    const [isLoaded, setIsLoaded] = useState(false);
    const [loading, setLoading] = useState(false);
    const [statusMsg, setStatusMsg] = useState<{ type: 'success' | 'error', text: string } | null>(null);

    useEffect(() => {
        const loadKey = async () => {
            try {
                const key = await GetAssemblyAIAPIKey();
                setApiKey(key || '');
            } catch (err) {
                console.error('Failed to load AssemblyAI API key:', err);
            } finally {
                setIsLoaded(true);
            }
        };
        loadKey();
    }, []);

    useEffect(() => {
        if (!isLoaded) return;
        const timer = setTimeout(() => {
            SaveAssemblyAIAPIKey(apiKey).catch((err: any) => console.error('Failed to save API key:', err));
        }, 1000);
        return () => clearTimeout(timer);
    }, [apiKey, isLoaded]);

    const handleCheckConnection = async () => {
        setStatusMsg(null);
        if (!apiKey) return;

        setLoading(true);
        try {
            // First save the key to be sure
            await SaveAssemblyAIAPIKey(apiKey);
            await CheckAssemblyAIConnection(apiKey);
            setStatusMsg({ type: 'success', text: t('api.assemblyaiSettings.connectionSuccess') });
            setTimeout(() => setStatusMsg(null), 3000);
        } catch (err: any) {
            setStatusMsg({ type: 'error', text: err || t('api.assemblyaiSettings.connectionError') });
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="content-wrapper animate-fade">
            <div className="settings-container" style={{ maxWidth: '1000px' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '30px' }}>
                    <h2 className="settings-title" style={{ margin: 0 }}>AssemblyAI</h2>
                </div>

                <div className="settings-section glass-panel" style={{ padding: '25px', borderRadius: '12px', background: 'rgba(255, 255, 255, 0.03)', border: '1px solid rgba(255, 255, 255, 0.05)', marginBottom: '30px' }}>
                    <h3 className="section-title" style={{ marginBottom: '20px', fontSize: '1.1em', opacity: 0.9 }}>{t('api.assemblyaiSettings.apikey')}</h3>
                    <div style={{ display: 'flex', gap: '12px' }}>
                        <input
                            type="password"
                            className="premium-input"
                            style={{
                                flex: 1,
                                padding: '12px 16px',
                                borderRadius: '8px',
                                border: '1px solid rgba(255, 255, 255, 0.08)',
                                background: 'rgba(0, 0, 0, 0.3)',
                                color: '#fff',
                                outline: 'none',
                                fontSize: '0.95em'
                            }}
                            value={apiKey}
                            onChange={(e) => {
                                setApiKey(e.target.value);
                                setStatusMsg(null);
                            }}
                            placeholder="Enter your AssemblyAI API Key..."
                        />
                        <button
                            onClick={handleCheckConnection}
                            disabled={loading || !apiKey}
                            style={{
                                padding: '12px 24px',
                                borderRadius: '8px',
                                background: accentColor,
                                border: 'none',
                                color: '#fff',
                                cursor: 'pointer',
                                fontWeight: '600',
                                display: 'flex',
                                alignItems: 'center',
                                gap: '8px',
                                transition: 'all 0.2s ease',
                                opacity: (loading || !apiKey) ? 0.5 : 1,
                                boxShadow: `0 4px 15px ${accentColor}33`
                            }}
                        >
                            {loading ? <div className="spinner-small" /> : <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path><polyline points="22 4 12 14.01 9 11.01"></polyline></svg>}
                            {t('api.assemblyaiSettings.checkConnection')}
                        </button>
                    </div>
                    {statusMsg && (
                        <div style={{ marginTop: '10px', color: statusMsg.type === 'success' ? '#4caf50' : '#ff5252', fontSize: '0.85em', textAlign: 'right', fontWeight: '500' }}>
                            {statusMsg.text}
                        </div>
                    )}
                </div>

                <div className="stat-group glass-panel" style={{ padding: '25px', borderRadius: '12px', background: 'rgba(255, 255, 255, 0.02)', border: '1px solid rgba(255, 255, 255, 0.05)' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '15px' }}>
                        <div style={{ width: '50px', height: '50px', borderRadius: '10px', background: 'rgba(255,255,255,0.05)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke={accentColor} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12 20h9"></path><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"></path></svg>
                        </div>
                        <div>
                            <div style={{ opacity: 0.5, fontSize: '0.8em', textTransform: 'uppercase' }}>{t('api.assemblyaiSettings.serviceTitle')}</div>
                            <div style={{ fontWeight: '600', color: apiKey ? '#4caf50' : '#ff5252' }}>
                                {apiKey ? t('api.assemblyaiSettings.configured') : t('api.assemblyaiSettings.notConfigured')}
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            <style>{`
                @keyframes spin { to { transform: rotate(360deg); } }
                .spinner-small { width: 16px; height: 16px; border: 2px solid rgba(255,255,255,0.3); border-top-color: #fff; border-radius: 50%; animation: spin 0.8s linear infinite; }
            `}</style>
        </div>
    );
};
