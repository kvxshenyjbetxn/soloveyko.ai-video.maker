import { useState, useEffect } from 'react';
import { useI18n } from '../../contexts/I18nContext';
import {
    GetSubtitleMaxConnections,
    SaveSubtitleMaxConnections,
    GetMontageMaxConnections,
    SaveMontageMaxConnections,
    GetMontageMode,
    SaveMontageMode,
    GetPipelineSettings,
    SavePipelineSettings
} from '../../../wailsjs/go/main/App';
import './general.css';

export const Performance = () => {
    const { t } = useI18n();
    const [subtitleMax, setSubtitleMax] = useState(2);
    const [montageMax, setMontageMax] = useState(1);
    const [montageMode, setMontageMode] = useState('standard');
    const [montageCodec, setMontageCodec] = useState('cpu');
    const [montagePriority, setMontagePriority] = useState('normal');
    const [montageCores, setMontageCores] = useState(0);
    const [isSaving, setIsSaving] = useState(false);
    const totalCores = typeof navigator !== 'undefined' ? (navigator.hardwareConcurrency || 8) : 8;

    useEffect(() => {
        Promise.all([
            GetSubtitleMaxConnections(),
            GetMontageMaxConnections(),
            GetMontageMode(),
            GetPipelineSettings()
        ]).then(([max, mmax, mode, ps]) => {
            setSubtitleMax(max || 2);
            setMontageMax(mmax || 1);
            setMontageMode(mode || 'standard');
            setMontageCodec((ps as any)?.montageVideoCodec || 'cpu');
            setMontagePriority((ps as any)?.montageProcessPriority || 'normal');
            setMontageCores((ps as any)?.montageCPUCores || 0);
        });
    }, []);

    const handleSubtitleMaxChange = async (val: number) => {
        setSubtitleMax(val);
        setIsSaving(true);
        try { await SaveSubtitleMaxConnections(val); }
        catch (err) { console.error(err); }
        finally { setIsSaving(false); }
    };

    const handleMontageMaxChange = async (val: number) => {
        setMontageMax(val);
        setIsSaving(true);
        try { await SaveMontageMaxConnections(val); }
        catch (err) { console.error(err); }
        finally { setIsSaving(false); }
    };

    const handleMontageModeChange = async (mode: string) => {
        setMontageMode(mode);
        setIsSaving(true);
        try { await SaveMontageMode(mode); }
        catch (err) { console.error(err); }
        finally { setIsSaving(false); }
    };

    const savePipelineField = async (field: string, value: any) => {
        setIsSaving(true);
        try {
            const ps = await GetPipelineSettings();
            await SavePipelineSettings({ ...(ps as any), [field]: value });
        } catch (err) { console.error(err); }
        finally { setIsSaving(false); }
    };

    const subtitleProgress = ((subtitleMax - 1) / 4) * 100;

    const btnStyle = (active: boolean): React.CSSProperties => ({
        padding: '8px 14px', borderRadius: '6px', fontSize: '12px', fontWeight: 600,
        cursor: 'pointer', transition: 'all 0.2s ease', border: 'none',
        background: active ? 'var(--accent-primary)' : 'transparent',
        color: active ? 'white' : 'var(--text-secondary)',
        boxShadow: active ? '0 2px 8px rgba(255,0,195,0.2)' : 'none'
    });

    return (
        <div className="content-wrapper animate-fade">
            <div className="settings-container">
                <div className="settings-section">
                    <h3 className="section-title">{t('settings.performance')}</h3>
                    <p className="section-description">{t('performanceTab.description')}</p>
                </div>

                {/* Subtitle Block */}
                <div className="settings-section">
                    <h4 className="section-title" style={{ fontSize: '12px', opacity: 0.6, textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '16px' }}>
                        {t('performanceTab.subtitle_block')}
                    </h4>
                    <div style={{ background: 'rgba(255,255,255,0.02)', border: '1px solid var(--border-color)', borderRadius: '12px', padding: '20px' }}>
                        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
                            <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                                <span style={{ fontSize: '14px', fontWeight: 600 }}>{t('performanceTab.subtitle_max_concurrency')}</span>
                                <span style={{ fontSize: '12px', color: 'var(--text-tertiary)', maxWidth: '400px', lineHeight: '1.4' }}>{t('performanceTab.subtitle_max_concurrency_desc')}</span>
                            </div>
                            <div style={{ fontSize: '24px', fontWeight: 800, color: 'var(--accent-primary)', fontFamily: 'var(--font-mono)', minWidth: '30px', textAlign: 'right' }}>{subtitleMax}</div>
                        </div>
                        <input type="range" min="1" max="5" className="settings-slider" value={subtitleMax}
                            onChange={(e) => handleSubtitleMaxChange(parseInt(e.target.value))}
                            style={{ width: '100%', margin: 0, '--range-progress': `${subtitleProgress}%` } as React.CSSProperties} />
                        <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: '8px', padding: '0 2px' }}>
                            {[1, 2, 3, 4, 5].map(v => (
                                <span key={v} style={{ fontSize: '10px', color: subtitleMax === v ? 'var(--accent-primary)' : 'var(--text-tertiary)', fontWeight: subtitleMax === v ? 700 : 400, transition: 'all 0.2s ease' }}>{v}</span>
                            ))}
                        </div>
                    </div>
                </div>

                {/* Montage Block */}
                <div className="settings-section">
                    <h4 className="section-title" style={{ fontSize: '12px', opacity: 0.6, textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '16px' }}>
                        {t('performanceTab.montage_block')}
                    </h4>
                    <div style={{ background: 'rgba(255,255,255,0.02)', border: '1px solid var(--border-color)', borderRadius: '12px', padding: '20px', display: 'flex', flexDirection: 'column', gap: '20px' }}>

                        {/* Montage Concurrency Slider */}
                        <div>
                            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
                                <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                                    <span style={{ fontSize: '14px', fontWeight: 600 }}>{t('performanceTab.montage_max_concurrency')}</span>
                                    <span style={{ fontSize: '12px', color: 'var(--text-tertiary)', maxWidth: '400px', lineHeight: '1.4' }}>{t('performanceTab.montage_max_concurrency_desc')}</span>
                                </div>
                                <div style={{ fontSize: '24px', fontWeight: 800, color: 'var(--accent-primary)', fontFamily: 'var(--font-mono)', minWidth: '30px', textAlign: 'right' }}>{montageMax}</div>
                            </div>
                            <input type="range" min="1" max="5" className="settings-slider" value={montageMax}
                                onChange={(e) => handleMontageMaxChange(parseInt(e.target.value))}
                                style={{ width: '100%', margin: 0, '--range-progress': `${((montageMax - 1) / 4) * 100}%` } as React.CSSProperties} />
                            <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: '8px', padding: '0 2px' }}>
                                {[1, 2, 3, 4, 5].map(v => (
                                    <span key={v} style={{ fontSize: '10px', color: montageMax === v ? 'var(--accent-primary)' : 'var(--text-tertiary)', fontWeight: montageMax === v ? 700 : 400, transition: 'all 0.2s ease' }}>{v}</span>
                                ))}
                            </div>
                        </div>

                        <hr style={{ border: 'none', borderTop: '1px solid var(--border-color)', margin: 0, opacity: 0.5 }} />

                        {/* Montage Mode */}
                        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                            <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                                <span style={{ fontSize: '14px', fontWeight: 600 }}>{t('performanceTab.montage_mode')}</span>
                                <span style={{ fontSize: '12px', color: 'var(--text-tertiary)', maxWidth: '400px', lineHeight: '1.4' }}>{t('performanceTab.montage_mode_desc')}</span>
                            </div>
                            <div style={{ display: 'flex', background: 'rgba(0,0,0,0.2)', padding: '4px', borderRadius: '8px', border: '1px solid var(--border-color)' }}>
                                <button onClick={() => handleMontageModeChange('standard')} style={btnStyle(montageMode === 'standard')}>{t('performanceTab.montage_mode_standard')}</button>
                                <button onClick={() => handleMontageModeChange('experimental')} style={btnStyle(montageMode === 'experimental')}>{t('performanceTab.montage_mode_experimental')}</button>
                            </div>
                        </div>

                        {/* Video Codec */}
                        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                            <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                                <span style={{ fontSize: '14px', fontWeight: 600 }}>{t('performanceTab.montage_codec')}</span>
                                <span style={{ fontSize: '12px', color: 'var(--text-tertiary)', maxWidth: '400px', lineHeight: '1.4' }}>{t('performanceTab.montage_codec_desc')}</span>
                            </div>
                            <div style={{ display: 'flex', background: 'rgba(0,0,0,0.2)', padding: '4px', borderRadius: '8px', border: '1px solid var(--border-color)' }}>
                                {(['cpu', 'nvidia', 'amd', 'apple'] as const).map(codec => (
                                    <button key={codec} onClick={() => { setMontageCodec(codec); savePipelineField('montageVideoCodec', codec); }} style={btnStyle(montageCodec === codec)}>
                                        {t(`performanceTab.montage_codec_${codec}`)}
                                    </button>
                                ))}
                            </div>
                        </div>

                        {/* Process Priority */}
                        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                            <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                                <span style={{ fontSize: '14px', fontWeight: 600 }}>{t('performanceTab.montage_priority')}</span>
                                <span style={{ fontSize: '12px', color: 'var(--text-tertiary)', maxWidth: '400px', lineHeight: '1.4' }}>{t('performanceTab.montage_priority_desc')}</span>
                            </div>
                            <div style={{ display: 'flex', background: 'rgba(0,0,0,0.2)', padding: '4px', borderRadius: '8px', border: '1px solid var(--border-color)' }}>
                                {(['idle', 'low', 'normal'] as const).map(p => (
                                    <button key={p} onClick={() => { setMontagePriority(p); savePipelineField('montageProcessPriority', p); }} style={btnStyle(montagePriority === p)}>
                                        {t(`performanceTab.montage_priority_${p}`)}
                                    </button>
                                ))}
                            </div>
                        </div>

                        {/* CPU Cores Affinity */}
                        <div>
                            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '10px' }}>
                                <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                                    <span style={{ fontSize: '14px', fontWeight: 600 }}>{t('performanceTab.montage_cpu_cores')}</span>
                                    <span style={{ fontSize: '12px', color: 'var(--text-tertiary)', maxWidth: '400px', lineHeight: '1.4' }}>{t('performanceTab.montage_cpu_cores_desc')}</span>
                                </div>
                                <div style={{ fontSize: '20px', fontWeight: 800, color: 'var(--accent-primary)', fontFamily: 'var(--font-mono)', minWidth: '60px', textAlign: 'right' }}>
                                    {montageCores === 0 ? t('performanceTab.montage_threads_auto') : `${montageCores}/${totalCores}`}
                                </div>
                            </div>
                            <input type="range" min="0" max={totalCores} step="1" className="settings-slider"
                                value={montageCores}
                                onChange={(e) => {
                                    const v = parseInt(e.target.value);
                                    setMontageCores(v);
                                    savePipelineField('montageCPUCores', v);
                                }}
                                style={{ width: '100%', margin: 0, '--range-progress': `${(montageCores / totalCores) * 100}%` } as React.CSSProperties} />
                            <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: '6px', padding: '0 2px' }}>
                                <span style={{ fontSize: '9px', color: 'var(--text-tertiary)' }}>auto</span>
                                {Array.from({ length: totalCores }, (_, i) => i + 1).filter(v => v % 2 === 0 || v === 1 || v === totalCores).map(v => (
                                    <span key={v} style={{ fontSize: '9px', color: montageCores === v ? 'var(--accent-primary)' : 'var(--text-tertiary)' }}>{v}</span>
                                ))}
                            </div>
                        </div>

                    </div>
                </div>

                <div style={{ marginTop: 'auto', display: 'flex', alignItems: 'center', gap: '8px', height: '20px' }}>
                    {isSaving && (
                        <>
                            <div className="spinner-tiny" style={{ borderTopColor: 'var(--accent-primary)' }} />
                            <span style={{ fontSize: '12px', color: 'var(--text-tertiary)', fontStyle: 'italic' }}>{t('performanceTab.saving')}</span>
                        </>
                    )}
                </div>
            </div >
        </div >
    );
};
