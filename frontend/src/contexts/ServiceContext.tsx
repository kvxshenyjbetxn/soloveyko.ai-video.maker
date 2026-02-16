import React, { createContext, useContext, useState, useEffect } from 'react';
// @ts-ignore
import { GetOpenRouterCredits, GetOpenRouterAPIKey } from '../../wailsjs/go/main/App';

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
    const [openRouterBalance, setOpenRouterBalance] = useState<number | null>(null);
    const [loadingOpenRouter, setLoadingOpenRouter] = useState(false);

    const refreshOpenRouterBalance = async () => {
        // Prevent concurrent requests if already loading? 
        // No, user might want to force refresh.
        setLoadingOpenRouter(true);
        try {
            const apiKey = await GetOpenRouterAPIKey();
            if (apiKey) {
                const credit = await GetOpenRouterCredits(apiKey);
                setOpenRouterBalance(Math.max(0, credit));
            } else {
                setOpenRouterBalance(null);
            }
        } catch (err) {
            console.error("Failed to update balance:", err);
            // Optional: set error state
        } finally {
            setLoadingOpenRouter(false);
        }
    };

    // Initial fetch on app start
    useEffect(() => {
        refreshOpenRouterBalance();
    }, []);

    return (
        <ServiceContext.Provider value={{ openRouterBalance, refreshOpenRouterBalance, loadingOpenRouter }}>
            {children}
        </ServiceContext.Provider>
    );
};
