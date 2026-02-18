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

const container = document.getElementById('root');

const root = createRoot(container!);

root.render(
    <React.StrictMode>
        <I18nProvider>
            <ThemeProvider>
                <LoggerProvider>
                    <ServiceProvider>
                        <ToastProvider>
                            <QueueProvider>
                                <App />
                            </QueueProvider>
                        </ToastProvider>
                    </ServiceProvider>
                </LoggerProvider>
            </ThemeProvider>
        </I18nProvider>
    </React.StrictMode>
);
