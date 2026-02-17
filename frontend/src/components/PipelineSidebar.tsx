import { useState, useEffect, useRef, useCallback } from 'react';
import './PipelineSidebar.css';
import { useI18n } from '../contexts/I18nContext';
// @ts-ignore
import { GetPipelineSettings, SavePipelineSettings } from '../../wailsjs/go/main/App';

interface PipelineSidebarProps {
    type: 'translate' | 'rewrite';
}

export const PipelineSidebar: React.FC<PipelineSidebarProps> = ({ type }) => {
    const { t } = useI18n();
    const [settings, setSettings] = useState<any>(null);
    const [isResizing, setIsResizing] = useState(false);

    const sidebarRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const loadSettings = async () => {
            try {
                const s = await GetPipelineSettings();
                setSettings(s);
            } catch (err) {
                console.error("Failed to load pipeline settings:", err);
            }
        };

        loadSettings();
    }, []);

    const startResizing = useCallback((e: React.MouseEvent) => {
        setIsResizing(true);
        e.preventDefault();
    }, []);

    const stopResizing = useCallback(() => {
        setIsResizing(false);
        if (settings) {
            SavePipelineSettings(settings);
        }
    }, [settings]);

    const resize = useCallback((e: MouseEvent) => {
        if (isResizing && sidebarRef.current) {
            const newWidth = window.innerWidth - e.pageX;
            if (newWidth >= 250 && newWidth <= 600) {
                setSettings((prev: any) => ({
                    ...prev,
                    sidebarWidth: newWidth
                }));
            }
        }
    }, [isResizing]);

    useEffect(() => {
        if (isResizing) {
            window.addEventListener('mousemove', resize);
            window.addEventListener('mouseup', stopResizing);
        } else {
            window.removeEventListener('mousemove', resize);
            window.removeEventListener('mouseup', stopResizing);
        }

        return () => {
            window.removeEventListener('mousemove', resize);
            window.removeEventListener('mouseup', stopResizing);
        };
    }, [isResizing, resize, stopResizing]);

    if (!settings) return null;

    return (
        <aside
            className="pipeline-sidebar"
            ref={sidebarRef}
            style={{ width: `${settings.sidebarWidth || 320}px` }}
        >
            <div
                className={`sidebar-resizer ${isResizing ? 'is-resizing' : ''}`}
                onMouseDown={startResizing}
            />

            <div className="pipeline-sidebar-header">
                <div className="pipeline-sidebar-title">{t(`pipeline.${type}.title`)}</div>
            </div>

            <div className="pipeline-sidebar-content">
                {/* Панель готова для поступового заповнення */}
                <div style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    height: '100%',
                    color: 'var(--text-secondary)',
                    fontSize: '12px',
                    fontStyle: 'italic',
                    opacity: 0.5
                }}>
                    {t('pipeline.empty_state')}
                </div>
            </div>
        </aside>
    );
};
