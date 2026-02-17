import React, { useState, useEffect } from 'react';
import { useI18n } from '../../../../contexts/I18nContext';
import { useTheme } from '../../../../contexts/ThemeContext';
import { useServices } from '../../../../contexts/ServiceContext';
import { useLogger } from '../../../../contexts/LoggerContext';
// @ts-ignore
import { GetElevenLabsUnlimAPIKey, SaveElevenLabsUnlimAPIKey } from '../../../../../wailsjs/go/main/App';
import '../../general.css';

export const ElevenLabsUnlim = () => {
    const { t } = useI18n();
    const { accentColor } = useTheme();
    const { addLog } = useLogger();
    const { elevenLabsUnlimBalance, refreshElevenLabsUnlimBalance, loadingElevenLabsUnlim } = useServices();

    const [apiKey, setApiKey] = useState('');
    const [isLoaded, setIsLoaded] = useState(false);
    const [statusMsg, setStatusMsg] = useState<{ type: 'success' | 'error', text: string } | null>(null);

    // Initial Load
    useEffect(() => {
        const loadKey = async () => {
            const key = await GetElevenLabsUnlimAPIKey();
            setApiKey(key || '');
            setIsLoaded(true);
        };
        loadKey();
    }, []);

    // Auto-save API Key
    useEffect(() => {
        if (!isLoaded) return;

        const timer = setTimeout(() => {
            SaveElevenLabsUnlimAPIKey(apiKey);
        }, 1000);

        return () => clearTimeout(timer);
    }, [apiKey, isLoaded]);

    const handleCheckBalance = async () => {
        setStatusMsg(null);
        if (!apiKey) return;

        // Save immediately before checking
        await SaveElevenLabsUnlimAPIKey(apiKey);

        try {
            await refreshElevenLabsUnlimBalance();
            setStatusMsg({ type: 'success', text: 'Updated' });
            setTimeout(() => setStatusMsg(null), 3000);
        } catch (err) {
            setStatusMsg({ type: 'error', text: 'Failed' });
        }
    };

    return (
        <div className="content-wrapper animate-fade">
            <div className="settings-container">

                {/* API Key Section */}
                <div className="settings-section">
                    <h3 className="section-title">{t('settings.voice.apiKey')}</h3>
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
                                placeholder={t('settings.voice.apiKeyPlaceholder')}
                            />

                            <button
                                onClick={handleCheckBalance}
                                disabled={loadingElevenLabsUnlim || !apiKey}
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
                                    opacity: (loadingElevenLabsUnlim || !apiKey) ? 0.5 : 1
                                }}
                            >
                                {loadingElevenLabsUnlim ? '...' : t('settings.voice.fetchBalance')}
                            </button>
                        </div>

                        <div style={{ minHeight: '20px', display: 'flex', justifyContent: 'flex-end', alignItems: 'center' }}>
                            <span style={{ color: '#4caf50', fontWeight: 'bold', fontSize: '1.1em', marginRight: '10px' }}>
                                {t('settings.voice.balance')} {elevenLabsUnlimBalance !== null ? (elevenLabsUnlimBalance === -1 ? 'Unlimited' : elevenLabsUnlimBalance.toLocaleString()) : '---'}
                            </span>
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

            </div>
        </div>
    );
};
