import React, { useState, useEffect, useRef } from 'react';
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

interface GPUData {
    name: string;
    percent: number;
}

interface SystemStats {
    cpuPercent: number;
    ramTotal: number;
    ramUsed: number;
    ramPercent: number;
    gpus: GPUData[];
    disks: DiskInfo[];
}

export const SystemMonitor = () => {
    const { t } = useI18n();
    const [isExpanded, setIsExpanded] = useState(false);
    const [isPinned, setIsPinned] = useState(false);
    const [stats, setStats] = useState<SystemStats | null>(null);
    const wrapperRef = useRef<HTMLDivElement>(null);

    React.useEffect(() => {
        // @ts-ignore
        if (window.runtime) {
            // @ts-ignore
            const unsub = window.runtime.EventsOn("monitor-opened", (id: string) => {
                if (id !== 'system' && !isPinned) {
                    setIsExpanded(false);
                }
            });
            return () => unsub();
        }
    }, [isPinned]);

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (wrapperRef.current && !wrapperRef.current.contains(event.target as Node) && isExpanded && !isPinned) {
                setIsExpanded(false);
            }
        };

        document.addEventListener('mousedown', handleClickOutside);
        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
        };
    }, [isExpanded, isPinned]);

    const handleExpand = (val: boolean) => {
        setIsExpanded(val);
        if (val) {
            // @ts-ignore
            window.runtime?.EventsEmit("monitor-opened", 'system');
        }
    };

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
        <div className={`system-monitor ${isExpanded ? 'expanded' : 'collapsed'} ${isPinned ? 'pinned' : ''}`} ref={wrapperRef}>
            <div className="monitor-container">
                {/* Панель моніторигу */}
                <div className="monitor-panel">
                    <div className="monitor-header">
                        <h3>{t('systemMonitor.cpu')} & {t('systemMonitor.ram')}</h3>
                        <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
                            <button
                                className={`pin-btn ${isPinned ? 'active' : ''}`}
                                onClick={() => setIsPinned(!isPinned)}
                                title={isPinned ? t('common.unpin') : t('common.pin')}
                            >
                                <svg width="14" height="14" viewBox="0 0 24 24" fill={isPinned ? "currentColor" : "none"} stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                                    <line x1="12" y1="17" x2="12" y2="22"></line>
                                    <path d="M5 17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.79-.9A2 2 0 0 1 15 10.76V6a3 3 0 0 0-3-3 3 3 0 0 0-3 3v4.76a2 2 0 0 1-1.11 1.79l-1.79.9A2 2 0 0 0 5 15.24Z"></path>
                                </svg>
                            </button>
                            <button className="close-btn" onClick={() => setIsExpanded(false)}>&times;</button>
                        </div>
                    </div>

                    <div className="monitor-content">
                        {/* CPU */}
                        <div className="stat-group">
                            <div className="stat-label">
                                <span>{t('systemMonitor.cpu')}</span>
                                <span>{(stats?.cpuPercent ?? 0).toFixed(1)}%</span>
                            </div>
                            <div className="stat-bar-bg">
                                <div className="stat-bar-fill" style={{ width: `${stats?.cpuPercent ?? 0}%` }}></div>
                            </div>
                        </div>

                        {/* RAM */}
                        <div className="stat-group">
                            <div className="stat-label">
                                <span>{t('systemMonitor.ram')}</span>
                                <span>{(stats?.ramPercent ?? 0).toFixed(1)}%</span>
                            </div>
                            <div className="stat-bar-bg">
                                <div className="stat-bar-fill" style={{ width: `${stats?.ramPercent ?? 0}%` }}></div>
                            </div>
                            <div className="stat-subtext">
                                {formatBytes(stats?.ramUsed ?? 0)} / {formatBytes(stats?.ramTotal ?? 0)}
                            </div>
                        </div>

                        {/* GPUs */}
                        <div className="gpus-list">
                            {(stats?.gpus || []).map((gpu, idx) => (
                                <div key={idx} className="stat-group gpu-item">
                                    <div className="stat-label">
                                        <span>{t('systemMonitor.gpu')} {(stats?.gpus?.length ?? 0) > 1 ? `#${idx + 1}` : ''}</span>
                                        <span>{(gpu.percent ?? 0).toFixed(1)}%</span>
                                    </div>
                                    <div className="stat-bar-bg">
                                        <div className="stat-bar-fill" style={{ width: `${gpu.percent ?? 0}%` }}></div>
                                    </div>
                                    <div className="gpu-name">{gpu.name}</div>
                                </div>
                            ))}
                            {(!stats?.gpus || stats.gpus.length === 0) && (
                                <div className="stat-group">
                                    <div className="stat-label">
                                        <span>{t('systemMonitor.gpu')}</span>
                                        <span>0.0%</span>
                                    </div>
                                    <div className="stat-bar-bg">
                                        <div className="stat-bar-fill" style={{ width: '0%' }}></div>
                                    </div>
                                    <div className="gpu-name">Detecting...</div>
                                </div>
                            )}
                        </div>

                        {/* Disks */}
                        <div className="stat-group disks-group">
                            <div className="stat-label">
                                <span>{t('systemMonitor.disks')}</span>
                            </div>
                            <div className="disks-list">
                                {(stats?.disks || []).map((disk, idx) => (
                                    <div key={idx} className="disk-item">
                                        <div className="disk-meta">
                                            <span>{disk.mountpoint} ({disk.device})</span>
                                            <span>{(disk.usedPercent ?? 0).toFixed(0)}%</span>
                                        </div>
                                        <div className="stat-bar-bg small">
                                            <div className="stat-bar-fill" style={{ width: `${disk.usedPercent ?? 0}%` }}></div>
                                        </div>
                                        <div className="disk-usage-text">
                                            {t('systemMonitor.free')}: {formatBytes(disk.free ?? 0)} / {formatBytes(disk.total ?? 0)}
                                        </div>
                                    </div>
                                ))}
                            </div>
                        </div>
                    </div>
                </div>

                {/* Кнопка перемикання */}
                <div className="monitor-toggle" onClick={() => handleExpand(!isExpanded)} title="System Monitor">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                        <polyline points="22 12 18 12 15 21 9 3 6 12 2 12" />
                    </svg>
                </div>
            </div>
        </div>
    );
};
