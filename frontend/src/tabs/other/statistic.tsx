import React, { useState, useEffect } from 'react';
import { useI18n } from '../../contexts/I18nContext';
import {
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip,
    ResponsiveContainer,
    AreaChart,
    Area,
    BarChart,
    Bar
} from 'recharts';
import {
    Video,
    Clock,
    Calendar,
    BarChart2,
    Activity,
    Trash2
} from 'lucide-react';
import { GetProductionStats, ClearProductionStats } from '../../../wailsjs/go/main/App';
import './statistic.css';

interface DailyStat {
    date: string;
    videoCount: number;
    totalDuration: number;
}

interface StatsData {
    totalVideos: number;
    totalDuration: number;
    averageDuration: number;
    dailyData: DailyStat[];
    last30DaysVideos: number;
}

export const Statistic = () => {
    const { t } = useI18n();
    const [period, setPeriod] = useState<30 | 0>(30); // 30 days or 0 (all time)
    const [chartType, setChartType] = useState<'area' | 'bar'>('area');
    const [stats, setStats] = useState<StatsData | null>(null);
    const [loading, setLoading] = useState(true);

    const fetchStats = async () => {
        setLoading(true);
        try {
            const data = await GetProductionStats(period);
            setStats(data);
        } catch (err) {
            console.error("Failed to fetch stats:", err);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchStats();
    }, [period]);

    const handleClearStats = async () => {
        if (confirm(t('common.confirm_delete') || "Ви впевнені, що хочете видалити статистику?")) {
            await ClearProductionStats();
            fetchStats();
        }
    };

    const formatDuration = (seconds: number) => {
        if (!seconds) return '0:00';
        const mins = Math.floor(seconds / 60);
        const secs = Math.floor(seconds % 60);
        return `${mins}:${secs.toString().padStart(2, '0')}`;
    };

    const formatDate = (dateStr: string) => {
        const date = new Date(dateStr);
        return date.toLocaleDateString(undefined, { day: 'numeric', month: 'short' });
    };

    if (loading && !stats) {
        return (
            <div className="content-wrapper">
                <div className="stats-dashboard">
                    <div className="spinner-container">
                        <div className="spinner"></div>
                    </div>
                </div>
            </div>
        );
    }

    return (
        <div className="content-wrapper premium-scrollbar">
            <div className="stats-dashboard animate-fade">
                <div className="stats-header">
                    <h2 className="settings-title" style={{ margin: 0 }}>{t('other.statistic')}</h2>
                    <div className="stats-controls">
                        <div className="stats-type-selector">
                            <button
                                className={`period-btn ${chartType === 'area' ? 'active' : ''}`}
                                onClick={() => setChartType('area')}
                                title="Лінійний"
                            >
                                <Activity size={16} />
                            </button>
                            <button
                                className={`period-btn ${chartType === 'bar' ? 'active' : ''}`}
                                onClick={() => setChartType('bar')}
                                title="Стовпчиковий"
                            >
                                <BarChart2 size={16} />
                            </button>
                        </div>
                        <div className="stats-period-selector">
                            <button
                                className={`period-btn ${period === 30 ? 'active' : ''}`}
                                onClick={() => setPeriod(30)}
                            >
                                {t('stats.last_30_days') || 'Останні 30 днів'}
                            </button>
                            <button
                                className={`period-btn ${period === 0 ? 'active' : ''}`}
                                onClick={() => setPeriod(0)}
                            >
                                {t('stats.all_time') || 'Весь час'}
                            </button>
                        </div>
                    </div>
                </div>

                <div className="stats-grid">
                    <div className="dashboard-stat-card" style={{ animationDelay: '0.1s' }}>
                        <div className="dashboard-stat-icon" style={{ background: 'linear-gradient(135deg, #FF0080, #7928CA)' }}>
                            <Video size={32} color="#fff" />
                        </div>
                        <div className="dashboard-stat-info">
                            <span className="dashboard-stat-label">{t('stats.total_videos') || 'Всього відео'}</span>
                            <span className="dashboard-stat-value">{stats?.totalVideos || 0}</span>
                        </div>
                    </div>

                    <div className="dashboard-stat-card" style={{ animationDelay: '0.2s' }}>
                        <div className="dashboard-stat-icon" style={{ background: 'linear-gradient(135deg, #007CF0, #00DFD8)' }}>
                            <Clock size={32} color="#fff" />
                        </div>
                        <div className="dashboard-stat-info">
                            <span className="dashboard-stat-label">{t('stats.avg_time') || 'Середній час'}</span>
                            <span className="dashboard-stat-value">{formatDuration(stats?.averageDuration || 0)}</span>
                        </div>
                    </div>

                    <div className="dashboard-stat-card" style={{ animationDelay: '0.3s' }}>
                        <div className="dashboard-stat-icon" style={{ background: 'linear-gradient(135deg, #FF4D4D, #F9CB28)' }}>
                            <Calendar size={32} color="#fff" />
                        </div>
                        <div className="dashboard-stat-info">
                            <span className="dashboard-stat-label">{t('stats.last_30_days_count') || 'За 30 днів'}</span>
                            <span className="dashboard-stat-value">{stats?.last30DaysVideos || 0}</span>
                        </div>
                    </div>
                </div>

                <div className="chart-container animate-fade" style={{ animationDelay: '0.4s' }}>
                    <h3 className="chart-title">
                        <BarChart2 size={20} style={{ marginRight: '8px', verticalAlign: 'middle' }} />
                        {t('stats.production_graph') || 'Графік виробництва відео'}
                    </h3>
                    <div style={{ width: '100%', height: '350px' }}>
                        <ResponsiveContainer width="100%" height="100%">
                            {chartType === 'area' ? (
                                <AreaChart data={stats?.dailyData || []}>
                                    <defs>
                                        <linearGradient id="colorVideo" x1="0" y1="0" x2="0" y2="1">
                                            <stop offset="5%" stopColor="#ff0080" stopOpacity={0.3} />
                                            <stop offset="95%" stopColor="#ff0080" stopOpacity={0} />
                                        </linearGradient>
                                    </defs>
                                    <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.05)" vertical={false} />
                                    <XAxis
                                        dataKey="date"
                                        tickFormatter={formatDate}
                                        stroke="rgba(255,255,255,0.3)"
                                        fontSize={12}
                                        tickLine={false}
                                        axisLine={false}
                                        dy={10}
                                    />
                                    <YAxis
                                        stroke="rgba(255,255,255,0.3)"
                                        fontSize={12}
                                        tickLine={false}
                                        axisLine={false}
                                        allowDecimals={false}
                                    />
                                    <Tooltip
                                        content={({ active, payload, label }) => {
                                            if (active && payload && payload.length) {
                                                return (
                                                    <div className="custom-tooltip">
                                                        <p className="tooltip-date">{label ? new Date(label).toLocaleDateString() : ''}</p>
                                                        <p className="tooltip-value">{`${payload[0].value} відео`}</p>
                                                    </div>
                                                );
                                            }
                                            return null;
                                        }}
                                    />
                                    <Area
                                        type="monotone"
                                        dataKey="videoCount"
                                        stroke="#ff0080"
                                        strokeWidth={3}
                                        fillOpacity={1}
                                        fill="url(#colorVideo)"
                                        animationDuration={1500}
                                    />
                                </AreaChart>
                            ) : (
                                <BarChart data={stats?.dailyData || []}>
                                    <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.05)" vertical={false} />
                                    <XAxis
                                        dataKey="date"
                                        tickFormatter={formatDate}
                                        stroke="rgba(255,255,255,0.3)"
                                        fontSize={12}
                                        tickLine={false}
                                        axisLine={false}
                                        dy={10}
                                    />
                                    <YAxis
                                        stroke="rgba(255,255,255,0.3)"
                                        fontSize={12}
                                        tickLine={false}
                                        axisLine={false}
                                        allowDecimals={false}
                                    />
                                    <Tooltip
                                        cursor={false}
                                        content={({ active, payload, label }) => {
                                            if (active && payload && payload.length) {
                                                return (
                                                    <div className="custom-tooltip">
                                                        <p className="tooltip-date">{label ? new Date(label).toLocaleDateString() : ''}</p>
                                                        <p className="tooltip-value">{`${payload[0].value} відео`}</p>
                                                    </div>
                                                );
                                            }
                                            return null;
                                        }}
                                    />
                                    <Bar
                                        dataKey="videoCount"
                                        fill="#ff0080"
                                        radius={[4, 4, 0, 0]}
                                        animationDuration={1500}
                                    />
                                </BarChart>
                            )}
                        </ResponsiveContainer>
                    </div>
                </div>

                <button className="test-data-btn" onClick={handleClearStats} style={{
                    borderColor: 'rgba(255, 0, 128, 0.2)',
                    color: 'rgba(255, 255, 255, 0.5)'
                }}>
                    <Trash2 size={14} style={{ marginRight: '6px', verticalAlign: 'middle' }} />
                    {t('stats.clear_stats') || 'Очистити статистику'}
                </button>
            </div>
        </div>
    );
};
