import React, { useState, useEffect } from 'react';
import { useI18n } from '../contexts/I18nContext';
import {
    RegenerateGalleryImage,
    GetPollinationsSavedModels,
    SelectImage
} from '../../wailsjs/go/main/App';
import './RegenerateModal.css';

interface RegenerateModalProps {
    isOpen: boolean;
    onClose: () => void;
    onConfirm: (prompt: string, service: string, settings: any) => void;
    imagePath: string;
    initialPrompt: string;
    isBulk?: boolean;
}

export const RegenerateModal: React.FC<RegenerateModalProps> = ({
    isOpen,
    onClose,
    onConfirm,
    imagePath,
    initialPrompt,
    isBulk = false
}) => {
    const { t } = useI18n();
    const [service, setService] = useState('googler');
    const [prompt, setPrompt] = useState(initialPrompt || '');
    const [loading, setLoading] = useState(false);

    // Pollinations settings
    const [polModels, setPolModels] = useState<string[]>([]);
    const [polModel, setPolModel] = useState('flux');
    const [polWidth, setPolWidth] = useState(1920);
    const [polHeight, setPolHeight] = useState(1080);

    // Googler settings
    const [gooModel, setGooModel] = useState('flow');
    const [gooRatio, setGooRatio] = useState('IMAGE_ASPECT_RATIO_LANDSCAPE');
    const [gooRefImage, setGooRefImage] = useState('');
    const [gooVideo, setGooVideo] = useState(false);

    // ElevenLabs settings
    const [elRatio, setElRatio] = useState('landscape');

    const isMediaVideo = imagePath?.toLowerCase().endsWith('.mp4');

    useEffect(() => {
        if (isOpen) {
            loadInitialData();
            setPrompt(initialPrompt || '');
            if (isMediaVideo) {
                setService('googler');
                setGooVideo(true);
            } else {
                setGooVideo(false);
            }
        }
    }, [isOpen, initialPrompt, isMediaVideo]);

    const loadInitialData = async () => {
        try {
            const models = await GetPollinationsSavedModels();
            if (models && models.length > 0) {
                setPolModels(models);
                setPolModel(models[0]);
            }
        } catch (e) {
            console.error("Failed to load models:", e);
        }
    };

    const handleSelectRefImage = async () => {
        try {
            const path = await SelectImage();
            if (path) setGooRefImage(path);
        } catch (e) {
            console.error("Failed to select image:", e);
        }
    };

    const handleRegenerate = () => {
        const settings: any = {};
        if (service === 'pollinations') {
            settings.imageModel = polModel;
            settings.imageWidth = polWidth;
            settings.imageHeight = polHeight;
        } else if (service === 'googler') {
            settings.imageGooglerModel = gooModel;
            settings.imageGooglerAspectRatio = gooRatio;
            settings.imageGooglerReferenceImage = gooRefImage;
            settings.imageGooglerVideoEnabled = gooVideo;
            // Defaults for video
            settings.imageGooglerVideoModel = gooModel;
            settings.imageGooglerVideoUpscale = true;
        } else if (service === 'elevenlabs') {
            settings.elevenLabsImageAspectRatio = elRatio;
        }

        onConfirm(prompt, service, settings);
        onClose();
    };

    if (!isOpen) return null;

    return (
        <div className="regenerate-modal-overlay animate-fade" onClick={onClose}>
            <div className="regenerate-modal-content animate-slide-up" onClick={e => e.stopPropagation()}>
                <div className="reg-modal-header">
                    <h3>{t('gallery.regenerate_modal.title')}</h3>
                    <button className="reg-close-btn" onClick={onClose}>&times;</button>
                </div>

                <div className="reg-modal-body premium-scrollbar">
                    <div className="reg-field">
                        <label>{t('gallery.regenerate_modal.service') || 'Service'}</label>
                        {!isMediaVideo ? (
                            <div className="service-selector">
                                <button className={service === 'googler' ? 'active' : ''} onClick={() => setService('googler')}>Googler</button>
                                <button className={service === 'pollinations' ? 'active' : ''} onClick={() => setService('pollinations')}>Pollinations</button>
                                <button className={service === 'elevenlabs' ? 'active' : ''} onClick={() => setService('elevenlabs')}>ElevenLabs</button>
                            </div>
                        ) : (
                            <div className="service-selector">
                                <button className="active">Googler (Video)</button>
                            </div>
                        )}
                    </div>

                    {!isBulk && (
                        <div className="reg-field">
                            <label>{t('gallery.regenerate_modal.prompt')}</label>
                            <textarea
                                value={prompt}
                                onChange={e => setPrompt(e.target.value)}
                                className="reg-textarea"
                                rows={4}
                            />
                        </div>
                    )}

                    {service === 'pollinations' && (
                        <div className="service-settings">
                            <div className="reg-field">
                                <label>{t('gallery.regenerate_modal.pollinations_model')}</label>
                                <select value={polModel} onChange={e => setPolModel(e.target.value)}>
                                    {polModels.map(m => <option key={m} value={m}>{m}</option>)}
                                </select>
                            </div>
                            <div className="reg-row">
                                <div className="reg-field">
                                    <label>{t('pipeline.image.width')}</label>
                                    <input type="number" value={polWidth} onChange={e => setPolWidth(parseInt(e.target.value))} />
                                </div>
                                <div className="reg-field">
                                    <label>{t('pipeline.image.height')}</label>
                                    <input type="number" value={polHeight} onChange={e => setPolHeight(parseInt(e.target.value))} />
                                </div>
                            </div>
                        </div>
                    )}

                    {service === 'googler' && (
                        <div className="service-settings">
                            <div className="reg-field">
                                <label>{t('gallery.regenerate_modal.googler_model')}</label>
                                <select value={gooModel} onChange={e => setGooModel(e.target.value)}>
                                    <option value="flow">Flow</option>
                                    <option value="flow_gempix2">Flow Nano Pro</option>
                                    <option value="flow_imagen4">Flow Imagen 4</option>
                                    <option value="flow_narwhal">Flow Nano Banana 2</option>
                                    <option value="gemini">Gemini</option>
                                    <option value="grok">Grok</option>
                                    <option value="flower">Flower / Veo 3.1</option>
                                    <option value="openai">OpenAI / ChatGPT</option>
                                </select>
                            </div>
                            <div className="reg-field">
                                <label>{t('gallery.regenerate_modal.aspect_ratio')}</label>
                                <select value={gooRatio} onChange={e => setGooRatio(e.target.value)}>
                                    <option value="IMAGE_ASPECT_RATIO_LANDSCAPE">{t('pipeline.image.aspect_ratio_landscape')}</option>
                                    <option value="IMAGE_ASPECT_RATIO_PORTRAIT">{t('pipeline.image.aspect_ratio_portrait')}</option>
                                    <option value="IMAGE_ASPECT_RATIO_SQUARE">Square (1:1)</option>
                                </select>
                            </div>

                            <div className="reg-field">
                                <label>{t('gallery.regenerate_modal.reference_image')}</label>
                                <div className="ref-image-box">
                                    {gooRefImage ? (
                                        <div className="ref-preview">
                                            <span>{gooRefImage.split(/[\\/]/).pop()}</span>
                                            <button onClick={handleSelectRefImage}>{t('gallery.regenerate_modal.change_image')}</button>
                                        </div>
                                    ) : (
                                        <button className="ref-select-btn" onClick={handleSelectRefImage}>
                                            {t('gallery.regenerate_modal.select_image')}
                                        </button>
                                    )}
                                </div>
                            </div>

                            {!isMediaVideo && (
                                <div className="reg-field-check">
                                    <input
                                        type="checkbox"
                                        id="gooVideo"
                                        checked={gooVideo}
                                        onChange={e => setGooVideo(e.target.checked)}
                                    />
                                    <label htmlFor="gooVideo">{t('pipeline.image.googler.video_enabled')}</label>
                                </div>
                            )}
                        </div>
                    )}

                    {service === 'elevenlabs' && (
                        <div className="service-settings">
                            <div className="reg-field">
                                <label>{t('gallery.regenerate_modal.aspect_ratio')}</label>
                                <select value={elRatio} onChange={e => setElRatio(e.target.value)}>
                                    <option value="landscape">{t('pipeline.image.aspect_ratio_landscape')}</option>
                                    <option value="portrait">{t('pipeline.image.aspect_ratio_portrait')}</option>
                                    <option value="square">Square (1:1)</option>
                                </select>
                            </div>
                        </div>
                    )}
                </div>

                <div className="reg-modal-footer">
                    <button className="reg-cancel-btn" onClick={onClose} disabled={loading}>
                        {t('common.cancel')}
                    </button>
                    <button className="reg-submit-btn" onClick={handleRegenerate} disabled={loading}>
                        {loading ? t('common.saving') : t('gallery.regenerate_modal.generate')}
                    </button>
                </div>
            </div>
        </div>
    );
};
