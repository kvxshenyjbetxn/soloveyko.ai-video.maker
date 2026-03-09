import React, { useState, useEffect } from 'react';
import { useI18n } from '../contexts/I18nContext';
import { useTheme } from '../contexts/ThemeContext';
import { utils } from '../../wailsjs/go/models';

interface UpdateModalProps {
    isOpen: boolean;
    manifest: utils.UpdateManifest;
    onClose: () => void;
}

export const UpdateModal: React.FC<UpdateModalProps> = ({ isOpen, manifest, onClose }) => {
    const { locale } = useI18n();
    const { accentColor } = useTheme();
    const [isDownloading, setIsDownloading] = useState(false);
    const [isDownloaded, setIsDownloaded] = useState(false);
    const [downloadedPath, setDownloadedPath] = useState("");
    const [os, setOs] = useState("");
    const [progress, setProgress] = useState(0);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        if (!isOpen) {
            setIsDownloading(false);
            setIsDownloaded(false);
            setProgress(0);
            setError(null);
        } else {
            // @ts-ignore
            if (window.go && window.go.main && window.go.main.App && window.go.main.App.GetOS) {
                // @ts-ignore
                window.go.main.App.GetOS().then(setOs);
            }
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
            const pkgPath = await window.go.main.App.DownloadUpdate(manifest.url);
            if (pkgPath) {
                if (os === 'darwin') {
                    setIsDownloading(false);
                    setIsDownloaded(true);
                    setDownloadedPath(pkgPath);
                } else {
                    // @ts-ignore
                    await window.go.main.App.ApplyUpdate(pkgPath);
                }
            }
        } catch (e: any) {
            console.error("Update failed:", e);
            setError(e.toString());
            setIsDownloading(false);
        }
    };

    const handleOpenFolder = () => {
        if (!downloadedPath) return;
        // Strip filename to get directory
        const dir = downloadedPath.substring(0, downloadedPath.lastIndexOf(os === 'windows' ? '\\' : '/'));
        // @ts-ignore
        if (window.go && window.go.main && window.go.main.App && window.go.main.App.OpenPath) {
            // @ts-ignore
            window.go.main.App.OpenPath(dir || downloadedPath);
        }
    };

    const getTitle = () => {
        if (isDownloaded) {
            if (locale === 'uk') return 'Файл завантажено';
            if (locale === 'ru') return 'Файл загружен';
            return 'File Downloaded';
        }
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
        <div className="update-modal-overlay">
            <div className="update-modal-container animate-slide-up">
                <div className="update-modal-glow"></div>

                <div className="update-modal-header">
                    <div className="header-icon-container">
                        <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" className="header-icon">
                            {isDownloaded ? (
                                <path d="M5 13L9 17L19 7" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
                            ) : (
                                <path d="M12 17V7M12 7L8 11M12 7L16 11M21 12C21 16.9706 16.9706 21 12 21C7.02944 21 3 16.9706 3 12C3 7.02944 7.02944 3 12 3C16.9706 3 21 7.02944 21 12Z" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
                            )}
                        </svg>
                    </div>
                    <div className="header-text">
                        <h2 className="modal-title">{getTitle()}</h2>
                        <span className="version-badge">v{manifest.version}</span>
                    </div>
                    {!isDownloading && (
                        <button className="close-button" onClick={onClose}>&times;</button>
                    )}
                </div>

                <div className="update-modal-body">
                    {!isDownloaded ? (
                        <>
                            <div className="notes-label">
                                {locale === 'uk' ? 'Що нового:' : (locale === 'ru' ? 'Что нового:' : 'What\'s new:')}
                            </div>

                            {manifest.notes && (
                                <div className="update-notes-fancy">
                                    {(() => {
                                        const urlRegex = /(https?:\/\/[^\s]+)/g;
                                        const parts = manifest.notes.split(urlRegex);
                                        return parts.map((part, i) => {
                                            if (part.match(urlRegex)) {
                                                return (
                                                    <span
                                                        key={i}
                                                        className="note-link"
                                                        onClick={() => {
                                                            // @ts-ignore
                                                            if (window.runtime) {
                                                                // @ts-ignore
                                                                window.runtime.BrowserOpenURL(part);
                                                            }
                                                        }}
                                                    >
                                                        {part}
                                                    </span>
                                                );
                                            }
                                            return part;
                                        });
                                    })()}
                                </div>
                            )}
                        </>
                    ) : (
                        <div className="download-success-message">
                            <p>
                                {locale === 'uk' 
                                    ? `Для оновлення треба закрити цю программу та відкрити нову в папці Завантаження (Downloads).` 
                                    : (locale === 'ru' 
                                        ? `Для обновления нужно закрыть эту программу и открыть новую в папке Загрузки (Downloads).` 
                                        : `To update, you need to close this program and open the new one in the Downloads folder.`)}
                            </p>
                            <div className="file-path-display">
                                {downloadedPath}
                            </div>
                        </div>
                    )}

                    {isDownloading && (
                        <div className="download-section">
                            <div className="progress-info">
                                <span>{locale === 'uk' ? 'Завантаження компонентів...' : (locale === 'ru' ? 'Загрузка компонентов...' : 'Downloading components...')}</span>
                                <span className="progress-percent">{progress}%</span>
                            </div>
                            <div className="fancy-progress-bg">
                                <div className="fancy-progress-fill" style={{ width: `${progress}%` }}>
                                    <div className="fill-glow"></div>
                                </div>
                            </div>
                        </div>
                    )}

                    {error && (
                        <div className="error-message">
                            <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" className="error-icon">
                                <path d="M12 8V12M12 16H12.01M21 12C21 16.9706 16.9706 21 12 21C7.02944 21 3 16.9706 3 12C3 7.02944 7.02944 3 12 3C16.9706 3 21 7.02944 21 12Z" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
                            </svg>
                            <span>{error}</span>
                        </div>
                    )}
                </div>

                <div className="update-modal-footer">
                    {!isDownloading && !isDownloaded && (
                        <button className="btn-later" onClick={onClose}>
                            {locale === 'uk' ? 'Пізніше' : (locale === 'ru' ? 'Позже' : 'Later')}
                        </button>
                    )}
                    
                    {isDownloaded ? (
                        <>
                            <button className="btn-later" onClick={handleOpenFolder}>
                                {locale === 'uk' ? 'Відкрити папку' : (locale === 'ru' ? 'Открыть папку' : 'Open Folder')}
                            </button>
                            <button className="btn-update-now" onClick={onClose}>
                                {locale === 'uk' ? 'Закрити' : (locale === 'ru' ? 'Закрыть' : 'Close')}
                            </button>
                        </>
                    ) : (
                        <button
                            className={`btn-update-now ${isDownloading ? 'loading' : ''}`}
                            onClick={handleUpdate}
                            disabled={isDownloading}
                        >
                            {getButtonText()}
                        </button>
                    )}
                </div>
            </div>


            <style dangerouslySetInnerHTML={{
                __html: `
                .update-modal-overlay {
                    position: fixed;
                    top: 0;
                    left: 0;
                    right: 0;
                    bottom: 0;
                    background: rgba(0, 0, 0, 0.85);
                    backdrop-filter: blur(8px);
                    display: flex;
                    justify-content: center;
                    align-items: center;
                    z-index: 99999;
                    font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
                }
                .update-modal-container {
                    position: relative;
                    background: linear-gradient(145deg, #1a1a1a, #0d0d0d);
                    border-radius: 20px;
                    width: 480px;
                    max-width: 90%;
                    padding: 30px;
                    border: 1px solid rgba(255, 255, 255, 0.08);
                    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.7);
                    overflow: hidden;
                }
                .update-modal-glow {
                    position: absolute;
                    top: -100px;
                    right: -100px;
                    width: 250px;
                    height: 250px;
                    background: radial-gradient(circle, ${accentColor}26 0%, transparent 70%);
                    pointer-events: none;
                }
                .update-modal-header {
                    display: flex;
                    align-items: center;
                    margin-bottom: 25px;
                    position: relative;
                }
                .header-icon-container {
                    width: 48px;
                    height: 48px;
                    background: ${accentColor}1a;
                    border-radius: 12px;
                    display: flex;
                    justify-content: center;
                    align-items: center;
                    margin-right: 15px;
                    color: ${accentColor};
                }
                .header-icon {
                    width: 28px;
                    height: 28px;
                }
                .header-text {
                    flex-grow: 1;
                }
                .modal-title {
                    margin: 0;
                    font-size: 1.25rem;
                    font-weight: 700;
                    letter-spacing: -0.02em;
                    color: #fff;
                }
                .version-badge {
                    display: inline-block;
                    padding: 2px 8px;
                    background: rgba(255, 255, 255, 0.05);
                    border: 1px solid rgba(255, 255, 255, 0.1);
                    border-radius: 6px;
                    font-size: 0.75rem;
                    color: rgba(255, 255, 255, 0.6);
                    margin-top: 4px;
                }
                .close-button {
                    background: none;
                    border: none;
                    color: rgba(255, 255, 255, 0.4);
                    font-size: 1.5rem;
                    cursor: pointer;
                    transition: color 0.2s;
                    padding: 5px;
                }
                .close-button:hover {
                    color: #fff;
                }
                .update-modal-body {
                    margin-bottom: 30px;
                }
                .notes-label {
                    font-size: 0.85rem;
                    color: rgba(255, 255, 255, 0.4);
                    margin-bottom: 10px;
                    font-weight: 600;
                    text-transform: uppercase;
                    letter-spacing: 0.05em;
                }
                .update-notes-fancy {
                    background: rgba(255, 255, 255, 0.03);
                    border: 1px solid rgba(255, 255, 255, 0.05);
                    border-radius: 12px;
                    padding: 15px;
                    max-height: 180px;
                    overflow-y: auto;
                    font-size: 0.95rem;
                    line-height: 1.6;
                    color: rgba(255, 255, 255, 0.8);
                    white-space: pre-wrap;
                }
                .download-section {
                    margin-top: 25px;
                }
                .progress-info {
                    display: flex;
                    justify-content: space-between;
                    font-size: 0.85rem;
                    color: rgba(255, 255, 255, 0.6);
                    margin-bottom: 10px;
                }
                .progress-percent {
                    font-weight: 700;
                    color: ${accentColor};
                }
                .fancy-progress-bg {
                    height: 6px;
                    background: rgba(255, 255, 255, 0.05);
                    border-radius: 10px;
                    overflow: hidden;
                }
                .fancy-progress-fill {
                    height: 100%;
                    background: linear-gradient(90deg, ${accentColor}, ${accentColor}cc);
                    border-radius: 10px;
                    transition: width 0.4s cubic-bezier(0.1, 0.7, 0.1, 1);
                    position: relative;
                }
                .fill-glow {
                    position: absolute;
                    top: 0;
                    left: 0;
                    right: 0;
                    bottom: 0;
                    background: linear-gradient(90deg, transparent, rgba(255,255,255,0.3), transparent);
                    animation: shimmer 1.5s infinite;
                }
                @keyframes shimmer {
                    0% { transform: translateX(-100%); }
                    100% { transform: translateX(100%); }
                }
                .error-message {
                    display: flex;
                    align-items: center;
                    background: rgba(255, 77, 77, 0.1);
                    border: 1px solid rgba(255, 77, 77, 0.2);
                    border-radius: 10px;
                    padding: 12px;
                    margin-top: 20px;
                    color: #ff4d4d;
                    font-size: 0.85rem;
                }
                .error-icon {
                    width: 18px;
                    height: 18px;
                    margin-right: 10px;
                    flex-shrink: 0;
                }
                .update-modal-footer {
                    display: flex;
                    justify-content: flex-end;
                    gap: 12px;
                }
                .btn-later {
                    background: transparent;
                    border: 1px solid rgba(255, 255, 255, 0.1);
                    color: rgba(255, 255, 255, 0.8);
                    padding: 12px 24px;
                    border-radius: 12px;
                    font-size: 0.95rem;
                    font-weight: 600;
                    cursor: pointer;
                    transition: all 0.2s;
                }
                .btn-later:hover {
                    background: rgba(255, 255, 255, 0.05);
                    color: #fff;
                }
                .btn-update-now {
                    background: ${accentColor};
                    border: none;
                    color: #fff;
                    padding: 12px 28px;
                    border-radius: 12px;
                    font-size: 0.95rem;
                    font-weight: 600;
                    cursor: pointer;
                    transition: all 0.2s;
                    box-shadow: 0 4px 15px ${accentColor}4d;
                }
                .btn-update-now:hover:not(:disabled) {
                    background: ${accentColor}ee;
                    transform: translateY(-1px);
                    box-shadow: 0 6px 20px ${accentColor}66;
                }
                .btn-update-now:active:not(:disabled) {
                    transform: translateY(0);
                }
                .btn-update-now:disabled {
                    background: rgba(255, 255, 255, 0.05);
                    color: rgba(255, 255, 255, 0.3);
                    cursor: not-allowed;
                    box-shadow: none;
                }
                .animate-slide-up {
                    animation: slideUp 0.4s cubic-bezier(0.16, 1, 0.3, 1);
                }
                @keyframes slideUp {
                    from { transform: translateY(20px); opacity: 0; }
                    to { transform: translateY(0); opacity: 1; }
                }
                .update-notes-fancy::-webkit-scrollbar {
                    width: 6px;
                }
                .update-notes-fancy::-webkit-scrollbar-thumb {
                    background: rgba(255, 255, 255, 0.1);
                    border-radius: 10px;
                }
                .note-link {
                    color: ${accentColor};
                    text-decoration: none;
                    cursor: pointer;
                    transition: all 0.2s;
                    border-bottom: 1px dashed ${accentColor}4d;
                }
                .note-link:hover {
                    filter: brightness(1.2);
                    border-bottom: 1px solid ${accentColor};
                    text-shadow: 0 0 10px ${accentColor}66;
                }
                .download-success-message {
                    color: rgba(255, 255, 255, 0.9);
                    font-size: 1rem;
                    line-height: 1.5;
                    text-align: center;
                    margin: 10px 0;
                }
                .file-path-display {
                    background: rgba(0, 0, 0, 0.3);
                    border: 1px dashed rgba(255, 255, 255, 0.2);
                    border-radius: 8px;
                    padding: 10px;
                    margin-top: 15px;
                    font-family: monospace;
                    font-size: 0.8rem;
                    color: rgba(255, 255, 255, 0.5);
                    word-break: break-all;
                }
            ` }} />
        </div>
    );
};
