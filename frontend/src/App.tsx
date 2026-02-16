import { useState } from 'react';
import './App.css';

// Simple Icons (SVG)
const ScriptIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z" /><polyline points="14 2 14 8 20 8" /><line x1="16" y1="13" x2="8" y2="13" /><line x1="16" y1="17" x2="8" y2="17" /><polyline points="10 9 9 9 8 9" /></svg>
);
const SettingsIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" /></svg>
);
const AIIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12 2a10 10 0 1 0 10 10H12V2z" /><path d="M12 12 2.1 12a10.05 10.05 0 0 1 9.9-10v10z" /><path d="m9 16.5 3-3" /></svg>
);

function App() {
    const [text, setText] = useState("");
    const [activeTab, setActiveTab] = useState('script');

    const updateText = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
        setText(e.target.value);
    };

    const characterCount = text.length;
    const wordCount = text.trim() === "" ? 0 : text.trim().split(/\s+/).length;
    const paragraphCount = text.trim() === "" ? 0 : text.trim().split(/\n+/).length;

    return (
        <div className="app-container">
            {/* Sidebar */}
            <aside className="sidebar">
                <div className="logo-area">
                    <div className="logo-icon"><AIIcon /></div>
                    <span className="logo-text">Soloveyko</span>
                </div>
                <nav className="nav-menu">
                    <div className={`nav-item ${activeTab === 'script' ? 'active' : ''}`} onClick={() => setActiveTab('script')}>
                        <ScriptIcon />
                        <span>Сценарій</span>
                    </div>
                    <div className={`nav-item ${activeTab === 'settings' ? 'active' : ''}`} onClick={() => setActiveTab('settings')}>
                        <SettingsIcon />
                        <span>Налаштування</span>
                    </div>
                </nav>
            </aside>

            {/* Main Content */}
            <main className="main-content">
                <header className="top-bar">
                    <h2>Редактор Сценарію</h2>
                    <div className="actions">
                        <button className="btn-primary">Згенерувати Відео</button>
                    </div>
                </header>

                <div className="content-area">
                    <div className="script-editor-container">
                        <textarea
                            className="script-input"
                            value={text}
                            onChange={updateText}
                            placeholder="Почніть писати ваш сценарій тут..."
                            spellCheck={false}
                        />
                        <div className="stats-bar">
                            <div className="stat-group">
                                <span className="stat-label">Символи:</span>
                                <span className="stat-value">{characterCount}</span>
                            </div>
                            <div className="stat-separator">|</div>
                            <div className="stat-group">
                                <span className="stat-label">Слова:</span>
                                <span className="stat-value">{wordCount}</span>
                            </div>
                            <div className="stat-separator">|</div>
                            <div className="stat-group">
                                <span className="stat-label">Абзаци:</span>
                                <span className="stat-value">{paragraphCount}</span>
                            </div>
                        </div>
                    </div>
                </div>
            </main>
        </div>
    )
}

export default App
