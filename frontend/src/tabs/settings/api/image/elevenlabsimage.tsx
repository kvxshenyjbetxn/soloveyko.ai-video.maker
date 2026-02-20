import React, { useState, useEffect } from 'react';
import { useI18n } from '../../../../contexts/I18nContext';
import { useTheme } from '../../../../contexts/ThemeContext';
// @ts-ignore
import { GetElevenLabsImageKeys, SaveElevenLabsImageKeys, GetElevenLabsImageMaxConnections, SaveElevenLabsImageMaxConnections } from '../../../../../wailsjs/go/main/App';
import '../../general.css';

export const ElevenLabsImage = () => {
    const { t } = useI18n();
    const { accentColor } = useTheme();

    const [keys, setKeys] = useState<any[]>([]);
    const [newName, setNewName] = useState('');
    const [newKey, setNewKey] = useState('');
    const [maxConnections, setMaxConnections] = useState<number>(25);
    const [isLoaded, setIsLoaded] = useState(false);
    const [statusMsg, setStatusMsg] = useState<{ type: 'success' | 'error', text: string } | null>(null);

    useEffect(() => {
        const loadSettings = async () => {
            try {
                const elKeys = await GetElevenLabsImageKeys();
                setKeys(elKeys || []);
                const max = await GetElevenLabsImageMaxConnections();
                setMaxConnections(max || 25);
            } catch (err) {
                console.error("Failed to load ElevenLabs Image settings:", err);
            } finally {
                setIsLoaded(true);
            }
        };
        loadSettings();
    }, []);

    useEffect(() => {
        if (!isLoaded) return;
        const timer = setTimeout(async () => {
            try {
                await SaveElevenLabsImageKeys(keys);
                await SaveElevenLabsImageMaxConnections(maxConnections);
                setStatusMsg({ type: 'success', text: 'Saved' });
                setTimeout(() => {
                    setStatusMsg(prev => prev?.text === 'Saved' ? null : prev);
                }, 2000);
            } catch (err) {
                setStatusMsg({ type: 'error', text: 'Error' });
            }
        }, 1000);
        return () => clearTimeout(timer);
    }, [keys, maxConnections, isLoaded]);

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

    return (
        <div className="content-wrapper animate-fade" style={{
            height: '100%',
            overflowY: 'auto',
            overflowX: 'hidden',
            paddingRight: '10px'
        }}>
            <div className="settings-container" style={{ maxWidth: '1000px', paddingBottom: '40px' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '30px' }}>
                    <h2 className="settings-title" style={{ margin: 0 }}>ElevenLabs Image</h2>
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
                            placeholder={t('api.openrouterSettings.keyNamePlaceholder')}
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
                            placeholder="X-API-Key..."
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
                                {t('api.openrouterSettings.noKeys')}
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
                                    <button
                                        onClick={() => handleRemoveKey(k.id)}
                                        style={{ background: 'none', border: 'none', color: '#ff5252', cursor: 'pointer', opacity: 0.6, fontSize: '1.2em' }}
                                    >
                                        &times;
                                    </button>
                                </div>
                            ))
                        )}
                    </div>

                    {statusMsg && (
                        <div style={{ marginTop: '10px', color: statusMsg.type === 'success' ? '#4caf50' : '#ff5252', fontSize: '0.85em', textAlign: 'right', fontWeight: '500' }}>
                            {statusMsg.text}
                        </div>
                    )}
                </div>

                <div className="settings-section glass-panel" style={{ padding: '25px', borderRadius: '12px', background: 'rgba(255, 255, 255, 0.03)', border: '1px solid rgba(255, 255, 255, 0.05)', marginBottom: '30px' }}>
                    <h3 className="section-title" style={{ marginBottom: '10px', fontSize: '1.1em', opacity: 0.9 }}>{t('api.elevenlabsimageSettings.threads')}</h3>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '15px' }}>
                        <input
                            type="range"
                            min="1"
                            max="25"
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

                <div className="stat-group glass-panel" style={{ padding: '25px', borderRadius: '12px', background: 'rgba(255, 255, 255, 0.02)', border: '1px solid rgba(255, 255, 255, 0.05)' }}>
                    <div style={{ display: 'flex', alignItems: 'flex-start', gap: '15px' }}>
                        <div style={{ width: '50px', height: '50px', borderRadius: '10px', background: 'rgba(255,255,255,0.05)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke={accentColor} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><circle cx="8.5" cy="8.5" r="1.5"></circle><polyline points="21 15 16 10 5 21"></polyline></svg>
                        </div>
                        <div>
                            <div style={{ opacity: 0.5, fontSize: '0.8em', textTransform: 'uppercase' }}>Available Methods</div>
                            <div style={{ marginTop: '5px', display: 'flex', flexWrap: 'wrap', gap: '8px' }}>
                                <span style={{ padding: '4px 10px', borderRadius: '4px', background: 'rgba(255,255,255,0.05)', fontSize: '0.85em' }}>Create</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
};
