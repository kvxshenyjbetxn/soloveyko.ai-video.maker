import React from 'react'
import { createRoot } from 'react-dom/client'
import './style.css'
import App from './App'
import { I18nProvider } from './contexts/I18nContext';
import { ThemeProvider } from './contexts/ThemeContext';
import { LoggerProvider } from './contexts/LoggerContext';
import { ServiceProvider } from './contexts/ServiceContext';
import { QueueProvider } from './contexts/QueueContext';
import { ToastProvider } from './contexts/ToastContext';
import { TemplateProvider } from './contexts/TemplateContext';
import { EditorDraftProvider } from './contexts/EditorDraftContext';
import { GoogleMonitorProvider } from './contexts/GoogleMonitorContext';
import { AuthProvider } from './contexts/AuthContext';

const container = document.getElementById('root');

const root = createRoot(container!);

root.render(
    <React.StrictMode>
        <I18nProvider>
            <ThemeProvider>
                <LoggerProvider>
                    <ServiceProvider>
                        <ToastProvider>
                            <TemplateProvider>
                                <EditorDraftProvider>
                                    <QueueProvider>
                                        <GoogleMonitorProvider>
                                            <AuthProvider>
                                                <App />
                                            </AuthProvider>
                                        </GoogleMonitorProvider>
                                    </QueueProvider>
                                </EditorDraftProvider>
                            </TemplateProvider>
                        </ToastProvider>
                    </ServiceProvider>
                </LoggerProvider>
            </ThemeProvider>
        </I18nProvider>
    </React.StrictMode>
);
