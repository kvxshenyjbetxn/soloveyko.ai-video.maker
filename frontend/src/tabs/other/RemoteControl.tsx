import React, { useState, useEffect } from 'react';
import { useI18n } from '../../contexts/I18nContext';
import { GetWorkerStatus, ToggleWorkerMode, GetAvailableWorkers, GetMyHardwareID } from '../../../wailsjs/go/main/App';
import './RemoteControl.css';

interface WorkerNode {
    id: string;
    name: string;
    hostname: string;
    hardware_id: string;
    status: string;
    last_seen: string;
}

export function RemoteControl() {
    const { t } = useI18n();
    const [workers, setWorkers] = useState<WorkerNode[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [isWorkerActive, setIsWorkerActive] = useState(false);
    const [myHwId, setMyHwId] = useState("");

    const checkStatus = async () => {
        try {
            const active = await GetWorkerStatus();
            setIsWorkerActive(active);
            const hwId = await GetMyHardwareID();
            setMyHwId(hwId.substring(0, 12));
        } catch (e) {
            console.error("Failed to get worker status:", e);
        }
    };

    const fetchWorkers = async () => {
        try {
            const data = await GetAvailableWorkers();
            if (data) {
                setWorkers(data as unknown as WorkerNode[]);
            }
        } catch (e) {
            console.error("Failed to fetch workers:", e);
        } finally {
            setIsLoading(false);
        }
    };

    const toggleWorker = async () => {
        try {
            await ToggleWorkerMode(!isWorkerActive);
            await checkStatus();
        } catch (e) {
            console.error("Failed to toggle worker mode:", e);
        }
    };

    useEffect(() => {
        checkStatus();
        fetchWorkers();
        const interval = setInterval(fetchWorkers, 10000);
        return () => clearInterval(interval);
    }, []);

    return (
        <div className="remote-control-container animate-fade">
            <div className="remote-header">
                <h2>{t('other.remote_control_title') || 'Віддалене керування'}</h2>
                <p className="remote-description">
                    {t('other.remote_description') || 'Керуйте своєю рендер-фермою. Використовуйте потужність декількох ПК для швидкого створення відео.'}
                </p>
            </div>

            <section className="remote-section stagger-1">
                <div className="section-title">
                    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12 2a10 10 0 0 1 10 10c0 5.523-4.477 10-10 10S2 17.523 2 12 6.477 2 12 2Z"/><path d="m9 12 2 2 4-4"/></svg>
                    <span>{t('other.available_workers') || 'Доступні пристрої'}</span>
                </div>
                
                <div className="workers-list">
                    {isLoading ? (
                        <div className="no-workers">Завантаження...</div>
                    ) : workers.length === 0 ? (
                        <div className="no-workers">{t('other.no_workers') || 'Немає підключених пристроїв у вашій мережі'}</div>
                    ) : (
                        workers.map(worker => (
                            <div key={worker.id} className="worker-node">
                                <div className="worker-info">
                                    <div className="status-indicator online"></div>
                                    <div className="worker-details">
                                        <div className="worker-name">{worker.hostname || worker.name}</div>
                                        <div className="worker-hwid">{(worker.hardware_id || '').substring(0, 12)}...</div>
                                    </div>
                                </div>
                                <div className={`worker-status-text status-${(worker.status || '').toLowerCase() === 'busy' ? 'busy' : 'idle'}`}>
                                    {(worker.status || '').toLowerCase() !== 'busy' ? (t('common.ready') || 'Вільний') : (t('common.busy') || 'Зайнятий')}
                                </div>
                            </div>
                        ))
                    )}
                </div>
            </section>

            <section className="remote-section stagger-2">
                <div className="worker-config">
                    <div className="config-text">
                        <div className="section-title">
                            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="2" y="3" width="20" height="14" rx="2" ry="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>
                            <span>{t('other.add_pc') || 'Режим воркера'}</span>
                        </div>
                        <h3>{t('other.add_pc_title') || 'Додати цей ПК як воркер'}</h3>
                        <p>{t('other.worker_description') || 'Увімкніть цей режим, щоб цей ПК міг приймати завдання на рендер з інших пристроїв.'}</p>
                        
                        <div className="worker-name-group" style={{marginTop: '1rem'}}>
                            <div style={{fontSize: '0.9rem', color: 'rgba(255,255,255,0.7)'}}>
                                HWID вашого ПК: <span style={{ fontFamily: 'monospace', userSelect: 'all', background: 'rgba(0,0,0,0.2)', padding: '4px 8px', borderRadius: '4px', marginLeft: '8px', border: '1px solid rgba(255,255,255,0.1)' }}>{myHwId}...</span>
                            </div>
                        </div>
                    </div>
                    <div className="toggle-wrapper">
                        <button 
                            className={`worker-toggle-btn ${isWorkerActive ? 'on' : 'off'}`}
                            onClick={toggleWorker}
                        >
                            {isWorkerActive ? (t('common.enabled') || 'Увімкнено') : (t('common.disabled') || 'Вимкнено')}
                        </button>
                    </div>
                </div>
            </section>
        </div>
    );
}

export default RemoteControl;
