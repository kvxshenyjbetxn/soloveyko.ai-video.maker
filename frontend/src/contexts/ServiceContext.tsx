import React, { createContext, useContext, useState, useEffect, useRef } from 'react';
// @ts-ignore
import { GetOpenRouterCredits, GetOpenRouterAPIKey } from '../../wailsjs/go/main/App';
import { useLogger } from './LoggerContext';

interface ServiceContextType {
    openRouterBalance: number | null;
    refreshOpenRouterBalance: () => Promise<void>;
    loadingOpenRouter: boolean;
}

const ServiceContext = createContext<ServiceContextType>({
    openRouterBalance: null,
    refreshOpenRouterBalance: async () => { },
    loadingOpenRouter: false,
});

export const useServices = () => useContext(ServiceContext);

export const ServiceProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
    const { addLog } = useLogger();
    const [openRouterBalance, setOpenRouterBalance] = useState<number | null>(null);
    const [loadingOpenRouter, setLoadingOpenRouter] = useState(false);
    const hasFetchedRef = useRef(false);

    const refreshOpenRouterBalance = async () => {
        if (loadingOpenRouter) return;

        setLoadingOpenRouter(true);
        addLog('INFO', 'Requesting OpenRouter balance update...');
        try {
            const apiKey = await GetOpenRouterAPIKey();
            if (apiKey) {
                const credit = await GetOpenRouterCredits(apiKey);
                const balance = Math.max(0, credit);
                setOpenRouterBalance(balance);
                addLog('INFO', `Received OpenRouter balance: $${balance.toFixed(4)}`);
            } else {
                setOpenRouterBalance(null);
                addLog('WARN', 'OpenRouter API Key not found');
            }
        } catch (err: any) {
            console.error("Failed to update balance:", err);
            addLog('ERROR', `Failed to fetch OpenRouter balance: ${err?.message || String(err)}`);
        } finally {
            setLoadingOpenRouter(false);
        }
    };

    // Initial fetch on app start, ensuring it runs only once even in StrictMode
    useEffect(() => {
        if (!hasFetchedRef.current) {
            hasFetchedRef.current = true;
            refreshOpenRouterBalance();
        }
    }, []);

    return (
        <ServiceContext.Provider value={{ openRouterBalance, refreshOpenRouterBalance, loadingOpenRouter }}>
            {children}
        </ServiceContext.Provider>
    );
};
