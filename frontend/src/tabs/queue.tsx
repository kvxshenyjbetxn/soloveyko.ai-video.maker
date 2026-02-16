import React from 'react';
import { useI18n } from '../contexts/I18nContext';
import './queue.css';

export const Queue = () => {
    const { t } = useI18n();

    // Черга поки що порожня, дані будуть додані пізніше
    const tasks: any[] = [];

    return (
        <div className="content-wrapper animate-fade">
            <div className="queue-header">
                <h2>{t('queue.title')}</h2>
            </div>

            <div className="queue-container">
                {tasks.length === 0 ? (
                    <div className="queue-empty">
                        <p>{t('queue.empty')}</p>
                    </div>
                ) : (
                    <div className="tasks-list">
                        {tasks.map((task) => (
                            <div key={task.id} className="task-item animate-sidebar-item">
                                <div className="task-info">
                                    <span className="task-name">{task.name}</span>
                                    <span className={`task-status ${task.status.toLowerCase()}`}>
                                        {task.status}
                                    </span>
                                </div>
                                <div className="progress-container">
                                    <div
                                        className="progress-bar"
                                        style={{ width: `${task.progress}%` }}
                                    ></div>
                                </div>
                                <div className="task-footer">
                                    <span>{t('queue.progress')}: {task.progress}%</span>
                                </div>
                            </div>
                        ))}
                    </div>
                )}
            </div>
        </div>
    );
};
