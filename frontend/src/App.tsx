import { useState, useEffect, useRef } from 'react';
import './App.css';
import { useI18n } from './contexts/I18nContext';
import { useQueue } from './contexts/QueueContext';
import { useLogger } from './contexts/LoggerContext';
import logo from './assets/logo.png';
import { ConfirmModal } from './components/ConfirmModal';
// @ts-ignore
import { GetPipelineSettings, OpenPath } from '../wailsjs/go/main/App';

// Import all tab components
import { Translate } from './tabs/text/translate';
import { Rewrite } from './tabs/text/rewrite';
import { Queue } from './tabs/queue';
import { Gallery } from './tabs/gallery';
import { General } from './tabs/settings/general';
import { SystemMonitor } from './components/SystemMonitor';
import { OpenRouter } from './tabs/settings/api/openrouter';
import { ServiceBalanceMonitor } from './components/ServiceBalanceMonitor';
import { QueueMonitor } from './components/QueueMonitor';
import { ElevenLabsBot } from './tabs/settings/api/voice/elevenlabsbot';
import { ElevenLabsUnlim } from './tabs/settings/api/voice/elevenlabsunlim';
import { ElevenLabsUA } from './tabs/settings/api/voice/elevenlabsua';
import { VoiceMaker } from './tabs/settings/api/voice/voicemaker';
import { PollinationsAI } from './tabs/settings/api/image/pollinationsai';
import { Googler } from './tabs/settings/api/image/googler';
import { ElevenLabsImage } from './tabs/settings/api/image/elevenlabsimage';
import { AssemblyAI } from './tabs/settings/api/assemblyai';
import { Montage } from './tabs/settings/montage';
import { Subtitle } from './tabs/settings/subtitle';
import { Templates } from './tabs/settings/templates';
import { Statistic } from './tabs/other/statistic';
import { History } from './tabs/other/history';
import { Logs } from './tabs/logs';

// Simple Icons (SVG)
const ScriptIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z" /><polyline points="14 2 14 8 20 8" /><line x1="16" y1="13" x2="8" y2="13" /><line x1="16" y1="17" x2="8" y2="17" /><polyline points="10 9 9 9 8 9" /></svg>
);
const QueueIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M6 18H18" /><path d="M6 12H18" /><path d="M6 6H18" /><circle cx="3" cy="6" r="1" /><circle cx="3" cy="12" r="1" /><circle cx="3" cy="18" r="1" /></svg>
);
const GalleryIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2" /><circle cx="8.5" cy="8.5" r="1.5" /><polyline points="21 15 16 10 5 21" /></svg>
);
const SettingsIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" /></svg>
);
const AIIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12 2a10 10 0 1 0 10 10H12V2z" /><path d="M12 12 2.1 12a10.05 10.05 0 0 1 9.9-10v10z" /><path d="m9 16.5 3-3" /></svg>
);
const OtherIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="1" /><circle cx="19" cy="12" r="1" /><circle cx="5" cy="12" r="1" /></svg>
);
const TerminalIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="4 17 10 11 4 5"></polyline><line x1="12" y1="19" x2="20" y2="19"></line></svg>
);
const ChevronRight = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="9 18 15 12 9 6"></polyline></svg>
);

type TabPath = string;

