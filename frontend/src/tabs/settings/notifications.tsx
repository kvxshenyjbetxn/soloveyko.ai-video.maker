import React, { useState, useEffect } from 'react';
import { useI18n } from '../../contexts/I18nContext';
import { useToast } from '../../contexts/ToastContext';
import './general.css';

const NotificationsSettings: React.FC = () => {
    const { t } = useI18n();
    const { showToast } = useToast();

    const [enabled, setEnabled] = useState(false);
    const [chatID, setChatID] = useState("");
    const [isSaving, setIsSaving] = useState(false);
    const [isTesting, setIsTesting] = useState(false);

    useEffect(() => {
        const loadSettings = async () => {
            try {
                // @ts-ignore
                const isEnabled = await window.go.main.App.GetTelegramNotificationsEnabled();
                // @ts-ignore
                const savedChatID = await window.go.main.App.GetTelegramChatID();

                setEnabled(isEnabled);
                setChatID(savedChatID || "");
            } catch (e) {
                console.error("Failed to load telegram settings", e);
            }
        };
        loadSettings();
    }, []);

    const handleSaveEnabled = async (newValue: boolean) => {
        setEnabled(newValue);
        try {
            // @ts-ignore
            await window.go.main.App.SaveTelegramNotificationsEnabled(newValue);
        } catch (e) {
            console.error("Failed to save enabled state", e);
        }
    };

    const handleSaveChatID = async () => {
        if (!chatID && enabled) {
            showToast(t('common.error') + ": " + t('notifications.chat_id') + " is empty", "error");
            return;
        }
        setIsSaving(true);
        try {
            // @ts-ignore
            await window.go.main.App.SaveTelegramChatID(chatID);
            showToast(t('common.save') + " OK", "success");
        } catch (e) {
            console.error("Failed to save chat ID", e);
            showToast(t('common.error'), "error");
        } finally {
            setIsSaving(false);
        }
    };

    const handleAutofill = () => {
        const id = sessionStorage.getItem('telegram_id');
        if (id) {
            setChatID(id);
        } else {
            showToast(t('common.error'), "error");
        }
    };

    const handleTest = async () => {
        if (!chatID) {
            showToast(t('common.error') + ": " + t('notifications.chat_id') + " is empty", "error");
            return;
        }
        setIsTesting(true);
        try {
            // @ts-ignore
            await window.go.main.App.TestTelegramNotification(chatID);
            showToast(t('notifications.test_sent'), "success");
        } catch (e: any) {
            console.error("Failed to send test notification", e);
            showToast(t('notifications.test_error') + e.toString(), "error");
        } finally {
            setIsTesting(false);
        }
    };

    return (
        <div className="content-wrapper animate-fade">
            <div className="settings-container">
                <div className="settings-section">
                    <h3 className="section-title">{t('notifications.tab_title')}</h3>
                    <p className="section-description" style={{ color: 'var(--text-secondary)', marginBottom: '20px' }}>
                        {t('notifications.description')}
                    </p>

                    <div className="settings-controls" style={{ display: 'flex', alignItems: 'center', marginBottom: '20px', gap: '12px', userSelect: 'none' }}>
                        <label className="toggle-switch">
                            <input
                                type="checkbox"
                                checked={enabled}
                                onChange={(e) => handleSaveEnabled(e.target.checked)}
                            />
                            <span className="toggle-slider" style={enabled ? { backgroundColor: 'var(--accent-primary)' } : {}}></span>
                        </label>
                        <span
                            className="toggle-label"
                            onClick={() => handleSaveEnabled(!enabled)}
                            style={{ fontSize: '15px', color: enabled ? 'var(--text-primary)' : 'var(--text-secondary)' }}
                        >
                            {t('notifications.enable')}
                        </span>
                    </div>

                    <div
                        className="settings-controls"
                        style={{
                            opacity: enabled ? 1 : 0.5,
                            pointerEvents: enabled ? 'auto' : 'none',
                            transition: 'opacity 0.3s ease'
                        }}
                    >
                        <div style={{ display: 'flex', gap: '15px', alignItems: 'center' }}>
                            <div style={{ flex: 1 }}>
                                <label className="settings-label" style={{ display: 'block', marginBottom: '8px' }}>{t('notifications.chat_id')}</label>
                                <input
                                    type="text"
                                    className="settings-input"
                                    value={chatID}
                                    onChange={(e) => setChatID(e.target.value)}
                                    placeholder={t('notifications.chat_id_placeholder')}
                                    style={{ width: '100%', padding: '10px', borderRadius: '8px', border: '1px solid var(--border)', background: 'var(--bg-secondary)', color: 'var(--text-primary)' }}
                                />
                            </div>
                            <button
                                className="btn-secondary"
                                onClick={handleAutofill}
                                title={t('notifications.autofill_desc')}
                                style={{ marginTop: '26px', padding: '10px 15px', borderRadius: '8px' }}
                            >
                                <i className="fa-solid fa-wand-magic-sparkles" style={{ marginRight: '8px' }}></i> {t('notifications.autofill')}
                            </button>
                        </div>

                        <div style={{ display: 'flex', gap: '15px', marginTop: '20px' }}>
                            <button
                                className="btn-primary"
                                onClick={handleSaveChatID}
                                disabled={isSaving}
                                style={{ padding: '10px 20px', borderRadius: '8px' }}
                            >
                                {isSaving ? <i className="fa-solid fa-spinner fa-spin" style={{ marginRight: '8px' }}></i> : <i className="fa-solid fa-floppy-disk" style={{ marginRight: '8px' }}></i>}
                                {t('common.save')}
                            </button>
                            <button
                                className="btn-secondary"
                                onClick={handleTest}
                                disabled={isTesting || !chatID}
                                style={{ padding: '10px 20px', borderRadius: '8px' }}
                            >
                                {isTesting ? <i className="fa-solid fa-spinner fa-spin" style={{ marginRight: '8px' }}></i> : <i className="fa-solid fa-paper-plane" style={{ marginRight: '8px' }}></i>}
                                {t('notifications.test')}
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default NotificationsSettings;
