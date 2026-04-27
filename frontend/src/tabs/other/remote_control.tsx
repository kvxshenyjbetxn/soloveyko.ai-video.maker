import { useState, useEffect } from 'react';
import { useI18n } from '../../contexts/I18nContext';
import { useAuth } from '../../contexts/AuthContext';
import { useToast } from '../../contexts/ToastContext';
import { renameMyDevice, useMyDevices } from '../../lib/devicePresence';
import { GetRemotePreviewLimit, SaveRemotePreviewLimit } from '../../../wailsjs/go/main/App';
import './remote_control.css';

export const RemoteControl = () => {
    const { t } = useI18n();
    const { user } = useAuth();
    const { showToast } = useToast();
    const devices = useMyDevices(user);
    const [editingDeviceId, setEditingDeviceId] = useState<string | null>(null);
    const [deviceNameDraft, setDeviceNameDraft] = useState('');
    const [renaming, setRenaming] = useState(false);
    
    const [previewLimit, setPreviewLimit] = useState<number>(3);
    const [savingLimit, setSavingLimit] = useState(false);

    useEffect(() => {
        GetRemotePreviewLimit().then(limit => {
            setPreviewLimit(limit);
        }).catch(err => console.error("Failed to load preview limit:", err));
    }, []);

    const handleLimitChange = async (newLimit: number) => {
        if (newLimit < 1) newLimit = 1;
        if (newLimit > 50) newLimit = 50;
        setPreviewLimit(newLimit);
        setSavingLimit(true);
        try {
            await SaveRemotePreviewLimit(newLimit);
            showToast(t('general.saved_successfully') || 'Saved', 'success');
        } catch (err) {
            console.error("Failed to save preview limit:", err);
            showToast(t('general.save_error') || 'Error saving', 'error');
        } finally {
            setSavingLimit(false);
        }
    };

    const hasRealtimePresence = Boolean(import.meta.env.VITE_FIREBASE_DATABASE_URL);

    return (
        <div className="content-wrapper animate-fade">
            <div className="settings-container">
                <h2 className="settings-title">{t('other.remote_control')}</h2>
                <div className="settings-section">
                    <p className="section-description">{t('other.remote_control_description')}</p>
                    {!user && (
                        <p className="devices-hint">{t('general.devicesSignInHint')}</p>
                    )}
                    {user && (
                        <>
                            <ul className="device-list">
                                {devices.map((row) => (
                                    <li key={row.deviceId} className="device-row">
                                        <div className="device-row-main">
                                            <span
                                                className={`device-status device-status-${row.state}`}
                                                title={row.state === 'online' ? t('general.devicesOnline') : t('general.devicesOffline')}
                                            />
                                            {editingDeviceId === row.deviceId ? (
                                                <input
                                                    className="device-name-input"
                                                    value={deviceNameDraft}
                                                    onChange={(e) => setDeviceNameDraft(e.target.value)}
                                                    disabled={renaming}
                                                    autoFocus
                                                />
                                            ) : (
                                                <span className="device-name">
                                                    {row.name}
                                                    {row.isCurrent ? (
                                                        <span className="device-this-pc"> · {t('general.devicesThisPc')}</span>
                                                    ) : null}
                                                </span>
                                            )}
                                        </div>
                                        <div className="device-row-actions">
                                            {editingDeviceId === row.deviceId ? (
                                                <>
                                                    <button
                                                        type="button"
                                                        className="btn-secondary device-action"
                                                        disabled={renaming}
                                                        onClick={() => {
                                                            setEditingDeviceId(null);
                                                            setDeviceNameDraft('');
                                                        }}
                                                    >
                                                        {t('general.devicesCancel')}
                                                    </button>
                                                    <button
                                                        type="button"
                                                        className="btn-secondary device-action device-action-primary"
                                                        disabled={renaming}
                                                        onClick={async () => {
                                                            if (!user) {
                                                                return;
                                                            }
                                                            setRenaming(true);
                                                            try {
                                                                await renameMyDevice(user, row.deviceId, deviceNameDraft);
                                                                showToast(t('general.devicesRenamed'), 'success');
                                                                setEditingDeviceId(null);
                                                                setDeviceNameDraft('');
                                                            } catch (err) {
                                                                const message = err instanceof Error ? err.message : String(err);
                                                                showToast(message, 'error');
                                                            } finally {
                                                                setRenaming(false);
                                                            }
                                                        }}
                                                    >
                                                        {t('general.devicesSave')}
                                                    </button>
                                                </>
                                            ) : (
                                                <button
                                                    type="button"
                                                    className="btn-secondary device-action"
                                                    onClick={() => {
                                                        setEditingDeviceId(row.deviceId);
                                                        setDeviceNameDraft(row.name);
                                                    }}
                                                >
                                                    {t('general.devicesRename')}
                                                </button>
                                            )}
                                        </div>
                                    </li>
                                ))}
                            </ul>
                            {!hasRealtimePresence && (
                                <p className="devices-hint">{t('general.devicesRealtimeOff')}</p>
                            )}

                            <div className="settings-section" style={{ marginTop: '2rem' }}>
                                <h3>{t('other.remote_preview_settings') || 'Налаштування прев\'ю'}</h3>
                                <p className="section-description">
                                    {t('other.remote_preview_desc') || 'Вкажіть кількість перших зображень/відео, які будуть передаватись на Майстер ПК для віддаленого контролю кожного завдання.'}
                                </p>
                                <div className="setting-row">
                                    <label className="setting-label">
                                        {t('other.remote_preview_limit') || 'Кількість медіа (N):'}
                                    </label>
                                    <input 
                                        type="number" 
                                        className="device-name-input" 
                                        style={{ width: '80px', marginLeft: '1rem' }}
                                        value={previewLimit}
                                        min={1}
                                        max={50}
                                        disabled={savingLimit}
                                        onChange={(e) => setPreviewLimit(parseInt(e.target.value) || 1)}
                                        onBlur={(e) => handleLimitChange(parseInt(e.target.value) || 1)}
                                    />
                                </div>
                            </div>
                        </>
                    )}
                </div>
            </div>
        </div>
    );
};
