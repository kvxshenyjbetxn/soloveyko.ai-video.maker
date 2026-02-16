import React, { useState } from 'react';
import { useI18n } from '../contexts/I18nContext';
import './gallery.css';

interface MediaFile {
    name: string;
    type: 'image' | 'video';
    url: string;
}

export const Gallery = () => {
    const { t } = useI18n();
    const [files, setFiles] = useState<MediaFile[]>([]);
    const [selectedMedia, setSelectedMedia] = useState<MediaFile | null>(null);

    const loadDemoFiles = () => {
        const demoFiles: MediaFile[] = [
            {
                name: `test_image_${files.length + 1}.jpg`,
                type: 'image',
                url: 'https://picsum.photos/1280/720?random=' + Math.random()
            },
            {
                name: `test_video_${files.length + 2}.mp4`,
                type: 'video',
                url: 'https://www.w3schools.com/html/mov_bbb.mp4'
            }
        ];
        setFiles(prev => [...prev, ...demoFiles]);
    };

    return (
        <div className="content-wrapper animate-fade gallery-page">
            <div className="gallery-header">
                <h2>{t('gallery.title')}</h2>
                <button className="btn-accent" onClick={loadDemoFiles}>
                    {t('gallery.loadDemo')}
                </button>
            </div>

            <div className="gallery-scroll-container">
                {files.length === 0 ? (
                    <div className="gallery-empty">
                        <p>{t('gallery.empty')}</p>
                    </div>
                ) : (
                    <div className="gallery-grid">
                        {files.map((file, index) => (
                            <div
                                key={index}
                                className="gallery-card animate-sidebar-item"
                                onClick={() => setSelectedMedia(file)}
                            >
                                <div className="media-container">
                                    {file.type === 'image' ? (
                                        <img src={file.url} alt={file.name} loading="lazy" />
                                    ) : (
                                        <video
                                            src={file.url}
                                            loop
                                            muted
                                            playsInline
                                            onMouseEnter={(e) => e.currentTarget.play()}
                                            onMouseLeave={(e) => {
                                                e.currentTarget.pause();
                                                e.currentTarget.currentTime = 0;
                                            }}
                                        />
                                    )}
                                </div>
                                <div className="media-info">
                                    <span className="media-name">{file.name}</span>
                                </div>
                            </div>
                        ))}
                    </div>
                )}
            </div>

            {selectedMedia && (
                <div className="media-modal" onClick={() => setSelectedMedia(null)}>
                    <div className="modal-content animate-fade" onClick={e => e.stopPropagation()}>
                        <button className="modal-close" onClick={() => setSelectedMedia(null)}>&times;</button>
                        {selectedMedia.type === 'image' ? (
                            <img src={selectedMedia.url} alt={selectedMedia.name} />
                        ) : (
                            <video src={selectedMedia.url} autoPlay loop muted playsInline />
                        )}
                        <div className="modal-info">
                            <h3>{selectedMedia.name}</h3>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
};
