import React, { useState, useEffect } from 'react';
import { useI18n } from '../../contexts/I18nContext';
import { useToast } from '../../contexts/ToastContext';
import './general.css';

const NotificationsSettings: React.FC = () => {
    const { t } = useI18n();
    const { showToast } = useToast();

    const [enabled, setEnabled] = useState(false);
    const [systemEnabled, setSystemEnabled] = useState(false);
    const [chatID, setChatID] = useState("");
    const [isSaving, setIsSaving] = useState(false);
    const [isTesting, setIsTesting] = useState(false);

    useEffect(() => {
        const loadSettings = async () => {
            try {
                // @ts-ignore
                const isEnabled = await window.go.main.App.GetTelegramNotificationsEnabled();
                // @ts-ignore
                const isSystemEnabled = await window.go.main.App.GetSystemNotificationsEnabled();
                // @ts-ignore
                const savedChatID = await window.go.main.App.GetTelegramChatID();

                setEnabled(isEnabled);
                setSystemEnabled(isSystemEnabled);
                setChatID(savedChatID || "");
            } catch (e) {
                console.error("Failed to load notification settings", e);
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
            console.error("Failed to save telegram enabled state", e);
        }
    };

    const handleSaveSystemEnabled = async (newValue: boolean) => {
        setSystemEnabled(newValue);
        try {
            // @ts-ignore
            await window.go.main.App.SaveSystemNotificationsEnabled(newValue);
        } catch (e) {
            console.error("Failed to save system enabled state", e);
        }
    };

    const handleSaveChatID = async (idToSave?: string) => {
        const targetID = idToSave !== undefined ? idToSave : chatID;

        if (!targetID && enabled) {
            // Don't show error on auto-save if empty, only on manual
            if (idToSave === undefined) {
                showToast(t('common.error') + ": " + t('notifications.chat_id') + " is empty", "error");
            }
            return;
        }

        setIsSaving(true);
        try {
            // @ts-ignore
            await window.go.main.App.SaveTelegramChatID(targetID);
            if (idToSave === undefined) {
                showToast(t('common.save') + " OK", "success");
            }
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
            handleSaveChatID(id);
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

    const handleTestSystem = async () => {
        setIsSaving(true); // Reuse state or add new one, isSaving is fine for visual feedback
        try {
            // @ts-ignore
            await window.go.main.App.TestSystemNotification();
            showToast(t('notifications.system_test_sent'), "success");
        } catch (e: any) {
            console.error("Failed to send test system notification", e);
            showToast(t('notifications.system_test_error') + e.toString(), "error");
        } finally {
            setIsSaving(false);
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

                    <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '24px', marginBottom: '30px' }}>
                        {/* Telegram Section */}
                        <div style={{ background: 'var(--bg-secondary)', padding: '20px', borderRadius: '12px', border: '1px solid var(--border)' }}>
                            <div className="settings-controls" style={{ display: 'flex', alignItems: 'center', gap: '12px', userSelect: 'none', marginBottom: '20px' }}>
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
                                    style={{ fontSize: '15px', fontWeight: 600, color: enabled ? 'var(--text-primary)' : 'var(--text-secondary)' }}
                                >
                                    {t('notifications.enable')}
                                </span>
                            </div>

                            <div
                                className="settings-controls"
                                style={{
                                    opacity: enabled ? 1 : 0.5,
                                    pointerEvents: enabled ? 'auto' : 'none',
                                    transition: 'opacity 0.3s ease',
                                    display: 'flex',
                                    flexDirection: 'column',
                                    gap: '15px'
                                }}
                            >
                                <div>
                                    <label className="settings-label" style={{ display: 'block', marginBottom: '8px' }}>{t('notifications.chat_id')}</label>
                                    <div style={{ display: 'flex', gap: '10px' }}>
                                        <input
                                            type="text"
                                            className="settings-input"
                                            value={chatID}
                                            onChange={(e) => setChatID(e.target.value)}
                                            onBlur={() => handleSaveChatID()}
                                            placeholder={t('notifications.chat_id_placeholder')}
                                            style={{ flex: 1, padding: '10px', borderRadius: '8px', border: '1px solid var(--border)', background: 'var(--bg-primary)', color: 'var(--text-primary)' }}
                                        />
                                        <button
                                            className="btn-secondary"
                                            onClick={handleAutofill}
                                            title={t('notifications.autofill_desc')}
                                            style={{ padding: '0 12px', borderRadius: '8px', height: '40px' }}
                                        >
                                            <i className="fa-solid fa-wand-magic-sparkles"></i>
                                        </button>
                                    </div>
                                </div>

                                <div style={{ display: 'flex', gap: '10px' }}>
                                    <button
                                        className="btn-primary"
                                        onClick={() => handleSaveChatID()}
                                        disabled={isSaving}
                                        style={{ padding: '10px 15px', borderRadius: '8px', fontSize: '14px', flex: 1 }}
                                    >
                                        {isSaving ? <i className="fa-solid fa-spinner fa-spin" style={{ marginRight: '8px' }}></i> : <i className="fa-solid fa-floppy-disk" style={{ marginRight: '8px' }}></i>}
                                        {t('common.save')}
                                    </button>
                                    <button
                                        className="btn-secondary"
                                        onClick={handleTest}
                                        disabled={isTesting || !chatID}
                                        style={{ padding: '10px 15px', borderRadius: '8px', fontSize: '14px', flex: 1 }}
                                    >
                                        {isTesting ? <i className="fa-solid fa-spinner fa-spin" style={{ marginRight: '8px' }}></i> : <i className="fa-solid fa-paper-plane" style={{ marginRight: '8px' }}></i>}
                                        {t('notifications.test')}
                                    </button>
                                </div>
                            </div>
                        </div>

                        {/* System Section */}
                        <div style={{ background: 'var(--bg-secondary)', padding: '20px', borderRadius: '12px', border: '1px solid var(--border)', display: 'flex', flexDirection: 'column' }}>
                            <div className="settings-controls" style={{ display: 'flex', alignItems: 'center', gap: '12px', userSelect: 'none', marginBottom: '20px' }}>
                                <label className="toggle-switch">
                                    <input
                                        type="checkbox"
                                        checked={systemEnabled}
                                        onChange={(e) => handleSaveSystemEnabled(e.target.checked)}
                                    />
                                    <span className="toggle-slider" style={systemEnabled ? { backgroundColor: 'var(--accent-primary)' } : {}}></span>
                                </label>
                                <span
                                    className="toggle-label"
                                    onClick={() => handleSaveSystemEnabled(!systemEnabled)}
                                    style={{ fontSize: '15px', fontWeight: 600, color: systemEnabled ? 'var(--text-primary)' : 'var(--text-secondary)' }}
                                >
                                    {t('notifications.system_enable')}
                                </span>
                            </div>

                            <div
                                style={{
                                    opacity: systemEnabled ? 1 : 0.5,
                                    pointerEvents: systemEnabled ? 'auto' : 'none',
                                    transition: 'opacity 0.3s ease',
                                    marginTop: 'auto'
                                }}
                            >
                                <button
                                    className="btn-secondary"
                                    onClick={handleTestSystem}
                                    style={{ width: '100%', padding: '10px 15px', borderRadius: '8px', fontSize: '14px' }}
                                >
                                    <i className="fa-solid fa-desktop" style={{ marginRight: '8px' }}></i>
                                    {t('notifications.system_test')}
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default NotificationsSettings;
