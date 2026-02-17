import React, { useState, useEffect } from 'react';
import { useI18n } from '../../../../contexts/I18nContext';
import { useTheme } from '../../../../contexts/ThemeContext';
import { useServices } from '../../../../contexts/ServiceContext';
// @ts-ignore
import { GetVoiceMakerAPIKey, SaveVoiceMakerAPIKey } from '../../../../../wailsjs/go/main/App';
import '../../general.css';

export const VoiceMaker = () => {
    const { t } = useI18n();
    const { accentColor } = useTheme();
    const { voiceMakerBalance, refreshVoiceMakerBalance, loadingVoiceMaker, voiceMakerThreshold, setVoiceMakerThreshold } = useServices();

    // @ts-ignore
    const { SaveVoiceMakerAlertThreshold } = window.go.main.App;

    const [apiKey, setApiKey] = useState('');
    const [threshold, setThreshold] = useState<string>('0');
    const [isLoaded, setIsLoaded] = useState(false);
    const [statusMsg, setStatusMsg] = useState<{ type: 'success' | 'error', text: string } | null>(null);

    useEffect(() => {
        const loadKey = async () => {
            const key = await GetVoiceMakerAPIKey();
            setApiKey(key || '');
            setThreshold(voiceMakerThreshold.toString());
            setIsLoaded(true);
        };
        loadKey();
    }, [voiceMakerThreshold]);

    useEffect(() => {
        if (!isLoaded) return;
        const timer = setTimeout(() => {
            SaveVoiceMakerAPIKey(apiKey);
            const numThreshold = parseFloat(threshold) || 0;
            if (numThreshold !== voiceMakerThreshold) {
                SaveVoiceMakerAlertThreshold(numThreshold);
                setVoiceMakerThreshold(numThreshold);
            }
        }, 1000);
        return () => clearTimeout(timer);
    }, [apiKey, threshold, isLoaded, voiceMakerThreshold, setVoiceMakerThreshold]);

    const handleCheckBalance = async () => {
        setStatusMsg(null);
        if (!apiKey) return;
        await SaveVoiceMakerAPIKey(apiKey);
        try {
            await refreshVoiceMakerBalance();
            setStatusMsg({ type: 'success', text: t('image.success') || 'Updated' });
            setTimeout(() => setStatusMsg(null), 3000);
        } catch (err: any) {
            setStatusMsg({ type: 'error', text: err?.message || 'Error' });
        }
    };

    return (
        <div className="content-wrapper animate-fade">
            <div className="settings-container" style={{ maxWidth: '1000px' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '30px' }}>
                    <h2 className="settings-title" style={{ margin: 0 }}>VoiceMaker</h2>
                    {voiceMakerBalance !== null && (
                        <div style={{
                            padding: '10px 20px',
                            borderRadius: '12px',
                            background: 'rgba(76, 175, 80, 0.1)',
                            border: '1px solid rgba(76, 175, 80, 0.2)',
                            display: 'flex',
                            flexDirection: 'column',
                            alignItems: 'flex-end'
                        }}>
                            <span style={{ fontSize: '0.75em', opacity: 0.6, textTransform: 'uppercase' }}>Available Balance</span>
                            <span style={{ fontSize: '1.4em', fontWeight: 'bold', color: '#4caf50' }}>{voiceMakerBalance.toLocaleString()}</span>
                        </div>
                    )}
                </div>

                <div className="settings-section glass-panel" style={{ padding: '25px', borderRadius: '12px', background: 'rgba(255, 255, 255, 0.03)', border: '1px solid rgba(255, 255, 255, 0.05)', marginBottom: '30px' }}>
                    <h3 className="section-title" style={{ marginBottom: '20px', fontSize: '1.1em', opacity: 0.9 }}>{t('settings.voice.apiKey')}</h3>
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
                            placeholder={t('settings.voice.apiKeyPlaceholder')}
                        />
                        <button
                            onClick={handleCheckBalance}
                            disabled={loadingVoiceMaker || !apiKey}
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
                                opacity: (loadingVoiceMaker || !apiKey) ? 0.5 : 1,
                                boxShadow: `0 4px 15px ${accentColor}33`
                            }}
                        >
                            {loadingVoiceMaker ? <div className="spinner-small" /> : <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><path d="M21 2v6h-6"></path><path d="M3 12a9 9 0 0 1 15-6.7L21 8"></path><path d="M3 22v-6h6"></path><path d="M21 12a9 9 0 0 1-15 6.7L3 16"></path></svg>}
                            {t('settings.voice.fetchBalance')}
                        </button>
                    </div>
                    {statusMsg && (
                        <div style={{ marginTop: '10px', color: statusMsg.type === 'success' ? '#4caf50' : '#ff5252', fontSize: '0.85em', textAlign: 'right', fontWeight: '500' }}>
                            {statusMsg.text}
                        </div>
                    )}
                </div>

                <div className="settings-section glass-panel" style={{ padding: '25px', borderRadius: '12px', background: 'rgba(255, 255, 255, 0.03)', border: '1px solid rgba(255, 255, 255, 0.05)', marginBottom: '30px' }}>
                    <h3 className="section-title" style={{ marginBottom: '20px', fontSize: '1.1em', opacity: 0.9 }}>{t('settings.voice.alertThreshold')}</h3>
                    <div style={{ display: 'flex', gap: '12px' }}>
                        <input
                            type="number"
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
                            value={threshold}
                            onChange={(e) => setThreshold(e.target.value)}
                            placeholder={t('settings.voice.alertThresholdPlaceholder')}
                        />
                    </div>
                </div>

                <div className="stat-group glass-panel" style={{ padding: '25px', borderRadius: '12px', background: 'rgba(255, 255, 255, 0.02)', border: '1px solid rgba(255, 255, 255, 0.05)' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '15px' }}>
                        <div style={{ width: '50px', height: '50px', borderRadius: '10px', background: 'rgba(255,255,255,0.05)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke={accentColor} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"></path><path d="M19 10v1a7 7 0 0 1-14 0v-1"></path><line x1="12" y1="19" x2="12" y2="23"></line><line x1="8" y1="23" x2="16" y2="23"></line></svg>
                        </div>
                        <div>
                            <div style={{ opacity: 0.5, fontSize: '0.8em', textTransform: 'uppercase' }}>Voice Processing</div>
                            <div style={{ fontWeight: '600', color: voiceMakerBalance !== null ? '#4caf50' : '#ff5252' }}>
                                {voiceMakerBalance !== null ? 'Ready to Synthesize' : 'Configuration Required'}
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            <style>{`
                @keyframes spin { to { transform: rotate(360deg); } }
                .spinner-small { width: 16px; height: 16px; border: 2px solid rgba(255,255,255,0.3); border-top-color: #fff; borderRadius: 50%; animation: spin 0.8s linear infinite; }
            `}</style>
        </div>
    );
};
