import React, { useState, useEffect } from 'react';
import { useI18n } from '../../../../contexts/I18nContext';
import { useTheme } from '../../../../contexts/ThemeContext';
import { useServices } from '../../../../contexts/ServiceContext';
// @ts-ignore
import { GetGooglerAPIKey, SaveGooglerAPIKey, SaveGooglerVideoAlertThreshold, SaveGooglerImageAlertThreshold, SaveGooglerMaxImageConnections, SaveGooglerMaxVideoConnections } from '../../../../../wailsjs/go/main/App';
import '../../general.css';

export const Googler = () => {
    const { t } = useI18n();
    const { accentColor } = useTheme();
    const {
        googlerUsage,
        refreshGooglerUsage,
        loadingGoogler,
        googlerVideoThreshold,
        setGooglerVideoThreshold,
        googlerImageThreshold,
        setGooglerImageThreshold,
        googlerMaxImages,
        setGooglerMaxImages,
        googlerMaxVideos,
        setGooglerMaxVideos
    } = useServices();

    const [apiKey, setApiKey] = useState('');
    const [videoThreshold, setVideoThreshold] = useState<string>('0');
    const [imageThreshold, setImageThreshold] = useState<string>('0');
    const [isLoaded, setIsLoaded] = useState(false);
    const [statusMsg, setStatusMsg] = useState<{ type: 'success' | 'error', text: string } | null>(null);

    useEffect(() => {
        const loadKey = async () => {
            const key = await GetGooglerAPIKey();
            setApiKey(key || '');
            setVideoThreshold(googlerVideoThreshold.toString());
            setImageThreshold(googlerImageThreshold.toString());
            setIsLoaded(true);
        };
        loadKey();
    }, [googlerVideoThreshold, googlerImageThreshold]);

    useEffect(() => {
        if (!isLoaded) return;
        const timer = setTimeout(() => {
            SaveGooglerAPIKey(apiKey);

            const numVideoThreshold = parseFloat(videoThreshold) || 0;
            if (numVideoThreshold !== googlerVideoThreshold) {
                SaveGooglerVideoAlertThreshold(numVideoThreshold);
                setGooglerVideoThreshold(numVideoThreshold);
            }

            const numImageThreshold = parseFloat(imageThreshold) || 0;
            if (numImageThreshold !== googlerImageThreshold) {
                SaveGooglerImageAlertThreshold(numImageThreshold);
                setGooglerImageThreshold(numImageThreshold);
            }
        }, 1000);
        return () => clearTimeout(timer);
    }, [apiKey, videoThreshold, imageThreshold, isLoaded]);

    const handleCheckUsage = async () => {
        setStatusMsg(null);
        if (!apiKey) return;
        await SaveGooglerAPIKey(apiKey);
        try {
            await refreshGooglerUsage();
            setStatusMsg({ type: 'success', text: t('image.success') || 'Updated' });
            setTimeout(() => setStatusMsg(null), 3000);
        } catch (err: any) {
            setStatusMsg({ type: 'error', text: err?.message || 'Error' });
        }
    };

    const formatDate = (timestamp: number) => {
        if (!timestamp) return '---';
        return new Date(timestamp * 1000).toLocaleString();
    };

    return (
        <div className="content-wrapper animate-fade premium-scrollbar" style={{ overflowY: 'auto', paddingRight: '10px' }}>
            <div className="settings-container" style={{ maxWidth: '1000px' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '30px' }}>
                    <h2 className="settings-title" style={{ margin: 0 }}>{t('image.googler')}</h2>
                    <div style={{ display: 'flex', gap: '10px', alignItems: 'center' }}>
                        <div style={{
                            padding: '4px 12px',
                            borderRadius: '20px',
                            background: googlerUsage.expiration_date > (Date.now() / 1000) ? 'rgba(76, 175, 80, 0.1)' : 'rgba(255, 82, 82, 0.1)',
                            color: googlerUsage.expiration_date > (Date.now() / 1000) ? '#4caf50' : '#ff5252',
                            fontSize: '0.85em',
                            fontWeight: '600',
                            border: `1px solid ${googlerUsage.expiration_date > (Date.now() / 1000) ? 'rgba(76, 175, 80, 0.2)' : 'rgba(255, 82, 82, 0.2)'}`
                        }}>
                            {googlerUsage.expiration_date > (Date.now() / 1000) ? 'ACTIVE' : 'INACTIVE'}
                        </div>
                    </div>
                </div>

                <div className="settings-section glass-panel" style={{ padding: '25px', borderRadius: '12px', background: 'rgba(255, 255, 255, 0.03)', border: '1px solid rgba(255, 255, 255, 0.05)', marginBottom: '30px' }}>
                    <h3 className="section-title" style={{ marginBottom: '20px', fontSize: '1.1em', opacity: 0.9 }}>{t('settings.voice.apiKey') || 'API Connection'}</h3>
                    <div style={{ display: 'flex', gap: '12px' }}>
                        <input
                            type="password"
                            className="premium-input"
                            style={{
                                flex: 1,
                                padding: '12px 16px',
                                borderRadius: '8px',
                                border: '1px solid rgba(255, 255, 255, 0.08)',
                                background: 'rgba(0, 0, 0, 0.3)',
                                color: '#fff',
                                outline: 'none',
                                fontSize: '0.95em'
                            }}
                            value={apiKey}
                            onChange={(e) => {
                                setApiKey(e.target.value);
                                setStatusMsg(null);
                            }}
                            placeholder="Enter your X-API-KEY here..."
                        />
                        <button
                            onClick={handleCheckUsage}
                            disabled={loadingGoogler || !apiKey}
                            style={{
                                padding: '12px 24px',
                                borderRadius: '8px',
                                background: accentColor,
                                border: 'none',
                                color: '#fff',
                                cursor: 'pointer',
                                fontWeight: '600',
                                display: 'flex',
                                alignItems: 'center',
                                gap: '8px',
                                transition: 'all 0.2s ease',
                                opacity: (loadingGoogler || !apiKey) ? 0.5 : 1,
                                boxShadow: `0 4px 15px ${accentColor}33`
                            }}
                        >
                            {loadingGoogler ? (
                                <div className="spinner-small" style={{ width: '16px', height: '16px', border: '2px solid rgba(255,255,255,0.3)', borderTopColor: '#fff', borderRadius: '50%', animation: 'spin 0.8s linear infinite' }}></div>
                            ) : (
                                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><path d="M21 2v6h-6"></path><path d="M3 12a9 9 0 0 1 15-6.7L21 8"></path><path d="M3 22v-6h6"></path><path d="M21 12a9 9 0 0 1-15 6.7L3 16"></path></svg>
                            )}
                            {t('settings.voice.fetchBalance') || 'Sync Data'}
                        </button>
                    </div>
                    {statusMsg && (
                        <div style={{ marginTop: '10px', color: statusMsg.type === 'success' ? '#4caf50' : '#ff5252', fontSize: '0.85em', textAlign: 'right', fontWeight: '500' }}>
                            {statusMsg.text}
                        </div>
                    )}
                </div>

                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '25px' }}>
                    {/* Active Threads Card */}
                    <div className="stat-group glass-panel" style={{ background: 'rgba(255, 255, 255, 0.02)', borderRadius: '12px', padding: '20px', border: '1px solid rgba(255, 255, 255, 0.05)' }}>
                        <h4 style={{ margin: '0 0 20px 0', opacity: 0.6, fontSize: '0.85em', textTransform: 'uppercase', letterSpacing: '1px' }}>Current Active Threads</h4>
                        <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
                            <div>
                                <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '8px', fontSize: '0.9em' }}>
                                    <span>Video Generation</span>
                                    <span style={{ fontWeight: '600', color: accentColor }}>{googlerUsage.current_usage.active_threads.video_threads} / {googlerUsage.account_limits.video_generation_threads_allowed}</span>
                                </div>
                                <div style={{ height: '6px', background: 'rgba(255,255,255,0.05)', borderRadius: '3px', overflow: 'hidden' }}>
                                    <div style={{
                                        height: '100%',
                                        width: `${Math.min(100, (googlerUsage.current_usage.active_threads.video_threads / (googlerUsage.account_limits.video_generation_threads_allowed || 1)) * 100)}%`,
                                        background: accentColor,
                                        boxShadow: `0 0 10px ${accentColor}66`
                                    }}></div>
                                </div>
                            </div>
                            <div>
                                <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '8px', fontSize: '0.9em' }}>
                                    <span>Image Generation</span>
                                    <span style={{ fontWeight: '600' }}>{googlerUsage.current_usage.active_threads.image_threads} / {googlerUsage.account_limits.img_generation_threads_allowed}</span>
                                </div>
                                <div style={{ height: '6px', background: 'rgba(255,255,255,0.05)', borderRadius: '3px', overflow: 'hidden' }}>
                                    <div style={{
                                        height: '100%',
                                        width: `${Math.min(100, (googlerUsage.current_usage.active_threads.image_threads / (googlerUsage.account_limits.img_generation_threads_allowed || 1)) * 100)}%`,
                                        background: '#fff',
                                        opacity: 0.8
                                    }}></div>
                                </div>
                            </div>
                        </div>
                    </div>

                    {/* Hourly Limits Card */}
                    <div className="stat-group glass-panel" style={{ background: 'rgba(255, 255, 255, 0.02)', borderRadius: '12px', padding: '20px', border: '1px solid rgba(255, 255, 255, 0.05)' }}>
                        <h4 style={{ margin: '0 0 20px 0', opacity: 0.6, fontSize: '0.85em', textTransform: 'uppercase', letterSpacing: '1px' }}>Hourly Quotas ({googlerUsage.usage_window || 'per hour'})</h4>
                        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '15px' }}>
                            <div style={{ background: 'rgba(255,255,255,0.03)', padding: '12px', borderRadius: '8px' }}>
                                <div style={{ opacity: 0.5, fontSize: '0.75em', marginBottom: '4px' }}>Video Units (Hr)</div>
                                <div style={{ fontSize: '1.2em', fontWeight: 'bold' }}>
                                    {googlerUsage.current_usage.hourly_usage.video_generation || 0} / {googlerUsage.account_limits.video_gen_per_hour_limit.toLocaleString()}
                                </div>
                            </div>
                            <div style={{ background: 'rgba(255,255,255,0.03)', padding: '12px', borderRadius: '8px' }}>
                                <div style={{ opacity: 0.5, fontSize: '0.75em', marginBottom: '4px' }}>Image Units (Hr)</div>
                                <div style={{ fontSize: '1.2em', fontWeight: 'bold' }}>
                                    {googlerUsage.current_usage.hourly_usage.image_generation || 0} / {googlerUsage.account_limits.img_gen_per_hour_limit.toLocaleString()}
                                </div>
                            </div>
                            <div style={{ background: 'rgba(255,255,255,0.03)', padding: '12px', borderRadius: '8px', gridColumn: 'span 2' }}>
                                <div style={{ opacity: 0.5, fontSize: '0.75em', marginBottom: '4px' }}>Prompt Tokens</div>
                                <div style={{ fontSize: '1.2em', fontWeight: 'bold', color: '#FFC107' }}>{googlerUsage.account_limits.prompt_tokens_per_hour_limit?.toLocaleString() || 0}</div>
                            </div>
                        </div>
                    </div>
                </div>

                <div className="settings-section glass-panel" style={{ padding: '25px', borderRadius: '12px', background: 'rgba(255, 255, 255, 0.03)', border: '1px solid rgba(255, 255, 255, 0.05)', marginTop: '25px' }}>
                    <h3 className="section-title" style={{ marginBottom: '20px', fontSize: '1.1em', opacity: 0.9 }}>{t('general.googlerLimits')}</h3>
                    <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '25px' }}>
                        <div>
                            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
                                <span style={{ fontSize: '0.9em', opacity: 0.7 }}>{t('general.googlerMaxImages')}</span>
                                <span style={{ fontSize: '0.95em', fontWeight: 'bold', color: accentColor }}>{googlerMaxImages} / 25</span>
                            </div>
                            <input
                                type="range"
                                min="1"
                                max="25"
                                step="1"
                                className="premium-range"
                                style={{
                                    width: '100%',
                                    cursor: 'pointer',
                                    background: `linear-gradient(to right, ${accentColor} ${((googlerMaxImages - 1) / 24) * 100}%, rgba(255,255,255,0.05) ${((googlerMaxImages - 1) / 24) * 100}%)`
                                }}
                                value={googlerMaxImages}
                                onChange={(e) => {
                                    const val = parseInt(e.target.value);
                                    setGooglerMaxImages(val);
                                    SaveGooglerMaxImageConnections(val);
                                }}
                            />
                        </div>
                        <div>
                            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
                                <span style={{ fontSize: '0.9em', opacity: 0.7 }}>{t('general.googlerMaxVideos')}</span>
                                <span style={{ fontSize: '0.95em', fontWeight: 'bold', color: accentColor }}>{googlerMaxVideos} / 10</span>
                            </div>
                            <input
                                type="range"
                                min="1"
                                max="10"
                                step="1"
                                className="premium-range"
                                style={{
                                    width: '100%',
                                    cursor: 'pointer',
                                    background: `linear-gradient(to right, ${accentColor} ${((googlerMaxVideos - 1) / 9) * 100}%, rgba(255,255,255,0.05) ${((googlerMaxVideos - 1) / 9) * 100}%)`
                                }}
                                value={googlerMaxVideos}
                                onChange={(e) => {
                                    const val = parseInt(e.target.value);
                                    setGooglerMaxVideos(val);
                                    SaveGooglerMaxVideoConnections(val);
                                }}
                            />
                        </div>
                    </div>
                </div>

                <div className="settings-section glass-panel" style={{ padding: '25px', borderRadius: '12px', background: 'rgba(255, 255, 255, 0.03)', border: '1px solid rgba(255, 255, 255, 0.05)', marginTop: '25px' }}>
                    <h3 className="section-title" style={{ marginBottom: '20px', fontSize: '1.1em', opacity: 0.9 }}>{t('api.googlerSettings.videoAlertThreshold')} & {t('api.googlerSettings.imageAlertThreshold')}</h3>
                    <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '15px' }}>
                        <div>
                            <span style={{ fontSize: '0.8em', opacity: 0.5, display: 'block', marginBottom: '8px' }}>{t('api.googlerSettings.videoAlertThreshold')}</span>
                            <input
                                type="number"
                                className="premium-input"
                                style={{
                                    width: '100%',
                                    padding: '12px 16px',
                                    borderRadius: '8px',
                                    border: '1px solid rgba(255, 255, 255, 0.08)',
                                    background: 'rgba(0, 0, 0, 0.3)',
                                    color: '#fff',
                                    outline: 'none',
                                    fontSize: '0.95em'
                                }}
                                value={videoThreshold}
                                onChange={(e) => setVideoThreshold(e.target.value)}
                                placeholder={t('api.googlerSettings.videoThresholdPlaceholder')}
                            />
                        </div>
                        <div>
                            <span style={{ fontSize: '0.8em', opacity: 0.5, display: 'block', marginBottom: '8px' }}>{t('api.googlerSettings.imageAlertThreshold')}</span>
                            <input
                                type="number"
                                className="premium-input"
                                style={{
                                    width: '100%',
                                    padding: '12px 16px',
                                    borderRadius: '8px',
                                    border: '1px solid rgba(255, 255, 255, 0.08)',
                                    background: 'rgba(0, 0, 0, 0.3)',
                                    color: '#fff',
                                    outline: 'none',
                                    fontSize: '0.95em'
                                }}
                                value={imageThreshold}
                                onChange={(e) => setImageThreshold(e.target.value)}
                                placeholder={t('api.googlerSettings.imageThresholdPlaceholder')}
                            />
                        </div>
                    </div>
                </div>

                <div className="glass-panel" style={{ marginTop: '25px', padding: '15px 25px', borderRadius: '12px', background: 'rgba(255, 255, 255, 0.02)', border: '1px solid rgba(255, 255, 255, 0.05)', display: 'flex', justifyContent: 'space-between', fontSize: '0.85em' }}>
                    <div style={{ display: 'flex', gap: '30px' }}>
                        <div><span style={{ opacity: 0.5 }}>Activated:</span> <span style={{ marginLeft: '8px' }}>{formatDate(googlerUsage.activation_date)}</span></div>
                        <div><span style={{ opacity: 0.5 }}>Expires:</span> <span style={{ marginLeft: '8px', color: googlerUsage.expiration_date > (Date.now() / 1000) ? '#4caf50' : '#ff5252' }}>{formatDate(googlerUsage.expiration_date)}</span></div>
                    </div>
                    <div><span style={{ opacity: 0.3 }}>v3 Fast-Gen.AI Integration</span></div>
                </div>
            </div>

            <style>{`
                @keyframes spin { to { transform: rotate(360deg); } }
                .glass-panel { backdrop-filter: blur(10px); }
                .premium-input:focus { border-color: ${accentColor} !important; border-opacity: 0.3 !important; }
                .premium-scrollbar::-webkit-scrollbar { width: 6px; }
                .premium-scrollbar::-webkit-scrollbar-track { background: rgba(0,0,0,0.2); }
                .premium-scrollbar::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.1); border-radius: 10px; }
                .premium-scrollbar::-webkit-scrollbar-thumb:hover { background: ${accentColor}; }
                
                .premium-range {
                    -webkit-appearance: none;
                    appearance: none;
                    height: 6px;
                    border-radius: 3px;
                    outline: none;
                    margin: 10px 0;
                }
                .premium-range::-webkit-slider-thumb {
                    -webkit-appearance: none;
                    appearance: none;
                    width: 18px;
                    height: 18px;
                    border-radius: 50%;
                    background: ${accentColor};
                    cursor: pointer;
                    border: 3px solid #1a1a1a;
                    box-shadow: 0 0 10px ${accentColor}66;
                    transition: transform 0.1s ease;
                }
                .premium-range::-webkit-slider-thumb:hover {
                    transform: scale(1.15);
                }
                .premium-range::-moz-range-thumb {
                    width: 18px;
                    height: 18px;
                    border-radius: 50%;
                    background: ${accentColor};
                    cursor: pointer;
                    border: 3px solid #1a1a1a;
                    box-shadow: 0 0 10px ${accentColor}66;
                }
            `}</style>
        </div>
    );
};
