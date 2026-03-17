import React, { useState, useEffect, useRef, useCallback } from 'react';
import './Preview.css';
import { useI18n } from '../../contexts/I18nContext';
import { useToast } from '../../contexts/ToastContext';
// @ts-ignore
import { GeneratePreview, GetPipelineSettings, GetPreviewPath } from '../../../wailsjs/go/main/App';
// @ts-ignore
import { EventsOn } from '../../../wailsjs/runtime/runtime';
import { SubtitleSection } from '../../components/pipeline-sidebar/SubtitleSection';
import { MontageSection } from '../../components/pipeline-sidebar/MontageSection';

export const Preview: React.FC = () => {
    const { t } = useI18n();
    const { showToast } = useToast();
    const [settings, setSettings] = useState<any>(null);
    const [loading, setLoading] = useState(false);
    const [progressMessage, setProgressMessage] = useState<string>('');
    const [videoUrl, setVideoUrl] = useState<string | null>(null);
    const [previewDir, setPreviewDir] = useState<string>("");
    
    // Video Player State
    const videoRef = useRef<HTMLVideoElement>(null);
    const [isPlaying, setIsPlaying] = useState(false);
    const [currentTime, setCurrentTime] = useState(0);
    const [duration, setDuration] = useState(0);
    const [maxAudioDuration, setMaxAudioDuration] = useState(0);

    useEffect(() => {
        const handleStatus = (tid: string, stage: string, status: string, msg: string) => {
            console.log("Preview handleStatus:", tid, stage, status, msg);
            if (tid === 'preview_task' || tid === 'Preview') {
                const stageKey = stage === 'subtitle' ? 'subtitles' : stage;
                const baseMsg = t(`stages.${stageKey}`) || stage;
                
                if (stage === 'montage') {
                    setProgressMessage(`${baseMsg}: ${msg || '0%'}`);
                } else {
                    setProgressMessage(baseMsg);
                }
            }
        };

        const unregister = EventsOn('stageStatus', handleStatus);
        return () => {
            if (unregister) unregister();
        };
    }, []);

    useEffect(() => {
        const init = async () => {
            try {
                const s = await GetPipelineSettings() as any;
                // @ts-ignore
                const { GetPreviewAudioDuration } = await import('../../../wailsjs/go/main/App');
                const audioDur = await GetPreviewAudioDuration();
                setMaxAudioDuration(audioDur);

                if (s.previewLimitSeconds === undefined || s.previewLimitSeconds === 10) {
                     s.previewLimitSeconds = audioDur > 0 ? Math.floor(audioDur) : 10;
                }
                
                if (s.previewImageMax === undefined) s.previewImageMax = 5;
                if (s.previewVideoMax === undefined) s.previewVideoMax = 5;
                
                setSettings(s);
                const path = await GetPreviewPath();
                setPreviewDir(path);

                if (s.previewLimitSeconds > audioDur && audioDur > 0) {
                    s.previewLimitSeconds = Math.floor(audioDur);
                }
            } catch (err) {
                console.error("Failed to load settings:", err);
            }
        };
        init();
    }, []);

    const handleChange = (field: string, value: any) => {
        setSettings((prev: any) => ({
            ...prev,
            [field]: value
        }));
    };

    const handleGeneratePreview = async () => {
        setLoading(true);
        setProgressMessage('');
        try {
            const finalPath = await GeneratePreview(settings);
            // Convert file path to local URL for Wails FileLoader
            // Replace backslashes with forward slashes for URL compatibility
            const cleanPath = finalPath.replace(/\\/g, '/');
            // Use encodeURI to handle spaces and special characters while keeping / and :
            const url = `/local/${encodeURI(cleanPath)}?t=${Date.now()}`;
            setVideoUrl(url);
            showToast(t('common.success') || 'Success', 'success');
        } catch (err: any) {
            console.error(err);
            const msg = typeof err === 'string' ? err : (err.message || 'Failed to generate preview');
            showToast(msg, 'error');
        } finally {
            setLoading(false);
        }
    };

    const togglePlay = () => {
        if (videoRef.current) {
            if (isPlaying) {
                videoRef.current.pause();
            } else {
                videoRef.current.play();
            }
            setIsPlaying(!isPlaying);
        }
    };

    const handleTimeUpdate = () => {
        if (videoRef.current) {
            setCurrentTime(videoRef.current.currentTime);
        }
    };

    const handleLoadedMetadata = () => {
        if (videoRef.current) {
            setDuration(videoRef.current.duration);
        }
    };

    const handleSeek = (e: React.ChangeEvent<HTMLInputElement>) => {
        const time = parseFloat(e.target.value);
        if (videoRef.current) {
            videoRef.current.currentTime = time;
            setCurrentTime(time);
        }
    };

    const formatTime = (time: number) => {
        const minutes = Math.floor(time / 60);
        const seconds = Math.floor(time % 60);
        return `${minutes}:${seconds.toString().padStart(2, '0')}`;
    };

    if (!settings) return <div className="loading-overlay"><div className="spinner"></div></div>;

    return (
        <div className="preview-container">
            <aside className="preview-sidebar">
                <div className="sidebar-header">
                    <h2>{t('previewTab.title')}</h2>
                    <p>{t('previewTab.description')}</p>
                </div>

                <div className="preview-settings-scroll">
                    <SubtitleSection 
                        settings={settings} 
                        handleChange={handleChange} 
                        setSettings={setSettings}
                        isCollapsed={false}
                    />
                    <div style={{ height: '20px' }}></div>
                    <MontageSection 
                        settings={settings} 
                        handleChange={handleChange} 
                        setSettings={setSettings}
                        isCollapsed={false}
                    />
                    
                    <div className="preview-limit-section">
                        <h3>{t('previewTab.limits_title') || 'Preview limits'}</h3>
                        
                        <div className="setting-item">
                            <label>
                                {t('previewTab.duration_limit') || 'Duration (sec)'}: {settings.previewLimitSeconds}s
                            </label>
                            <input 
                                type="range" 
                                min="5" 
                                max={maxAudioDuration > 0 ? Math.floor(maxAudioDuration) : 60} 
                                value={settings.previewLimitSeconds} 
                                onChange={(e) => handleChange('previewLimitSeconds', parseInt(e.target.value))}
                            />
                        </div>

                        <div className="setting-grid">
                            <div className="setting-item">
                                <label>{t('previewTab.images_limit') || 'Images'}: {settings.previewImageMax}</label>
                                <input 
                                    type="number" 
                                    min="0" 
                                    max="10" 
                                    value={settings.previewImageMax} 
                                    onChange={(e) => handleChange('previewImageMax', parseInt(e.target.value))}
                                />
                            </div>
                            <div className="setting-item">
                                <label>{t('previewTab.videos_limit') || 'Videos'}: {settings.previewVideoMax}</label>
                                <input 
                                    type="number" 
                                    min="0" 
                                    max="10" 
                                    value={settings.previewVideoMax} 
                                    onChange={(e) => handleChange('previewVideoMax', parseInt(e.target.value))}
                                />
                            </div>
                        </div>
                    </div>
                </div>

                <div className="preview-actions">
                    <button 
                        className="btn-render-preview" 
                        onClick={handleGeneratePreview}
                        disabled={loading}
                    >
                        {loading ? (
                            <><div className="spinner-tiny"></div> {t('common.processing') || 'Processing...'}</>
                        ) : (
                            <><svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="23 4 23 10 17 10"></polyline><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"></path></svg> {t('common.ok') || 'OK'} - {t('other.preview')}</>
                        )}
                    </button>
                    
                    <div className="preview-source-hint">
                        Source: <code>{previewDir}</code>
                    </div>
                </div>
            </aside>

            <main className="preview-main">
                <div className="video-player-wrapper">
                    {loading && (
                        <div className="loading-overlay">
                            <div className="spinner"></div>
                            <span className="progress-text">{progressMessage || t('pipeline.stage.rendering') || 'Rendering...'}</span>
                        </div>
                    )}
                    
                    {videoUrl ? (
                        <>
                            <video 
                                key={videoUrl}
                                ref={videoRef}
                                className="preview-video"
                                onTimeUpdate={handleTimeUpdate}
                                onLoadedMetadata={handleLoadedMetadata}
                                onEnded={() => {
                                    if (videoRef.current) {
                                        videoRef.current.currentTime = 0;
                                        videoRef.current.play();
                                    }
                                }}
                                onClick={togglePlay}
                                autoPlay
                                loop
                            >
                                <source src={videoUrl} type="video/mp4" />
                                Your browser does not support the video tag.
                            </video>
                            
                            <div className="video-controls">
                                <input 
                                    type="range" 
                                    className="seek-bar"
                                    min="0"
                                    max={duration}
                                    step="0.1"
                                    value={currentTime}
                                    onChange={handleSeek}
                                />
                                <div className="controls-row">
                                    <button className="play-pause-btn" onClick={togglePlay}>
                                        {isPlaying ? (
                                            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="4" width="4" height="16" /><rect x="14" y="4" width="4" height="16" /></svg>
                                        ) : (
                                            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z" /></svg>
                                        )}
                                    </button>
                                    <div className="time-display">
                                        {formatTime(currentTime)} / {formatTime(duration)}
                                    </div>
                                </div>
                            </div>
                        </>
                    ) : (
                        <div className="no-video-placeholder">
                            <svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1" strokeLinecap="round" strokeLinejoin="round"><rect x="2" y="2" width="20" height="20" rx="2.18" ry="2.18"/><line x1="7" y1="2" x2="7" y2="22"/><line x1="17" y1="2" x2="17" y2="22"/><line x1="2" y1="12" x2="22" y2="12"/><line x1="2" y1="7" x2="7" y2="7"/><line x1="2" y1="17" x2="7" y2="17"/><line x1="17" y1="17" x2="22" y2="17"/><line x1="17" y1="7" x2="22" y2="7"/></svg>
                            <p>{t('previewTab.no_video')}</p>
                        </div>
                    )}
                </div>
                
                <div className="preview-hint-wrapper">
                    <p>{t('previewTab.hint')}</p>
                </div>
            </main>
        </div>
    );
};
