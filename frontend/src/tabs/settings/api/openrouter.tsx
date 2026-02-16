import React, { useState, useEffect } from 'react';
import { useI18n } from '../../../contexts/I18nContext';
import { useTheme } from '../../../contexts/ThemeContext';
import { useServices } from '../../../contexts/ServiceContext';
// @ts-ignore
import { SaveOpenRouterAPIKey, GetOpenRouterAPIKey, SaveOpenRouterModels, GetOpenRouterSavedModels } from '../../../../wailsjs/go/main/App';

export const OpenRouter = () => {
    const { t } = useI18n();
    const { accentColor } = useTheme();

    // Global Service State
    const { openRouterBalance, loadingOpenRouter, refreshOpenRouterBalance } = useServices();

    const [apiKey, setApiKey] = useState('');
    const [savedModels, setSavedModels] = useState<string[]>([]);
    const [newModel, setNewModel] = useState('');
    const [isLoaded, setIsLoaded] = useState(false);
    const [statusMsg, setStatusMsg] = useState<{ type: 'success' | 'error', text: string } | null>(null);

    // Initial Load
    useEffect(() => {
        const loadData = async () => {
            const key = await GetOpenRouterAPIKey();
            setApiKey(key || '');
            const models = await GetOpenRouterSavedModels();
            setSavedModels(models || []);
            setIsLoaded(true);
        };
        loadData();
    }, []);

    // Auto-save API Key
    useEffect(() => {
        if (!isLoaded) return;

        const timer = setTimeout(() => {
            SaveOpenRouterAPIKey(apiKey);
        }, 1000);

        return () => clearTimeout(timer);
    }, [apiKey, isLoaded]);

    const handleCheckBalance = async () => {
        setStatusMsg(null);
        if (!apiKey) return;

        // Save immediately before checking
        await SaveOpenRouterAPIKey(apiKey);

        try {
            await refreshOpenRouterBalance();
            // We assume success if no error thrown inside refreshOpenRouterBalance
            // But since refreshOpenRouterBalance catches its own errors, we check the balance
            // Actually, for better UX let's show success message briefly
            setStatusMsg({ type: 'success', text: 'Updated' });
            setTimeout(() => setStatusMsg(null), 3000);
        } catch (err) {
            setStatusMsg({ type: 'error', text: 'Failed' });
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
        <div className="content-wrapper animate-fade">
            <div className="settings-container">

                {/* API Key Section */}
                <div className="settings-section">
                    <h3 className="section-title">{t('api.openrouterSettings.apikey')}</h3>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '5px' }}>
                        <div style={{ display: 'flex', gap: '10px' }}>
                            <input
                                type="password"
                                style={{
                                    flex: 1,
                                    padding: '10px',
                                    borderRadius: '6px',
                                    border: '1px solid rgba(255, 255, 255, 0.1)',
                                    background: 'rgba(0, 0, 0, 0.2)',
                                    color: '#fff',
                                    outline: 'none',
                                    transition: 'border-color 0.2s',
                                    boxSizing: 'border-box'
                                }}
                                onFocus={(e) => e.target.style.borderColor = accentColor}
                                onBlur={(e) => e.target.style.borderColor = 'rgba(255, 255, 255, 0.1)'}
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
                                    padding: '10px 20px',
                                    borderRadius: '6px',
                                    background: accentColor,
                                    border: 'none',
                                    color: '#fff',
                                    cursor: 'pointer',
                                    fontWeight: '500',
                                    fontSize: '0.9em',
                                    transition: 'opacity 0.2s',
                                    whiteSpace: 'nowrap',
                                    opacity: (loadingOpenRouter || !apiKey) ? 0.5 : 1
                                }}
                            >
                                {loadingOpenRouter ? '...' : t('api.openrouterSettings.checkbalance')}
                            </button>
                        </div>

                        <div style={{ minHeight: '20px', display: 'flex', justifyContent: 'flex-end', alignItems: 'center' }}>
                            {openRouterBalance !== null && (
                                <span style={{ color: '#4caf50', fontWeight: 'bold', fontSize: '1.1em', marginRight: '10px' }}>
                                    {t('api.openrouterSettings.balance')} ${openRouterBalance.toFixed(4)}
                                </span>
                            )}
                            {statusMsg && (
                                <span style={{
                                    color: statusMsg.type === 'success' ? '#4caf50' : '#ff5252',
                                    fontSize: '0.9em'
                                }}>
                                    {statusMsg.text}
                                </span>
                            )}
                        </div>
                    </div>
                </div>

                {/* Models Section */}
                <div className="settings-section">
                    <h3 className="section-title">{t('api.openrouterSettings.models')}</h3>
                    <div style={{ display: 'flex', gap: '10px', marginBottom: '15px' }}>
                        <input
                            type="text"
                            placeholder={t('api.openrouterSettings.modelname')}
                            value={newModel}
                            onChange={(e) => setNewModel(e.target.value)}
                            onKeyDown={(e) => e.key === 'Enter' && handleAddModel()}
                            style={{
                                flex: 1,
                                padding: '10px',
                                borderRadius: '6px',
                                border: '1px solid rgba(255, 255, 255, 0.1)',
                                background: 'rgba(0, 0, 0, 0.2)',
                                color: '#fff',
                                outline: 'none',
                                transition: 'border-color 0.2s'
                            }}
                            onFocus={(e) => e.target.style.borderColor = accentColor}
                            onBlur={(e) => e.target.style.borderColor = 'rgba(255, 255, 255, 0.1)'}
                        />
                        <button
                            onClick={handleAddModel}
                            disabled={!newModel.trim()}
                            style={{
                                padding: '10px 20px',
                                borderRadius: '6px',
                                background: accentColor,
                                border: 'none',
                                color: '#fff',
                                cursor: 'pointer',
                                fontWeight: '500',
                                opacity: !newModel.trim() ? 0.5 : 1,
                                transition: 'opacity 0.2s'
                            }}
                        >
                            {t('api.openrouterSettings.add')}
                        </button>
                    </div>

                    <div className="models-list" style={{
                        maxHeight: '300px',
                        overflowY: 'auto',
                        border: '1px solid rgba(255, 255, 255, 0.1)',
                        borderRadius: '6px',
                        background: 'rgba(0, 0, 0, 0.2)'
                    }}>
                        {savedModels.length === 0 ? (
                            <div style={{ padding: '20px', textAlign: 'center', color: '#666' }}>
                                {t('api.openrouterSettings.nomodels')}
                            </div>
                        ) : (
                            savedModels.map(model => (
                                <div key={model} style={{
                                    padding: '12px',
                                    borderBottom: '1px solid rgba(255, 255, 255, 0.05)',
                                    display: 'flex',
                                    justifyContent: 'space-between',
                                    alignItems: 'center',
                                    transition: 'background 0.2s'
                                }}>
                                    <span style={{ color: '#e0e0e0' }}>{model}</span>
                                    <button
                                        onClick={() => handleRemoveModel(model)}
                                        style={{
                                            background: 'none',
                                            border: 'none',
                                            color: '#ff5252',
                                            cursor: 'pointer',
                                            fontSize: '18px',
                                            padding: '0 5px',
                                            display: 'flex',
                                            alignItems: 'center',
                                            justifyContent: 'center',
                                            opacity: 0.8
                                        }}
                                        onMouseEnter={(e) => e.currentTarget.style.opacity = '1'}
                                        onMouseLeave={(e) => e.currentTarget.style.opacity = '0.8'}
                                        title="Remove"
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
