import { useState } from 'react';
import { useI18n } from '../../contexts/I18nContext';
import { useAuth } from '../../contexts/AuthContext';
import { useToast } from '../../contexts/ToastContext';
import { renameMyDevice, useMyDevices } from '../../lib/devicePresence';
import './remote_control.css';

export const RemoteControl = () => {
    const { t } = useI18n();
    const { user } = useAuth();
    const { showToast } = useToast();
    const devices = useMyDevices(user);
    const [editingDeviceId, setEditingDeviceId] = useState<string | null>(null);
    const [deviceNameDraft, setDeviceNameDraft] = useState('');
    const [renaming, setRenaming] = useState(false);
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
                        </>
                    )}
                </div>
            </div>
        </div>
    );
};
