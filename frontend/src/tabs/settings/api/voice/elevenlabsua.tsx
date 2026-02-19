import React, { useState, useEffect } from 'react';
import { useI18n } from '../../../../contexts/I18nContext';
import { useTheme } from '../../../../contexts/ThemeContext';
import '../../general.css';

export const ElevenLabsUA = () => {
    const { t } = useI18n();
    const { accentColor } = useTheme();

    // @ts-ignore
    const { GetElevenLabsUAKeys, SaveElevenLabsUAKeys } = window.go.main.App;

    const [keys, setKeys] = useState<any[]>([]);
    const [newName, setNewName] = useState('');
    const [newKey, setNewKey] = useState('');
    const [isLoaded, setIsLoaded] = useState(false);

    useEffect(() => {
        const loadKey = async () => {
            if (GetElevenLabsUAKeys) {
                const uaKeys = await GetElevenLabsUAKeys();
                setKeys(uaKeys || []);
            }
            setIsLoaded(true);
        };
        loadKey();
    }, []);

    useEffect(() => {
        if (!isLoaded) return;
        const timer = setTimeout(() => {
            if (SaveElevenLabsUAKeys) SaveElevenLabsUAKeys(keys);
        }, 1000);
        return () => clearTimeout(timer);
    }, [keys, isLoaded]);

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
                    <h2 className="settings-title" style={{ margin: 0 }}>ElevenLabs UA</h2>
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
                            placeholder="API Key"
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
                                        <span style={{ fontSize: '0.8em', opacity: 0.4 }}>{k.key.substring(0, 5)}...{k.key.substring(k.key.length - 4)}</span>
                                    </div>
                                    <div style={{ display: 'flex', alignItems: 'center', gap: '15px' }}>
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
