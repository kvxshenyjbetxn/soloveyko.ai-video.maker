import React, { useState, useEffect } from 'react';
import { useI18n } from '../../../../contexts/I18nContext';
import { useTheme } from '../../../../contexts/ThemeContext';
import { useServices } from '../../../../contexts/ServiceContext';
// @ts-ignore
import { GetVoiceMakerKeys, SaveVoiceMakerKeys, SaveVoiceMakerAlertThreshold, GetVoiceMakerAlertThreshold } from '../../../../../wailsjs/go/main/App';
import '../../general.css';

export const VoiceMaker = () => {
    const { t } = useI18n();
    const { accentColor } = useTheme();
    const { voiceMakerBalances, voiceMakerKeys, refreshVoiceMakerBalance, loadingVoiceMaker, voiceMakerThreshold, setVoiceMakerThreshold } = useServices();

    const [keys, setKeys] = useState<any[]>([]);
    const [newName, setNewName] = useState('');
    const [newKey, setNewKey] = useState('');
    const [threshold, setThreshold] = useState<string>('0');
    const [isLoaded, setIsLoaded] = useState(false);
    const [statusMsg, setStatusMsg] = useState<{ type: 'success' | 'error', text: string } | null>(null);

    useEffect(() => {
        const loadSettings = async () => {
            const vmKeys = await GetVoiceMakerKeys();
            setKeys(vmKeys || []);
            setThreshold(voiceMakerThreshold.toString());
            setIsLoaded(true);
        };
        loadSettings();
    }, [voiceMakerThreshold]);

    useEffect(() => {
        if (!isLoaded) return;
        const timer = setTimeout(() => {
            SaveVoiceMakerKeys(keys);
            const numThreshold = parseFloat(threshold) || 0;
            if (numThreshold !== voiceMakerThreshold) {
                SaveVoiceMakerAlertThreshold(numThreshold);
                setVoiceMakerThreshold(numThreshold);
            }
        }, 1000);
        return () => clearTimeout(timer);
    }, [keys, threshold, isLoaded, voiceMakerThreshold, setVoiceMakerThreshold]);

    const handleAddKey = () => {
        if (!newName.trim() || !newKey.trim()) return;
        const id = 'vkey_' + Date.now();
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
        await SaveVoiceMakerKeys(keys);
        try {
            await refreshVoiceMakerBalance();
            setStatusMsg({ type: 'success', text: t('image.success') || 'Updated' });
            setTimeout(() => setStatusMsg(null), 3000);
        } catch (err: any) {
            setStatusMsg({ type: 'error', text: err?.message || 'Error' });
        }
    };

    const totalBalance = Object.values(voiceMakerBalances).reduce((acc: number, b) => acc + (b || 0), 0);

    return (
        <div className="content-wrapper animate-fade">
            <div className="settings-container" style={{ maxWidth: '1000px' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '30px' }}>
                    <h2 className="settings-title" style={{ margin: 0 }}>VoiceMaker</h2>
                    {Object.keys(voiceMakerBalances).length > 0 && (
                        <div style={{
                            padding: '10px 20px',
                            borderRadius: '12px',
                            background: 'rgba(76, 175, 80, 0.1)',
                            border: '1px solid rgba(76, 175, 80, 0.2)',
                            display: 'flex',
                            flexDirection: 'column',
                            alignItems: 'flex-end'
                        }}>
                            <span style={{ fontSize: '0.75em', opacity: 0.6, textTransform: 'uppercase' }}>Total Characters Available</span>
                            <span style={{ fontSize: '1.4em', fontWeight: 'bold', color: '#4caf50' }}>{totalBalance.toLocaleString()}</span>
                        </div>
                    )}
                </div>

                <div className="settings-section glass-panel" style={{ padding: '25px', borderRadius: '12px', background: 'rgba(255, 255, 255, 0.03)', border: '1px solid rgba(255, 255, 255, 0.05)', marginBottom: '30px' }}>
                    <h3 className="section-title" style={{ marginBottom: '20px', fontSize: '1.1em', opacity: 0.9 }}>{t('settings.voice.apiKey')}</h3>

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
                            placeholder={t('api.openrouterSettings.keyNamePlaceholder') || "Назва ключа"}
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
                            placeholder="sk-..."
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
                                No keys added
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
                                        <span style={{ fontSize: '0.8em', opacity: 0.4 }}>{k.key.substring(0, 10)}...</span>
                                    </div>
                                    <div style={{ display: 'flex', alignItems: 'center', gap: '15px' }}>
                                        <div style={{
                                            color: (typeof voiceMakerBalances[k.id] === 'number' && voiceMakerThreshold > 0 && voiceMakerBalances[k.id]! < voiceMakerThreshold) ? '#ff5252' : '#4caf50',
                                            fontWeight: '600',
                                            fontSize: '0.9em'
                                        }}>
                                            {typeof voiceMakerBalances[k.id] === 'number' ? `${voiceMakerBalances[k.id]!.toLocaleString()}` : '...'}
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
                            disabled={loadingVoiceMaker || keys.length === 0}
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
                                opacity: (loadingVoiceMaker || keys.length === 0) ? 0.5 : 1
                            }}
                        >
                            {loadingVoiceMaker ? <div className="spinner-small" /> : <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><path d="M21 2v6h-6"></path><path d="M3 12a9 9 0 0 1 15-6.7L21 8"></path><path d="M3 22v-6h6"></path><path d="M21 12a9 9 0 0 1-15 6.7L3 16"></path></svg>}
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
            </div>
            <style>{`
                @keyframes spin { to { transform: rotate(360deg); } }
                .spinner-small { width: 16px; height: 16px; border: 2px solid rgba(255,255,255,0.3); border-top-color: #fff; border-radius: 50%; animation: spin 0.8s linear infinite; }
            `}</style>
        </div>
    );
};
