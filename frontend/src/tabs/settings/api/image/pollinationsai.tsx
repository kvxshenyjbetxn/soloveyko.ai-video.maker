import React, { useState, useEffect } from 'react';
import { useI18n } from '../../../../contexts/I18nContext';
import { useTheme } from '../../../../contexts/ThemeContext';
import { useServices } from '../../../../contexts/ServiceContext';
// @ts-ignore
import { SavePollinationsAPIKey, GetPollinationsAPIKey, SavePollinationsModels, GetPollinationsSavedModels, GetPollinationsImageModels } from '../../../../../wailsjs/go/main/App';

export const PollinationsAI = () => {
    const { t } = useI18n();
    const { accentColor } = useTheme();
    const { refreshPollinationsKeys } = useServices();

    const [keys, setKeys] = useState<any[]>([]);
    const [newName, setNewName] = useState('');
    const [newKey, setNewKey] = useState('');
    const [savedModels, setSavedModels] = useState<string[]>([]);
    const [availableModels, setAvailableModels] = useState<string[]>([]);
    const [loadingModels, setLoadingModels] = useState(false);
    const [isLoaded, setIsLoaded] = useState(false);
    const [statusMsg, setStatusMsg] = useState<{ type: 'success' | 'error', text: string } | null>(null);

    // Initial Load
    useEffect(() => {
        const loadData = async () => {
            // @ts-ignore
            const { GetPollinationsKeys } = window.go.main.App;
            const pKeys = await GetPollinationsKeys();
            setKeys(pKeys || []);
            const models = await GetPollinationsSavedModels();
            setSavedModels(models || []);
            setIsLoaded(true);
        };
        loadData();
    }, []);

    // Auto-save API Keys
    useEffect(() => {
        if (!isLoaded) return;
        const timer = setTimeout(async () => {
            // @ts-ignore
            const { SavePollinationsKeys } = window.go.main.App;
            await SavePollinationsKeys(keys);
            refreshPollinationsKeys();
        }, 1000);
        return () => clearTimeout(timer);
    }, [keys, isLoaded]);

    const handleAddKey = () => {
        if (!newName.trim()) return;
        const id = 'key_' + Date.now();
        const updatedKeys = [...keys, { id, name: newName.trim(), key: newKey.trim() }];
        setKeys(updatedKeys);
        setNewName('');
        setNewKey('');
    };

    const handleRemoveKey = (id: string) => {
        setKeys(keys.filter(k => k.id !== id));
    };

    const handleFetchModels = async () => {
        setLoadingModels(true);
        setStatusMsg(null);
        try {
            // @ts-ignore
            const { SavePollinationsKeys } = window.go.main.App;
            await SavePollinationsKeys(keys);
            refreshPollinationsKeys();
            const models = await GetPollinationsImageModels();
            setAvailableModels(models || []);
        } catch (err) {
            console.error(err);
            setStatusMsg({ type: 'error', text: 'Failed to fetch models' });
        } finally {
            setLoadingModels(false);
        }
    };

    const handleAddModel = (modelName: string) => {
        if (savedModels.includes(modelName)) return;
        const updated = [...savedModels, modelName];
        setSavedModels(updated);
        SavePollinationsModels(updated);
    };

    const handleRemoveModel = (modelName: string) => {
        const updated = savedModels.filter(m => m !== modelName);
        setSavedModels(updated);
        SavePollinationsModels(updated);
    };

    return (
        <div className="content-wrapper animate-fade" style={{ overflowY: 'auto' }}>
            <div className="settings-container">

                {/* API Key */}
                <div className="settings-section">
                    <h3 className="section-title">{t('api.pollinationsSettings.apikey')}</h3>

                    {/* Add Key Form */}
                    <div style={{ display: 'flex', gap: '10px', marginBottom: '15px' }}>
                        <input
                            type="text"
                            style={{
                                flex: 0.4,
                                padding: '10px',
                                borderRadius: '6px',
                                border: '1px solid rgba(255, 255, 255, 0.1)',
                                background: 'rgba(0, 0, 0, 0.2)',
                                color: '#fff',
                                outline: 'none'
                            }}
                            placeholder={t('api.openrouterSettings.keyNamePlaceholder') || "Назва"}
                            value={newName}
                            onChange={(e) => setNewName(e.target.value)}
                        />
                        <input
                            type="password"
                            style={{
                                flex: 1,
                                padding: '10px',
                                borderRadius: '6px',
                                border: '1px solid rgba(255, 255, 255, 0.1)',
                                background: 'rgba(0, 0, 0, 0.2)',
                                color: '#fff',
                                outline: 'none'
                            }}
                            placeholder="API Key (можна пустим)"
                            value={newKey}
                            onChange={(e) => setNewKey(e.target.value)}
                        />
                        <button
                            onClick={handleAddKey}
                            style={{
                                padding: '10px 15px',
                                borderRadius: '6px',
                                background: accentColor,
                                border: 'none',
                                color: '#fff',
                                cursor: 'pointer',
                                fontWeight: 'bold'
                            }}
                        >
                            {t('common.add')}
                        </button>
                    </div>

                    {/* Keys List */}
                    <div style={{
                        borderRadius: '6px',
                        border: '1px solid rgba(255, 255, 255, 0.1)',
                        background: 'rgba(0,0,0,0.1)',
                        overflow: 'hidden'
                    }}>
                        {keys.length === 0 ? (
                            <div style={{ padding: '15px', textAlign: 'center', opacity: 0.5, fontSize: '0.9em' }}>
                                {t('api.openrouterSettings.noKeys')}
                            </div>
                        ) : (
                            keys.map(k => (
                                <div key={k.id} style={{
                                    padding: '10px 15px',
                                    borderBottom: '1px solid rgba(255, 255, 255, 0.05)',
                                    display: 'flex',
                                    justifyContent: 'space-between',
                                    alignItems: 'center'
                                }}>
                                    <div>
                                        <div style={{ fontWeight: 'bold', fontSize: '0.9em' }}>{k.name}</div>
                                        {k.key && <div style={{ fontSize: '0.8em', opacity: 0.5 }}>{k.key.substring(0, 8)}...</div>}
                                    </div>
                                    <button
                                        onClick={() => handleRemoveKey(k.id)}
                                        style={{ background: 'none', border: 'none', color: '#ff5252', cursor: 'pointer', fontSize: '1.2em' }}
                                    >
                                        &times;
                                    </button>
                                </div>
                            ))
                        )}
                    </div>
                </div>

                {/* Models Section */}
                <div className="settings-section">
                    <h3 className="section-title">{t('api.pollinationsSettings.models')}</h3>

                    <button
                        onClick={handleFetchModels}
                        disabled={loadingModels}
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
                            opacity: loadingModels ? 0.7 : 1,
                            marginBottom: '15px'
                        }}
                    >
                        {loadingModels ? '...' : t('api.pollinationsSettings.fetchModels')}
                    </button>

                    {statusMsg && (
                        <div style={{ color: '#ff5252', marginBottom: '10px', fontSize: '0.9em' }}>
                            {statusMsg.text}
                        </div>
                    )}

                    {/* Available Models List */}
                    {availableModels.length > 0 && (
                        <div style={{
                            marginBottom: '20px',
                            border: '1px solid rgba(255, 255, 255, 0.1)',
                            borderRadius: '6px',
                            background: 'rgba(0, 0, 0, 0.2)',
                            overflow: 'hidden'
                        }}>
                            <div style={{ padding: '10px', background: 'rgba(255, 255, 255, 0.05)', fontWeight: 'bold', fontSize: '0.9em' }}>
                                {t('api.pollinationsSettings.availableModels')}
                            </div>
                            <div style={{ maxHeight: '200px', overflowY: 'auto' }}>
                                {availableModels.map(model => (
                                    <div key={model} style={{
                                        padding: '10px',
                                        borderBottom: '1px solid rgba(255, 255, 255, 0.05)',
                                        display: 'flex',
                                        justifyContent: 'space-between',
                                        alignItems: 'center'
                                    }}>
                                        <span style={{ color: '#e0e0e0' }}>{model}</span>
                                        <button
                                            onClick={() => handleAddModel(model)}
                                            disabled={savedModels.includes(model)}
                                            style={{
                                                background: savedModels.includes(model) ? 'rgba(76, 175, 80, 0.2)' : 'rgba(255, 255, 255, 0.1)',
                                                color: savedModels.includes(model) ? '#4caf50' : '#fff',
                                                border: 'none',
                                                borderRadius: '4px',
                                                padding: '4px 8px',
                                                cursor: savedModels.includes(model) ? 'default' : 'pointer',
                                                fontSize: '0.8em'
                                            }}
                                        >
                                            {savedModels.includes(model) ? 'Added' : t('api.pollinationsSettings.add')}
                                        </button>
                                    </div>
                                ))}
                            </div>
                        </div>
                    )}

                    {/* Saved Models List */}
                    <div className="models-list" style={{
                        maxHeight: '300px',
                        overflowY: 'auto',
                        border: '1px solid rgba(255, 255, 255, 0.1)',
                        borderRadius: '6px',
                        background: 'rgba(0, 0, 0, 0.2)'
                    }}>
                        <div style={{ padding: '10px', background: 'rgba(255, 255, 255, 0.05)', fontWeight: 'bold', fontSize: '0.9em' }}>
                            {t('api.pollinationsSettings.savedModels')}
                        </div>
                        {savedModels.length === 0 ? (
                            <div style={{ padding: '20px', textAlign: 'center', color: '#666' }}>
                                {t('api.pollinationsSettings.nomodels')}
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
