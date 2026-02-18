import React, { useState, useEffect } from 'react';
import { useI18n } from '../../../contexts/I18nContext';
import { useTheme } from '../../../contexts/ThemeContext';
import { useServices } from '../../../contexts/ServiceContext';
// @ts-ignore
import { SaveOpenRouterAPIKey, GetOpenRouterAPIKey, SaveOpenRouterModels, GetOpenRouterSavedModels } from '../../../../wailsjs/go/main/App';
import '../general.css';

export const OpenRouter = () => {
    const { t } = useI18n();
    const { accentColor } = useTheme();
    const { openRouterBalance, loadingOpenRouter, refreshOpenRouterBalance, openRouterThreshold, setOpenRouterThreshold } = useServices();

    // @ts-ignore
    const { SaveOpenRouterAlertThreshold } = window.go.main.App;

    const [apiKey, setApiKey] = useState('');
    const [threshold, setThreshold] = useState<string>('0');
    const [savedModels, setSavedModels] = useState<string[]>([]);
    const [newModel, setNewModel] = useState('');
    const [isLoaded, setIsLoaded] = useState(false);
    const [statusMsg, setStatusMsg] = useState<{ type: 'success' | 'error', text: string } | null>(null);

    useEffect(() => {
        const loadKey = async () => {
            const key = await GetOpenRouterAPIKey();
            setApiKey(key || '');
            setThreshold(openRouterThreshold.toString());
            const models = await GetOpenRouterSavedModels();
            setSavedModels(models || []);
            setIsLoaded(true);
        };
        loadKey();
    }, [openRouterThreshold]);

    useEffect(() => {
        if (!isLoaded) return;
        const timer = setTimeout(() => {
            SaveOpenRouterAPIKey(apiKey);
            const numThreshold = parseFloat(threshold) || 0;
            if (numThreshold !== openRouterThreshold) {
                SaveOpenRouterAlertThreshold(numThreshold);
                setOpenRouterThreshold(numThreshold);
            }
        }, 1000);
        return () => clearTimeout(timer);
    }, [apiKey, threshold, isLoaded]);

    const handleCheckBalance = async () => {
        setStatusMsg(null);
        if (!apiKey) return;
        await SaveOpenRouterAPIKey(apiKey);
        try {
            await refreshOpenRouterBalance();
            setStatusMsg({ type: 'success', text: t('image.success') || 'Updated' });
            setTimeout(() => setStatusMsg(null), 3000);
        } catch (err: any) {
            setStatusMsg({ type: 'error', text: err?.message || 'Error' });
        }
    };

    const handleAddModel = () => {
        if (!newModel.trim()) return;
        const modelName = newModel.trim();
        if (savedModels.includes(modelName)) {
            setNewModel('');
            return;
        }
        const updatedModels = [...savedModels, modelName];
        setSavedModels(updatedModels);
        SaveOpenRouterModels(updatedModels);
        setNewModel('');
    };

    const handleRemoveModel = (modelToRemove: string) => {
        const updatedModels = savedModels.filter(m => m !== modelToRemove);
        setSavedModels(updatedModels);
        SaveOpenRouterModels(updatedModels);
    };

    return (
        <div className="content-wrapper animate-fade" style={{
            height: '100%',
            overflowY: 'auto',
            overflowX: 'hidden',
            paddingRight: '10px' // Space for scrollbar
        }}>
            <div className="settings-container" style={{ maxWidth: '1000px', paddingBottom: '40px' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '30px' }}>
                    <h2 className="settings-title" style={{ margin: 0 }}>OpenRouter</h2>
                    {openRouterBalance !== null && (
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
                            <span style={{ fontSize: '1.4em', fontWeight: 'bold', color: '#4caf50' }}>${openRouterBalance.toFixed(4)}</span>
                        </div>
                    )}
                </div>

                <div className="settings-section glass-panel" style={{ padding: '25px', borderRadius: '12px', background: 'rgba(255, 255, 255, 0.03)', border: '1px solid rgba(255, 255, 255, 0.05)', marginBottom: '30px' }}>
                    <h3 className="section-title" style={{ marginBottom: '20px', fontSize: '1.1em', opacity: 0.9 }}>{t('api.openrouterSettings.apikey')}</h3>
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
                            placeholder="sk-or-..."
                        />
                        <button
                            onClick={handleCheckBalance}
                            disabled={loadingOpenRouter || !apiKey}
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
                                opacity: (loadingOpenRouter || !apiKey) ? 0.5 : 1,
                                boxShadow: `0 4px 15px ${accentColor}33`
                            }}
                        >
                            {loadingOpenRouter ? <div className="spinner-small" /> : <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><path d="M21 2v6h-6"></path><path d="M3 12a9 9 0 0 1 15-6.7L21 8"></path><path d="M3 22v-6h6"></path><path d="M21 12a9 9 0 0 1-15 6.7L3 16"></path></svg>}
                            {t('api.openrouterSettings.checkbalance')}
                        </button>
                    </div>
                    {statusMsg && (
                        <div style={{ marginTop: '10px', color: statusMsg.type === 'success' ? '#4caf50' : '#ff5252', fontSize: '0.85em', textAlign: 'right', fontWeight: '500' }}>
                            {statusMsg.text}
                        </div>
                    )}
                </div>

                <div className="settings-section glass-panel" style={{ padding: '25px', borderRadius: '12px', background: 'rgba(255, 255, 255, 0.03)', border: '1px solid rgba(255, 255, 255, 0.05)', marginBottom: '30px' }}>
                    <h3 className="section-title" style={{ marginBottom: '20px', fontSize: '1.1em', opacity: 0.9 }}>{t('api.openrouterSettings.alertThreshold')}</h3>
                    <div style={{ display: 'flex', gap: '12px' }}>
                        <input
                            type="number"
                            step="0.01"
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
                            placeholder={t('api.openrouterSettings.alertThresholdPlaceholder')}
                        />
                    </div>
                </div>

                <div className="settings-section glass-panel" style={{ padding: '25px', borderRadius: '12px', background: 'rgba(255, 255, 255, 0.03)', border: '1px solid rgba(255, 255, 255, 0.05)' }}>
                    <h3 className="section-title" style={{ marginBottom: '20px', fontSize: '1.1em', opacity: 0.9 }}>{t('api.openrouterSettings.models')}</h3>
                    <div style={{ display: 'flex', gap: '12px', marginBottom: '20px' }}>
                        <input
                            type="text"
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
                            placeholder={t('api.openrouterSettings.modelname')}
                            value={newModel}
                            onChange={(e) => setNewModel(e.target.value)}
                            onKeyDown={(e) => e.key === 'Enter' && handleAddModel()}
                        />
                        <button
                            onClick={handleAddModel}
                            disabled={!newModel.trim()}
                            style={{
                                padding: '12px 24px',
                                borderRadius: '8px',
                                background: 'rgba(255, 255, 255, 0.1)',
                                border: '1px solid rgba(255, 255, 255, 0.1)',
                                color: '#fff',
                                cursor: 'pointer',
                                fontWeight: '600',
                                opacity: !newModel.trim() ? 0.5 : 1
                            }}
                        >
                            {t('api.openrouterSettings.add')}
                        </button>
                    </div>

                    <div style={{
                        maxHeight: '400px',
                        overflowY: 'auto',
                        borderRadius: '8px',
                        border: '1px solid rgba(255, 255, 255, 0.05)',
                        background: 'rgba(0,0,0,0.2)'
                    }}>
                        {savedModels.length === 0 ? (
                            <div style={{ padding: '40px', textAlign: 'center', opacity: 0.3 }}>
                                {t('api.openrouterSettings.nomodels')}
                            </div>
                        ) : (
                            savedModels.map(model => (
                                <div key={model} style={{
                                    padding: '12px 20px',
                                    borderBottom: '1px solid rgba(255, 255, 255, 0.03)',
                                    display: 'flex',
                                    justifyContent: 'space-between',
                                    alignItems: 'center',
                                    transition: 'background 0.2s'
                                }} onMouseEnter={(e) => e.currentTarget.style.background = 'rgba(255,255,255,0.02)'} onMouseLeave={(e) => e.currentTarget.style.background = 'transparent'}>
                                    <span style={{ fontSize: '0.95em', opacity: 0.8 }}>{model}</span>
                                    <button
                                        onClick={() => handleRemoveModel(model)}
                                        style={{ background: 'none', border: 'none', color: '#ff5252', cursor: 'pointer', opacity: 0.6, fontSize: '1.2em' }}
                                    >
                                        &times;
                                    </button>
                                </div>
                            ))
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
};
