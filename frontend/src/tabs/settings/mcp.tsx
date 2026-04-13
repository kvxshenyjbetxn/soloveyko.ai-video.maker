import React, { useEffect, useState } from 'react';
import { useI18n } from '../../contexts/I18nContext';
import {
    GetMCPAutoForwardEnabled,
    GetMCPForwardStatus,
    GetMCPForwardScriptPath,
    GetOS,
    SaveMCPAutoForwardEnabled,
} from '../../../wailsjs/go/main/App';
import './general.css';

type MCPForwardStatus = {
    supported?: boolean;
    enabled?: boolean;
    os?: string;
    scriptFound?: boolean;
    scriptPath?: string;
    running?: boolean;
    pid?: number;
};

const MCPSettings: React.FC = () => {
    const { t } = useI18n();
    const [enabled, setEnabled] = useState(false);
    const [osName, setOSName] = useState('');
    const [scriptPath, setScriptPath] = useState('');
    const [status, setStatus] = useState<MCPForwardStatus>({});

    useEffect(() => {
        const refreshStatus = async () => {
            try {
                const currentStatus = await GetMCPForwardStatus() as MCPForwardStatus;
                setStatus(currentStatus || {});
                if (typeof currentStatus?.scriptPath === 'string') {
                    setScriptPath(currentStatus.scriptPath);
                }
            } catch (error) {
                console.error('Failed to load MCP status', error);
            }
        };

        const load = async () => {
            try {
                const [savedEnabled, platform, resolvedPath] = await Promise.all([
                    GetMCPAutoForwardEnabled(),
                    GetOS(),
                    GetMCPForwardScriptPath(),
                ]);

                setEnabled(!!savedEnabled);
                setOSName(platform || '');
                setScriptPath(resolvedPath || '');
            } catch (error) {
                console.error('Failed to load MCP settings', error);
            }
        };

        load();
        refreshStatus();

        const intervalId = window.setInterval(() => {
            void refreshStatus();
        }, 3000);

        return () => {
            window.clearInterval(intervalId);
        };
    }, []);

    const handleToggle = async (nextValue: boolean) => {
        setEnabled(nextValue);
        try {
            await SaveMCPAutoForwardEnabled(nextValue);
            const currentStatus = await GetMCPForwardStatus() as MCPForwardStatus;
            setStatus(currentStatus || {});
        } catch (error) {
            console.error('Failed to save MCP auto-forward setting', error);
            setEnabled(!nextValue);
        }
    };

    const isWindows = osName === 'windows';
    const isRunning = !!status.running;
    const isScriptFound = !!status.scriptFound;

    return (
        <div className="content-wrapper animate-fade">
            <div className="settings-container">
                <div className="settings-section">
                    <h3 className="section-title">{t('mcpTab.title')}</h3>
                    <p className="section-description">{t('mcpTab.description')}</p>
                </div>

                <div
                    className="settings-section"
                    style={{
                        background: 'var(--bg-secondary)',
                        border: '1px solid var(--border-color)',
                        borderRadius: '12px',
                        padding: '20px',
                    }}
                >
                    <div
                        className="settings-controls"
                        style={{ display: 'flex', alignItems: 'center', gap: '12px', userSelect: 'none', marginBottom: '10px' }}
                    >
                        <label className="toggle-switch">
                            <input
                                type="checkbox"
                                checked={enabled}
                                disabled={!isWindows}
                                onChange={(e) => handleToggle(e.target.checked)}
                            />
                            <span className="toggle-slider" style={enabled ? { backgroundColor: 'var(--accent-primary)' } : {}}></span>
                        </label>
                        <span
                            className="toggle-label"
                            onClick={() => isWindows && handleToggle(!enabled)}
                            style={{ fontSize: '15px', fontWeight: 600, color: enabled ? 'var(--text-primary)' : 'var(--text-secondary)' }}
                        >
                            {t('mcpTab.auto_forward')}
                        </span>
                    </div>

                    <p className="section-description" style={{ marginBottom: '14px' }}>
                        {isWindows ? t('mcpTab.auto_forward_desc') : t('mcpTab.windows_only')}
                    </p>

                    <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', fontSize: '13px', color: 'var(--text-secondary)' }}>
                        <div style={{ display: 'flex', gap: '10px', flexWrap: 'wrap', marginBottom: '10px' }}>
                            <span style={{
                                padding: '6px 10px',
                                borderRadius: '999px',
                                border: '1px solid var(--border-color)',
                                background: enabled ? 'rgba(0, 200, 120, 0.12)' : 'rgba(255, 255, 255, 0.04)',
                                color: enabled ? 'var(--text-primary)' : 'var(--text-secondary)',
                            }}>
                                {enabled ? t('mcpTab.status_autostart_on') : t('mcpTab.status_autostart_off')}
                            </span>
                            <span style={{
                                padding: '6px 10px',
                                borderRadius: '999px',
                                border: '1px solid var(--border-color)',
                                background: isScriptFound ? 'rgba(0, 120, 255, 0.12)' : 'rgba(255, 120, 120, 0.12)',
                                color: 'var(--text-primary)',
                            }}>
                                {isScriptFound ? t('mcpTab.status_script_found') : t('mcpTab.status_script_missing')}
                            </span>
                            <span style={{
                                padding: '6px 10px',
                                borderRadius: '999px',
                                border: '1px solid var(--border-color)',
                                background: isRunning ? 'rgba(0, 200, 120, 0.12)' : 'rgba(255, 255, 255, 0.04)',
                                color: isRunning ? 'var(--text-primary)' : 'var(--text-secondary)',
                            }}>
                                {isRunning ? t('mcpTab.status_tunnel_running') : t('mcpTab.status_tunnel_stopped')}
                            </span>
                        </div>
                        <div>
                            <strong style={{ color: 'var(--text-primary)' }}>{t('mcpTab.script_name')}:</strong> startVPS.bat
                        </div>
                        <div>
                            <strong style={{ color: 'var(--text-primary)' }}>{t('mcpTab.platform')}:</strong> {osName || t('common.loading')}
                        </div>
                        <div>
                            <strong style={{ color: 'var(--text-primary)' }}>{t('mcpTab.lookup_order')}:</strong> {t('mcpTab.lookup_order_value')}
                        </div>
                        <div>
                            <strong style={{ color: 'var(--text-primary)' }}>{t('mcpTab.resolved_path')}:</strong> {scriptPath || t('mcpTab.not_found')}
                        </div>
                        <div>
                            <strong style={{ color: 'var(--text-primary)' }}>{t('mcpTab.process_id')}:</strong> {status.pid ? String(status.pid) : t('mcpTab.no_process')}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default MCPSettings;
