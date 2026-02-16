import React, { useState, useMemo } from 'react';
import { useI18n } from '../contexts/I18nContext';
import { useLogger, LogLevel } from '../contexts/LoggerContext';
import './Logs.css';

export const Logs = () => {
    const { t } = useI18n();
    const { logs, clearLogs } = useLogger();

    const [filterLevel, setFilterLevel] = useState<LogLevel | 'ALL'>('ALL');
    const [sortOrder, setSortOrder] = useState<'ASC' | 'DESC'>('DESC');
    const [searchQuery, setSearchQuery] = useState('');

    const filteredLogs = useMemo(() => {
        let result = logs;

        // Filter by Level
        if (filterLevel !== 'ALL') {
            result = result.filter(log => log.level === filterLevel);
        }

        // Filter by Search Query
        if (searchQuery.trim()) {
            const query = searchQuery.toLowerCase();
            result = result.filter(log =>
                log.message.toLowerCase().includes(query) ||
                log.level.toLowerCase().includes(query)
            );
        }

        // Sort
        return result.sort((a, b) => {
            if (sortOrder === 'ASC') {
                return a.timestamp.getTime() - b.timestamp.getTime();
            } else {
                return b.timestamp.getTime() - a.timestamp.getTime();
            }
        });
    }, [logs, filterLevel, sortOrder, searchQuery]);

    const getLevelColor = (level: LogLevel) => {
        switch (level) {
            case 'INFO': return '#4caf50';
            case 'ERROR': return '#f44336';
            case 'WARN': return '#ff9800';
            case 'DEBUG': return '#9e9e9e';
            default: return 'var(--text-secondary)';
        }
    };

    return (
        <div className="content-wrapper">
            <div className="settings-container" style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
                    <h2 className="settings-title" style={{ margin: 0 }}>{t('tabs.logs')}</h2>
                    <div style={{ display: 'flex', gap: '8px' }}>
                        <button onClick={clearLogs} style={{ padding: '6px 12px', background: '#f44336', color: '#fff', border: 'none', borderRadius: '4px', cursor: 'pointer' }}>
                            {t('logsTab.clear')}
                        </button>
                    </div>
                </div>

                {/* Controls */}
                <div style={{ display: 'flex', gap: '10px', marginBottom: '10px', flexWrap: 'wrap' }}>
                    <input
                        type="text"
                        placeholder={t('logsTab.searchPlaceholder') || 'Search...'}
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        style={{
                            padding: '6px 10px',
                            borderRadius: '4px',
                            background: 'var(--bg-tertiary)',
                            color: 'var(--text-primary)',
                            border: '1px solid var(--border-color)',
                            flex: 1,
                            minWidth: '150px'
                        }}
                    />

                    <select
                        value={filterLevel}
                        onChange={(e) => setFilterLevel(e.target.value as any)}
                        style={{ padding: '6px', borderRadius: '4px', background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border-color)' }}
                    >
                        <option value="ALL">{t('logsTab.allLevels')}</option>
                        <option value="INFO">INFO</option>
                        <option value="ERROR">ERROR</option>
                        <option value="WARN">WARN</option>
                        <option value="DEBUG">DEBUG</option>
                    </select>

                    <select
                        value={sortOrder}
                        onChange={(e) => setSortOrder(e.target.value as any)}
                        style={{ padding: '6px', borderRadius: '4px', background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border-color)' }}
                    >
                        <option value="DESC">{t('logsTab.newestFirst')}</option>
                        <option value="ASC">{t('logsTab.oldestFirst')}</option>
                    </select>
                </div>

                <div className="logs-container" style={{
                    backgroundColor: 'var(--bg-secondary)',
                    border: '1px solid var(--border-color)',
                    borderRadius: '4px',
                    padding: '16px',
                    fontFamily: 'var(--font-mono)',
                    fontSize: '12px',
                    color: 'var(--text-secondary)',
                    flex: 1,
                    overflowY: 'auto'
                }}>
                    {filteredLogs.length === 0 ? (
                        <div style={{ color: 'var(--text-placeholder)', fontStyle: 'italic' }}>{t('logsTab.empty')}</div>
                    ) : (
                        filteredLogs.map(log => (
                            <div key={log.id} className="log-row">
                                <span style={{ color: 'var(--text-placeholder)', minWidth: '85px', fontSize: '12px' }}>
                                    {log.timestamp.toLocaleTimeString()}
                                </span>
                                <span style={{ color: getLevelColor(log.level), fontWeight: 'bold', minWidth: '60px', fontSize: '12px' }}>
                                    [{log.level}]
                                </span>
                                <span style={{ color: 'var(--text-primary)', wordBreak: 'break-all', fontSize: '13px' }}>
                                    {log.message}
                                </span>
                            </div>
                        ))
                    )}
                </div>
            </div>
        </div>
    );
};
