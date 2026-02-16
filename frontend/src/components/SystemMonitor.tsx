import React, { useState, useEffect } from 'react';
import { useI18n } from '../contexts/I18nContext';
import './SystemMonitor.css';

interface DiskInfo {
    device: string;
    mountpoint: string;
    total: number;
    free: number;
    used: number;
    usedPercent: number;
}

interface SystemStats {
    cpuPercent: number;
    ramTotal: number;
    ramUsed: number;
    ramPercent: number;
    gpuInfo: string;
    gpuPercent: number;
    disks: DiskInfo[];
}

export const SystemMonitor = () => {
    const { t } = useI18n();
    const [isExpanded, setIsExpanded] = useState(false);
    const [stats, setStats] = useState<SystemStats | null>(null);

    useEffect(() => {
        const fetchStats = async () => {
            try {
                // @ts-ignore
                const data = await window.go.main.App.GetSystemStats();
                setStats(data);
            } catch (err) {
                console.error("Failed to fetch system stats:", err);
            }
        };

        fetchStats();
        const interval = setInterval(fetchStats, 3000);
        return () => clearInterval(interval);
    }, []);

    const formatBytes = (bytes: number) => {
        if (bytes === 0) return '0 Bytes';
        const k = 1024;
        const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    };

    return (
        <div className={`system-monitor ${isExpanded ? 'expanded' : 'collapsed'}`}>
            <div className="monitor-container">
                {/* Панель моніторингу */}
                <div className="monitor-panel">
                    <div className="monitor-header">
                        <h3>{t('systemMonitor.cpu')} & {t('systemMonitor.ram')}</h3>
                        <button className="close-btn" onClick={() => setIsExpanded(false)}>&times;</button>
                    </div>

                    <div className="monitor-content">
                        {/* CPU */}
                        <div className="stat-group">
                            <div className="stat-label">
                                <span>{t('systemMonitor.cpu')}</span>
                                <span>{stats?.cpuPercent.toFixed(1)}%</span>
                            </div>
                            <div className="stat-bar-bg">
                                <div className="stat-bar-fill" style={{ width: `${stats?.cpuPercent}%` }}></div>
                            </div>
                        </div>

                        {/* RAM */}
                        <div className="stat-group">
                            <div className="stat-label">
                                <span>{t('systemMonitor.ram')}</span>
                                <span>{stats?.ramPercent.toFixed(1)}%</span>
                            </div>
                            <div className="stat-bar-bg">
                                <div className="stat-bar-fill" style={{ width: `${stats?.ramPercent}%` }}></div>
                            </div>
                            <div className="stat-subtext">
                                {formatBytes(stats?.ramUsed || 0)} / {formatBytes(stats?.ramTotal || 0)}
                            </div>
                        </div>

                        {/* GPU */}
                        <div className="stat-group">
                            <div className="stat-label">
                                <span>{t('systemMonitor.gpu')}</span>
                                <span>{stats?.gpuPercent.toFixed(1)}%</span>
                            </div>
                            <div className="stat-bar-bg">
                                <div className="stat-bar-fill" style={{ width: `${stats?.gpuPercent}%` }}></div>
                            </div>
                            <div className="gpu-name">{stats?.gpuInfo || 'Detecting...'}</div>
                        </div>

                        {/* Disks */}
                        <div className="stat-group disks-group">
                            <div className="stat-label">
                                <span>{t('systemMonitor.disks')}</span>
                            </div>
                            <div className="disks-list">
                                {stats?.disks.map((disk, idx) => (
                                    <div key={idx} className="disk-item">
                                        <div className="disk-meta">
                                            <span>{disk.mountpoint} ({disk.device})</span>
                                            <span>{disk.usedPercent.toFixed(0)}%</span>
                                        </div>
                                        <div className="stat-bar-bg small">
                                            <div className="stat-bar-fill" style={{ width: `${disk.usedPercent}%` }}></div>
                                        </div>
                                        <div className="disk-usage-text">
                                            {t('systemMonitor.free')}: {formatBytes(disk.free)} / {formatBytes(disk.total)}
                                        </div>
                                    </div>
                                ))}
                            </div>
                        </div>
                    </div>
                </div>

                {/* Кнопка перемикання */}
                <div className="monitor-toggle" onClick={() => setIsExpanded(!isExpanded)} title="System Monitor">
                    <div className="pulse-icon">
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            <polyline points="22 12 18 12 15 21 9 3 6 12 2 12" />
                        </svg>
                    </div>
                </div>
            </div>
        </div>
    );
};
