import React, { useState } from 'react';
import { useI18n } from '../contexts/I18nContext';
import { useTheme } from '../contexts/ThemeContext';
import './AuthWindow.css';
import { api } from '../../wailsjs/go/models';

interface AuthWindowProps {
    onAuthenticated: (response: api.AuthResponse, key: string, saved: boolean) => void;
    error?: string;
}

const KeyIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4"></path>
    </svg>
);

export const AuthWindow: React.FC<AuthWindowProps> = ({ onAuthenticated, error: defaultError }) => {
    const { t } = useI18n();
    const { accentColor } = useTheme();
    const [key, setKey] = useState('');
    const [saveKey, setSaveKey] = useState(false);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | undefined>(defaultError);

    const formatError = (errMsg: string) => {
        if (!errMsg) return t('auth.invalid_key');

        const lowerMsg = errMsg.toLowerCase();

        if (lowerMsg.includes("subscription expired") || lowerMsg.includes("expired") || lowerMsg.includes("invalid or expired")) return t('auth.error_expired');
        if (lowerMsg.includes("hardware mismatch")) return t('auth.error_hardware_mismatch');
        if (lowerMsg.includes("invalid api key") || lowerMsg.includes("invalid api")) return t('auth.error_invalid');

        return errMsg;
    };

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!key.trim()) return;

        setLoading(true);
        setError(undefined);

        try {
            // @ts-ignore
            const response = await window.go.main.App.ValidateKey(key.trim());
            console.log("Auth response:", response);
            if (response && response.valid) {
                onAuthenticated(response, key.trim(), saveKey);
            } else {
                // If the server returned an object but valid is false, it might have an error message
                // However, our api.AuthResponse doesn't have an error field.
                // Usually server throws 403 which goes to catch.
                setError(t('auth.error_invalid'));
            }
        } catch (err: any) {
            console.error("Auth error catch:", err);
            setError(formatError(err));
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="auth-overlay animate-fade">
            <div className="auth-modal glass-panel">
                <div className="auth-header">
                    <div className="auth-icon-wrapper" style={{ backgroundColor: `${accentColor}20`, color: accentColor }}>
                        <KeyIcon />
                    </div>
                    <h2>{t('auth.title')}</h2>
                    <p className="auth-subtitle">{t('auth.subtitle') || 'Enter your access key to continue'}</p>
                </div>

                <form onSubmit={handleSubmit} className="auth-form">
                    <div className="form-group">
                        <input
                            type="password"
                            className="premium-input auth-input"
                            value={key}
                            onChange={e => {
                                setKey(e.target.value);
                                setError(undefined);
                            }}
                            placeholder={t('auth.key_placeholder')}
                            disabled={loading}
                            autoFocus
                        />
                    </div>

                    {error && (
                        <div className="auth-error animate-fade">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10"></circle><line x1="12" y1="8" x2="12" y2="12"></line><line x1="12" y1="16" x2="12.01" y2="16"></line></svg>
                            <span>{error}</span>
                        </div>
                    )}

                    <div className="auth-options">
                        <label className="toggle-switch">
                            <input
                                type="checkbox"
                                checked={saveKey}
                                onChange={e => setSaveKey(e.target.checked)}
                                disabled={loading}
                            />
                            <span className="toggle-slider" style={saveKey ? { backgroundColor: accentColor } : {}}></span>
                        </label>
                        <span className="toggle-label" onClick={() => !loading && setSaveKey(!saveKey)}>
                            {t('auth.save_key')}
                        </span>
                    </div>

                    <button
                        type="submit"
                        className="auth-submit-btn"
                        disabled={loading || !key.trim()}
                        style={{ backgroundColor: accentColor }}
                    >
                        {loading ? (
                            <div className="auth-spinner">
                                <div className="spinner-small"></div>
                                <span>{t('auth.checking')}</span>
                            </div>
                        ) : (
                            t('auth.login')
                        )}
                    </button>
                </form>
            </div>
        </div>
    );
};
