import React, { useState, useEffect } from 'react';
import { useI18n } from '../../../contexts/I18nContext';
import { useTheme } from '../../../contexts/ThemeContext';
import { useServices } from '../../../contexts/ServiceContext';
// @ts-ignore
import { SaveOpenRouterAPIKey, GetOpenRouterAPIKey, SaveOpenRouterModels, GetOpenRouterSavedModels, GetOpenRouterKeys, SaveOpenRouterKeys, GetOpenRouterMaxConnections, SaveOpenRouterMaxConnections } from '../../../../wailsjs/go/main/App';
import '../general.css';

export const OpenRouter = () => {
    const { t } = useI18n();
    const { accentColor } = useTheme();
    const { openRouterBalances, loadingOpenRouter, refreshOpenRouterBalance, openRouterThreshold, setOpenRouterThreshold } = useServices();

    // @ts-ignore
    const { SaveOpenRouterAlertThreshold } = window.go.main.App;

    const [keys, setKeys] = useState<any[]>([]);
    const [newName, setNewName] = useState('');
    const [newKey, setNewKey] = useState('');
    const [threshold, setThreshold] = useState<string>('0');
    const [savedModels, setSavedModels] = useState<string[]>([]);
    const [newModel, setNewModel] = useState('');
    const [isLoaded, setIsLoaded] = useState(false);
    const [statusMsg, setStatusMsg] = useState<{ type: 'success' | 'error', text: string } | null>(null);
    const [maxConnections, setMaxConnections] = useState<number>(10);

    useEffect(() => {
        const loadKey = async () => {
            const orKeys = await GetOpenRouterKeys();
            setKeys(orKeys || []);
            setThreshold(openRouterThreshold.toString());
            const models = await GetOpenRouterSavedModels();
            setSavedModels(models || []);
            const max = await GetOpenRouterMaxConnections();
            setMaxConnections(max || 10);
            setIsLoaded(true);
        };
        loadKey();
    }, [openRouterThreshold]);

    useEffect(() => {
        if (!isLoaded) return;
        const timer = setTimeout(() => {
            SaveOpenRouterKeys(keys);
            const numThreshold = parseFloat(threshold) || 0;
            if (numThreshold !== openRouterThreshold) {
                SaveOpenRouterAlertThreshold(numThreshold);
                setOpenRouterThreshold(numThreshold);
            }
            SaveOpenRouterMaxConnections(maxConnections);
        }, 1000);
        return () => clearTimeout(timer);
    }, [keys, threshold, maxConnections, isLoaded]);

    const handleAddKey = () => {
        if (!newName.trim() || !newKey.trim()) return;
        const id = 'key_' + Date.now();
        const updatedKeys = [...keys, { id, name: newName.trim(), key: newKey.trim() }];
        setKeys(updatedKeys);
        setNewName('');
        setNewKey('');
    };

    const handleRemoveKey = (id: string) => {
        setKeys(keys.filter(k => k.id !== id));
    };

    const handleCheckBalance = async () => {
        setStatusMsg(null);
        if (keys.length === 0) return;
        await SaveOpenRouterKeys(keys);
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

    const totalBalance = Object.values(openRouterBalances).reduce((acc: number, b) => acc + (b || 0), 0);

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
                    {Object.keys(openRouterBalances).length > 0 && (
                        <div style={{
                            padding: '10px 20px',
                            borderRadius: '12px',
                            background: 'rgba(76, 175, 80, 0.1)',
                            border: '1px solid rgba(76, 175, 80, 0.2)',
                            display: 'flex',
                            flexDirection: 'column',
                            alignItems: 'flex-end'
                        }}>
                            <span style={{ fontSize: '0.75em', opacity: 0.6, textTransform: 'uppercase' }}>{t('api.openrouterSettings.totalBalance') || 'Total Balance'}</span>
                            <span style={{ fontSize: '1.4em', fontWeight: 'bold', color: '#4caf50' }}>${totalBalance.toFixed(4)}</span>
                        </div>
                    )}
                </div>

                <div className="settings-section glass-panel" style={{ padding: '25px', borderRadius: '12px', background: 'rgba(255, 255, 255, 0.03)', border: '1px solid rgba(255, 255, 255, 0.05)', marginBottom: '30px' }}>
                    <h3 className="section-title" style={{ marginBottom: '20px', fontSize: '1.1em', opacity: 0.9 }}>{t('api.openrouterSettings.apikey')}</h3>

                    {/* Add Key Form */}
                    <div style={{ display: 'flex', gap: '12px', marginBottom: '20px' }}>
                        <input
                            type="text"
                            className="premium-input"
                            style={{
                                flex: 0.4,
                                padding: '12px 16px',
                                borderRadius: '8px',
                                border: '1px solid rgba(255, 255, 255, 0.08)',
                                background: 'rgba(0, 0, 0, 0.3)',
                                color: '#fff',
                                outline: 'none',
                                fontSize: '0.95em'
                            }}
                            placeholder={t('api.openrouterSettings.keyNamePlaceholder') || "Назва ключа (напр. Основний)"}
                            value={newName}
                            onChange={(e) => setNewName(e.target.value)}
                        />
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
                            placeholder="sk-or-..."
                            value={newKey}
                            onChange={(e) => setNewKey(e.target.value)}
                        />
                        <button
                            onClick={handleAddKey}
                            disabled={!newName.trim() || !newKey.trim()}
                            style={{
                                padding: '12px 24px',
                                borderRadius: '8px',
                                background: accentColor,
                                border: 'none',
                                color: '#fff',
                                cursor: 'pointer',
                                fontWeight: '600',
                                opacity: (!newName.trim() || !newKey.trim()) ? 0.5 : 1
                            }}
                        >
                            {t('api.openrouterSettings.add')}
                        </button>
                    </div>

                    {/* Keys List */}
                    <div style={{
                        borderRadius: '8px',
                        border: '1px solid rgba(255, 255, 255, 0.05)',
                        background: 'rgba(0,0,0,0.2)',
                        marginBottom: '20px'
                    }}>
                        {keys.length === 0 ? (
                            <div style={{ padding: '20px', textAlign: 'center', opacity: 0.3 }}>
                                {t('api.openrouterSettings.noKeys') || "Немає доданих ключів"}
                            </div>
                        ) : (
                            keys.map((k) => (
                                <div key={k.id} style={{
                                    padding: '12px 20px',
                                    borderBottom: '1px solid rgba(255, 255, 255, 0.03)',
                                    display: 'flex',
                                    justifyContent: 'space-between',
                                    alignItems: 'center'
                                }}>
                                    <div style={{ display: 'flex', flexDirection: 'column' }}>
                                        <span style={{ fontSize: '0.95em', fontWeight: 'bold' }}>{k.name}</span>
                                        <span style={{ fontSize: '0.8em', opacity: 0.4 }}>{k.key.substring(0, 10)}...{k.key.substring(k.key.length - 4)}</span>
                                    </div>
                                    <div style={{ display: 'flex', alignItems: 'center', gap: '15px' }}>
                                        <div style={{
                                            color: (typeof openRouterBalances[k.id] === 'number' && openRouterThreshold > 0 && openRouterBalances[k.id]! < openRouterThreshold) ? '#ff5252' : '#4caf50',
                                            fontWeight: '600',
                                            fontSize: '0.9em'
                                        }}>
                                            {typeof openRouterBalances[k.id] === 'number' ? `$${openRouterBalances[k.id]!.toFixed(4)}` : '...'}
                                        </div>
                                        <button
                                            onClick={() => handleRemoveKey(k.id)}
                                            style={{ background: 'none', border: 'none', color: '#ff5252', cursor: 'pointer', opacity: 0.6, fontSize: '1.2em' }}
                                        >
                                            &times;
                                        </button>
                                    </div>
                                </div>
                            ))
                        )}
                    </div>

                    <div style={{ display: 'flex', justifyContent: 'flex-end' }}>
                        <button
                            onClick={handleCheckBalance}
                            disabled={loadingOpenRouter || keys.length === 0}
                            style={{
                                padding: '10px 20px',
                                borderRadius: '8px',
                                background: 'rgba(255, 255, 255, 0.05)',
                                border: '1px solid rgba(255, 255, 255, 0.1)',
                                color: '#fff',
                                cursor: 'pointer',
                                fontWeight: '500',
                                display: 'flex',
                                alignItems: 'center',
                                gap: '8px',
                                opacity: (loadingOpenRouter || keys.length === 0) ? 0.5 : 1
                            }}
                        >
                            {loadingOpenRouter ? <div className="spinner-small" /> : <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><path d="M21 2v6h-6"></path><path d="M3 12a9 9 0 0 1 15-6.7L21 8"></path><path d="M3 22v-6h6"></path><path d="M21 12a9 9 0 0 1-15 6.7L3 16"></path></svg>}
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

                <div className="settings-section glass-panel" style={{ padding: '25px', borderRadius: '12px', background: 'rgba(255, 255, 255, 0.03)', border: '1px solid rgba(255, 255, 255, 0.05)', marginBottom: '30px' }}>
                    <h3 className="section-title" style={{ marginBottom: '10px', fontSize: '1.1em', opacity: 0.9 }}>{t('pipeline.openrouter.max_connections') || 'Кількість одночасних з\'єднань'}</h3>
                    <p style={{ fontSize: '0.85em', opacity: 0.5, marginBottom: '20px' }}>
                        {t('pipeline.openrouter.max_connections_desc') || 'Ліміт одночасних запитів до OpenRouter для всієї програми. Нові запити чекатимуть у черзі.'}
                    </p>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '15px' }}>
                        <input
                            type="range"
                            min="1"
                            max="50"
                            step="1"
                            style={{
                                flex: 1,
                                height: '6px',
                                borderRadius: '3px',
                                background: 'rgba(255, 255, 255, 0.1)',
                                appearance: 'none',
                                outline: 'none',
                                cursor: 'pointer'
                            }}
                            value={maxConnections}
                            onChange={(e) => setMaxConnections(parseInt(e.target.value))}
                        />
                        <div style={{
                            minWidth: '45px',
                            padding: '8px 12px',
                            background: 'rgba(0,0,0,0.3)',
                            borderRadius: '6px',
                            border: `1px solid ${accentColor}`,
                            textAlign: 'center',
                            fontWeight: 'bold',
                            color: accentColor
                        }}>
                            {maxConnections}
                        </div>
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