function App() {
    const { t } = useI18n();
    const { tasks, completionModal, closeCompletionModal, imageControlNotification, closeImageControlNotification } = useQueue();
    const pendingCount = tasks.filter(t => t.status === 'pending').length;
    const { addLog } = useLogger();
    const [currentPath, setCurrentPath] = useState<TabPath>('text.translate');
    const initLogRef = useRef(false);

    useEffect(() => {
        if (!initLogRef.current) {
            initLogRef.current = true;
            addLog('INFO', 'Application initialized');
        }
    }, []);

    const [expandedMenus, setExpandedMenus] = useState<{ [key: string]: boolean }>({
        'api': false,
        'voice': false,
        'image': false
    });

    const toggleMenu = (menu: string) => {
        setExpandedMenus(prev => ({
            ...prev,
            [menu]: !prev[menu]
        }));
    };

    const [hasImages, setHasImages] = useState(false);

    const checkGallery = async () => {
        try {
            // @ts-ignore
            const data = await window.go.main.App.GetGalleryImages();
            const exists = data && data.length > 0;
            setHasImages(exists);

            // If we are in gallery and it becomes empty, redirect
            if (!exists && currentPath === 'gallery') {
                setCurrentPath('settings.general');
            }
        } catch (e) {
            console.error("Failed to check gallery:", e);
        }
    };

    useEffect(() => {
        checkGallery();

        // Слухаємо лише специфічну подію для галереї
        // @ts-ignore
        if (window.runtime) {
            // @ts-ignore
            const unsubGallery = window.runtime.EventsOn("galleryUpdate", () => {
                checkGallery();
            });

            return () => {
                unsubGallery();
            };
        }
    }, []);

    // Використовуємо зміни в черзі для оновлення галереї замість Wails Events, 
    // щоб уникнути конфліктів відписки (відписується одразу для всіх)
    const completedTasksCount = tasks.filter(t => t.status === 'completed' || t.status === 'failed').length;
    const completedImagesCount = tasks.filter(t => t.imageStatus === 'completed').length;

    useEffect(() => {
        checkGallery();
    }, [completedTasksCount, completedImagesCount]);

    const renderContent = () => {
        switch (currentPath) {
            // Text tabs
            case 'text.translate': return <Translate setCurrentPath={setCurrentPath} />;
            case 'text.rewrite': return <Rewrite setCurrentPath={setCurrentPath} />;
            case 'queue': return <Queue setCurrentPath={setCurrentPath} />;
            case 'gallery': return <Gallery setCurrentPath={setCurrentPath} />;

            // Settings tabs
            case 'settings.general': return <General />;
            case 'settings.api.openrouter': return <OpenRouter />;
            case 'settings.api.voice.elevenlabsbot': return <ElevenLabsBot />;
            case 'settings.api.voice.elevenlabsunlim': return <ElevenLabsUnlim />;
            case 'settings.api.voice.elevenlabsua': return <ElevenLabsUA />;
            case 'settings.api.voice.voicemaker': return <VoiceMaker />;
            case 'settings.api.image.pollinationsai': return <PollinationsAI />;
            case 'settings.api.image.googler': return <Googler />;
            case 'settings.api.image.elevenlabsimage': return <ElevenLabsImage />;
            case 'settings.api.assemblyai': return <AssemblyAI />;
            case 'settings.montage': return <Montage />;
            case 'settings.subtitle': return <Subtitle />;
            case 'settings.templates': return <Templates />;

            // Other tabs
            case 'other.statistic': return <Statistic />;
            case 'other.history': return <History />;

            // Logs tab
            case 'logs': return <Logs />;

            default: return <Translate />;
        }
    };

    const getMainTab = (path: string) => path.split('.')[0];

    const renderSidebar = () => {
        const mainTab = getMainTab(currentPath);

        if (mainTab === 'logs') return null;

        if (mainTab === 'text') {
            return (
                <aside className="sidebar" key="text-sidebar">
                    <div
                        className={`sidebar-item animate-sidebar-item stagger-1 ${currentPath === 'text.translate' ? 'active' : ''}`}
                        onClick={() => setCurrentPath('text.translate')}
                    >
                        {t('text.translate')}
                    </div>
                    <div
                        className={`sidebar-item animate-sidebar-item stagger-2 ${currentPath === 'text.rewrite' ? 'active' : ''}`}
                        onClick={() => setCurrentPath('text.rewrite')}
                    >
                        {t('text.rewrite')}
                    </div>
                </aside>
            );
        }

        if (mainTab === 'settings') {
            return (
                <aside className="sidebar" key="settings-sidebar">
                    <div
                        className={`sidebar-item animate-sidebar-item stagger-1 ${currentPath === 'settings.general' ? 'active' : ''}`}
                        onClick={() => setCurrentPath('settings.general')}
                    >
                        {t('settings.general')}
                    </div>

                    {/* API Section */}
                    <div className="sidebar-group">
                        <div
                            className="sidebar-item animate-sidebar-item stagger-2 sidebar-parent"
                            onClick={() => toggleMenu('api')}
                        >
                            <span>{t('settings.api')}</span>
                            <span className={`chevron ${expandedMenus.api ? 'expanded' : ''}`}>
                                <ChevronRight />
                            </span>
                        </div>
                        {expandedMenus.api && (
                            <div className="sidebar-submenu">
                                <div
                                    className={`sidebar-item animate-sidebar-item stagger-1 ${currentPath === 'settings.api.openrouter' ? 'active' : ''}`}
                                    onClick={() => setCurrentPath('settings.api.openrouter')}
                                >
                                    {t('api.openrouter')}
                                </div>

                                {/* Voice submenu */}
                                <div className="sidebar-group">
                                    <div
                                        className="sidebar-item animate-sidebar-item stagger-2 sidebar-parent"
                                        onClick={() => toggleMenu('voice')}
                                    >
                                        <span>{t('api.voice')}</span>
                                        <span className={`chevron ${expandedMenus.voice ? 'expanded' : ''}`}>
                                            <ChevronRight />
                                        </span>
                                    </div>
                                    {expandedMenus.voice && (
                                        <div className="sidebar-submenu">
                                            <div
                                                className={`sidebar-item animate-sidebar-item stagger-1 ${currentPath === 'settings.api.voice.elevenlabsbot' ? 'active' : ''}`}
                                                onClick={() => setCurrentPath('settings.api.voice.elevenlabsbot')}
                                            >
                                                {t('voice.elevenlabsbot')}
                                            </div>
                                            <div
                                                className={`sidebar-item animate-sidebar-item stagger-2 ${currentPath === 'settings.api.voice.elevenlabsunlim' ? 'active' : ''}`}
                                                onClick={() => setCurrentPath('settings.api.voice.elevenlabsunlim')}
                                            >
                                                {t('voice.elevenlabsunlim')}
                                            </div>
                                            <div
                                                className={`sidebar-item animate-sidebar-item stagger-2 ${currentPath === 'settings.api.voice.elevenlabsua' ? 'active' : ''}`}
                                                onClick={() => setCurrentPath('settings.api.voice.elevenlabsua')}
                                            >
                                                {t('voice.elevenlabsua')}
                                            </div>
                                            <div
                                                className={`sidebar-item animate-sidebar-item stagger-3 ${currentPath === 'settings.api.voice.voicemaker' ? 'active' : ''}`}
                                                onClick={() => setCurrentPath('settings.api.voice.voicemaker')}
                                            >
                                                {t('voice.voicemaker')}
                                            </div>
                                        </div>
                                    )}
                                </div>

                                {/* Image submenu */}
                                <div className="sidebar-group">
                                    <div
                                        className="sidebar-item animate-sidebar-item stagger-3 sidebar-parent"
                                        onClick={() => toggleMenu('image')}
                                    >
                                        <span>{t('api.image')}</span>
                                        <span className={`chevron ${expandedMenus.image ? 'expanded' : ''}`}>
                                            <ChevronRight />
                                        </span>
                                    </div>
                                    {expandedMenus.image && (
                                        <div className="sidebar-submenu">
                                            <div
                                                className={`sidebar-item animate-sidebar-item stagger-1 ${currentPath === 'settings.api.image.pollinationsai' ? 'active' : ''}`}
                                                onClick={() => setCurrentPath('settings.api.image.pollinationsai')}
                                            >
                                                {t('image.pollinationsai')}
                                            </div>
                                            <div
                                                className={`sidebar-item animate-sidebar-item stagger-2 ${currentPath === 'settings.api.image.googler' ? 'active' : ''}`}
                                                onClick={() => setCurrentPath('settings.api.image.googler')}
                                            >
                                                {t('image.googler')}
                                            </div>
                                            <div
                                                className={`sidebar-item animate-sidebar-item stagger-3 ${currentPath === 'settings.api.image.elevenlabsimage' ? 'active' : ''}`}
                                                onClick={() => setCurrentPath('settings.api.image.elevenlabsimage')}
                                            >
                                                {t('image.elevenlabsimage')}
                                            </div>
                                        </div>
                                    )}
                                </div>

                                <div
                                    className={`sidebar-item animate-sidebar-item stagger-4 ${currentPath === 'settings.api.assemblyai' ? 'active' : ''}`}
                                    onClick={() => setCurrentPath('settings.api.assemblyai')}
                                >
                                    {t('api.assemblyai')}
                                </div>
                            </div>
                        )}
                    </div>

                    <div
                        className={`sidebar-item animate-sidebar-item stagger-3 ${currentPath === 'settings.montage' ? 'active' : ''}`}
                        onClick={() => setCurrentPath('settings.montage')}
                    >
                        {t('settings.montage')}
                    </div>
                    <div
                        className={`sidebar-item animate-sidebar-item stagger-4 ${currentPath === 'settings.subtitle' ? 'active' : ''}`}
                        onClick={() => setCurrentPath('settings.subtitle')}
                    >
                        {t('settings.subtitle')}
                    </div>
                    <div
                        className={`sidebar-item animate-sidebar-item stagger-5 ${currentPath === 'settings.templates' ? 'active' : ''}`}
                        onClick={() => setCurrentPath('settings.templates')}
                    >
                        {t('settings.templates')}
                    </div>
                </aside>
            );
        }

        if (mainTab === 'other') {
            return (
                <aside className="sidebar" key="other-sidebar">
                    <div
                        className={`sidebar-item animate-sidebar-item stagger-1 ${currentPath === 'other.statistic' ? 'active' : ''}`}
                        onClick={() => setCurrentPath('other.statistic')}
                    >
                        {t('other.statistic')}
                    </div>
                    <div
                        className={`sidebar-item animate-sidebar-item stagger-2 ${currentPath === 'other.history' ? 'active' : ''}`}
                        onClick={() => setCurrentPath('other.history')}
                    >
                        {t('other.history')}
                    </div>
                </aside>
            );
        }

        return null;
    };

    return (
        <div className="app-container">
            {/* Top Header with Tabs */}
            <header className="app-header">
                <div className="header-content">
                    <div className="logo-section">
                        <div className="logo-icon">
                            <img src={logo} alt="Soloveyko" className="app-logo-img" />
                        </div>
                        <span className="app-title">{t('app.title')}</span>
                    </div>

                    <nav className="tabs-nav">
                        <div
                            className={`tab-item ${getMainTab(currentPath) === 'text' ? 'active' : ''}`}
                            onClick={() => setCurrentPath('text.translate')}
                        >
                            <ScriptIcon />
                            <span>{t('tabs.text')}</span>
                        </div>
                        {tasks.length > 0 && (
                            <div
                                className={`tab-item ${getMainTab(currentPath) === 'queue' ? 'active' : ''}`}
                                onClick={() => setCurrentPath('queue')}
                            >
                                <QueueIcon />
                                <span>{t('tabs.queue')}</span>
                                {pendingCount > 0 && <span className="tab-badge">{pendingCount}</span>}
                            </div>
                        )}
                        {hasImages && (
                            <div
                                className={`tab-item ${getMainTab(currentPath) === 'gallery' ? 'active' : ''}`}
                                onClick={() => setCurrentPath('gallery')}
                            >
                                <GalleryIcon />
                                <span>{t('tabs.gallery')}</span>
                            </div>
                        )}
                        <div
                            className={`tab-item ${getMainTab(currentPath) === 'settings' ? 'active' : ''}`}
                            onClick={() => setCurrentPath('settings.general')}
                        >
                            <SettingsIcon />
                            <span>{t('tabs.settings')}</span>
                        </div>
                        <div
                            className={`tab-item ${getMainTab(currentPath) === 'other' ? 'active' : ''}`}
                            onClick={() => setCurrentPath('other.statistic')}
                        >
                            <OtherIcon />
                            <span>{t('tabs.other')}</span>
                        </div>
                        <div
                            className={`tab-item ${currentPath === 'logs' ? 'active' : ''}`}
                            onClick={() => setCurrentPath('logs')}
                        >
                            <TerminalIcon />
                            <span>{t('tabs.logs')}</span>
                        </div>
                    </nav>
                </div>
            </header>

            {/* Content Area with Sidebar */}
            <div className="content-with-sidebar">
                {renderSidebar()}
                <main className="main-content animate-fade" key={currentPath}>
                    {renderContent()}
                </main>
            </div>
            <div className="monitors-container" style={{
                position: 'fixed',
                bottom: '15px',
                right: 'calc(10px + var(--pipeline-sidebar-width, 0px) + var(--sidebar-toggle-width, 0px))',
                display: 'flex',
                transition: 'right 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
                flexDirection: 'row-reverse',
                gap: '15px',
                alignItems: 'flex-end',
                zIndex: 10000,
                pointerEvents: 'none'
            }}>
                <SystemMonitor />
                <ServiceBalanceMonitor navigateTo={setCurrentPath} />
                <QueueMonitor navigateTo={setCurrentPath} />
            </div>

            <ConfirmModal
                isOpen={completionModal.isOpen}
                onClose={closeCompletionModal}
                onConfirm={closeCompletionModal}
                title={t('queue.completion_title')}
                message={t('queue.completion_message')
                    .replace('{count}', completionModal.taskCount.toString())
                    .replace('{duration}', completionModal.duration)}
                confirmText={t('queue.completion_ok')}
                extraText={t('queue.completion_open_folder')}
                extraAction={async () => {
                    try {
                        const settings = await GetPipelineSettings();
                        if (settings && settings.outputPath) {
                            await OpenPath(settings.outputPath);
                        }
                    } catch (e) {
                        console.error("Failed to open output folder:", e);
                    }
                }}
                isDanger={false}
                type="info"
            />

            <ConfirmModal
                isOpen={imageControlNotification.isOpen}
                onClose={closeImageControlNotification}
                onConfirm={() => {
                    setCurrentPath('gallery');
                    closeImageControlNotification();
                }}
                title={t('pipeline.image_control_notification.title')}
                message={t('pipeline.image_control_notification.message')}
                confirmText={t('pipeline.image_control_notification.go_to_gallery')}
                isDanger={false}
                type="info"
            />
        </div>
    )
}

export default App
