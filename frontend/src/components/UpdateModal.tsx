import React, { useState, useEffect } from 'react';
import { useI18n } from '../contexts/I18nContext';
import { utils } from '../../wailsjs/go/models';

interface UpdateModalProps {
    isOpen: boolean;
    manifest: utils.UpdateManifest;
    onClose: () => void;
}

export const UpdateModal: React.FC<UpdateModalProps> = ({ isOpen, manifest, onClose }) => {
    const { t, locale } = useI18n();
    const [isDownloading, setIsDownloading] = useState(false);
    const [progress, setProgress] = useState(0);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        if (!isOpen) {
            setIsDownloading(false);
            setProgress(0);
            setError(null);
        }
    }, [isOpen]);

    useEffect(() => {
        // @ts-ignore
        if (window.runtime) {
            // @ts-ignore
            const unsub = window.runtime.EventsOn("updateProgress", (p: number) => {
                setProgress(p);
            });
            return () => unsub();
        }
    }, []);

    if (!isOpen) return null;

    const handleUpdate = async () => {
        setIsDownloading(true);
        setError(null);
        try {
            // @ts-ignore
            const pkgPath = await window.go.main.App.DownloadUpdate(manifest.url, manifest.checksum);
            if (pkgPath) {
                // @ts-ignore
                await window.go.main.App.ApplyUpdate(pkgPath);
            }
        } catch (e: any) {
            console.error("Update failed:", e);
            setError(e.toString());
            setIsDownloading(false);
        }
    };

    const getTitle = () => {
        if (locale === 'uk') return 'Доступне оновлення';
        if (locale === 'ru') return 'Доступно обновление';
        return 'Update Available';
    };

    const getButtonText = () => {
        if (isDownloading) {
            if (locale === 'uk') return `Завантаження... ${progress}%`;
            if (locale === 'ru') return `Загрузка... ${progress}%`;
            return `Downloading... ${progress}%`;
        }
        if (locale === 'uk') return 'Оновити зараз';
        if (locale === 'ru') return 'Обновить сейчас';
        return 'Update Now';
    };

    return (
        <div className="modal-overlay" style={{ zIndex: 20000 }}>
            <div className="modal-content animate-scale" style={{ maxWidth: '500px', width: '90%' }}>
                <div className="modal-header">
                    <h2 className="modal-title">{getTitle()}</h2>
                    {!isDownloading && (
                        <button className="modal-close" onClick={onClose}>&times;</button>
                    )}
                </div>

                <div className="modal-body">
                    <div style={{ marginBottom: '15px', fontSize: '1.1rem', fontWeight: 'bold' }}>
                        v{manifest.version}
                    </div>

                    {manifest.notes && (
                        <div className="update-notes" style={{
                            maxHeight: '200px',
                            overflowY: 'auto',
                            padding: '10px',
                            background: 'rgba(0,0,0,0.2)',
                            borderRadius: '5px',
                            fontSize: '0.9rem',
                            whiteSpace: 'pre-wrap',
                            marginBottom: '20px'
                        }}>
                            {manifest.notes}
                        </div>
                    )}

                    {isDownloading && (
                        <div className="progress-container" style={{ marginBottom: '20px' }}>
                            <div className="progress-bar-bg" style={{ height: '8px', background: 'rgba(255,255,255,0.1)', borderRadius: '4px', overflow: 'hidden' }}>
                                <div className="progress-bar-fill" style={{
                                    height: '100%',
                                    width: `${progress}%`,
                                    background: 'var(--accent-color, #0078d4)',
                                    transition: 'width 0.3s ease'
                                }} />
                            </div>
                        </div>
                    )}

                    {error && (
                        <div style={{ color: '#ff4d4d', fontSize: '0.85rem', marginBottom: '15px' }}>
                            {error}
                        </div>
                    )}
                </div>

                <div className="modal-footer">
                    {!isDownloading && (
                        <button className="btn-secondary" onClick={onClose}>
                            {locale === 'uk' ? 'Пізніше' : (locale === 'ru' ? 'Позже' : 'Later')}
                        </button>
                    )}
                    <button
                        className="btn-primary"
                        onClick={handleUpdate}
                        disabled={isDownloading}
                        style={{ minWidth: '150px' }}
                    >
                        {getButtonText()}
                    </button>
                </div>
            </div>

            <style dangerouslySetInnerHTML={{
                __html: `
                .modal-overlay {
                    position: fixed;
                    top: 0;
                    left: 0;
                    right: 0;
                    bottom: 0;
                    background: rgba(0, 0, 0, 0.7);
                    display: flex;
                    justify-content: center;
                    align-items: center;
                }
                .modal-content {
                    background: var(--bg-secondary, #1e1e1e);
                    border-radius: 12px;
                    padding: 24px;
                    border: 1px solid rgba(255, 255, 255, 0.1);
                    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
                }
                .modal-header {
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    margin-bottom: 20px;
                }
                .modal-title {
                    margin: 0;
                    font-size: 1.4rem;
                }
                .modal-close {
                    background: none;
                    border: none;
                    color: white;
                    font-size: 2rem;
                    cursor: pointer;
                    line-height: 1;
                    padding: 0;
                }
                .modal-footer {
                    display: flex;
                    justify-content: flex-end;
                    gap: 12px;
                    margin-top: 10px;
                }
                .update-notes::-webkit-scrollbar {
                    width: 6px;
                }
                .update-notes::-webkit-scrollbar-thumb {
                    background: rgba(255, 255, 255, 0.2);
                    border-radius: 3px;
                }
            ` }} />
        </div>
    );
};
