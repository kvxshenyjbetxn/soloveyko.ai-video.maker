import React, { useState } from 'react';
import './WelcomeWindow.css';
import { useI18n } from '../contexts/I18nContext';
// @ts-ignore
import logo from '../assets/logo.png';

interface WelcomeWindowProps {
    onFinish: () => void;
    onReconfigure: () => void;
}

export const WelcomeWindow: React.FC<WelcomeWindowProps> = ({ onFinish, onReconfigure }) => {
    const { t } = useI18n();
    const [dontShowAgain, setDontShowAgain] = useState(false);
    const [isSaving, setIsSaving] = useState(false);

    const handleStart = async () => {
        setIsSaving(true);
        try {
            if (dontShowAgain) {
                // @ts-ignore
                await window.go.main.App.SetShowWelcome(false);
            }
            onFinish();
        } catch (error) {
            console.error('Failed to close welcome window:', error);
            onFinish();
        } finally {
            setIsSaving(false);
        }
    };

    return (
        <div className="welcome-overlay">
            <div className="welcome-container glass-panel animate-scale">
                <div className="welcome-header">
                    <img src={logo} alt="Soloveyko" className="welcome-logo" />
                    <h1>{t('welcome.title')}</h1>
                    <p className="welcome-subtitle">{t('welcome.subtitle')}</p>
                </div>

                <div className="welcome-content">
                    <p className="welcome-description">
                        {t('welcome.description')}
                    </p>

                    <div className="welcome-features">
                        {/* Future features space */}
                    </div>
                </div>

                <div className="welcome-actions">
                    <button className="welcome-btn-secondary" onClick={onReconfigure}>
                        {t('welcome.quick_setup')}
                    </button>
                    <button className="welcome-btn-primary" onClick={handleStart} disabled={isSaving}>
                        {t('welcome.get_started')}
                    </button>
                </div>

                <div className="welcome-footer">
                    <label className="welcome-dont-show">
                        <input
                            type="checkbox"
                            checked={dontShowAgain}
                            onChange={(e) => setDontShowAgain(e.target.checked)}
                        />
                        <span>{t('welcome.dont_show_again')}</span>
                    </label>
                </div>
            </div>
        </div>
    );
};
