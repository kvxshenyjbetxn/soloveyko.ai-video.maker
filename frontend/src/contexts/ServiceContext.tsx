import React, { createContext, useContext, useState, useEffect, useRef } from 'react';
// @ts-ignore
import { GetOpenRouterCredits, GetOpenRouterAPIKey, GetElevenLabsBotBalance, GetElevenLabsBotAPIKey, GetElevenLabsUnlimBalance, GetElevenLabsUnlimAPIKey, GetVoiceMakerBalance, GetVoiceMakerAPIKey, GetVoiceMakerSavedBalance, SaveVoiceMakerBalance, GetGooglerUsage, GetGooglerAPIKey } from '../../wailsjs/go/main/App';
import { useLogger } from './LoggerContext';

interface ServiceContextType {
    openRouterBalance: number | null;
    refreshOpenRouterBalance: () => Promise<void>;
    loadingOpenRouter: boolean;
    elevenLabsBotBalance: number | null;
    refreshElevenLabsBotBalance: () => Promise<void>;
    loadingElevenLabsBot: boolean;
    elevenLabsUnlimBalance: number | null;
    refreshElevenLabsUnlimBalance: () => Promise<void>;
    loadingElevenLabsUnlim: boolean;
    voiceMakerBalance: number | null;
    refreshVoiceMakerBalance: () => Promise<void>;
    loadingVoiceMaker: boolean;
    googlerUsage: any;
    refreshGooglerUsage: () => Promise<void>;
    loadingGoogler: boolean;
    elevenLabsBotThreshold: number;
    setElevenLabsBotThreshold: (val: number) => void;
    elevenLabsUnlimThreshold: number;
    setElevenLabsUnlimThreshold: (val: number) => void;
    voiceMakerThreshold: number;
    setVoiceMakerThreshold: (val: number) => void;
    openRouterThreshold: number;
    setOpenRouterThreshold: (val: number) => void;
    googlerVideoThreshold: number;
    setGooglerVideoThreshold: (val: number) => void;
    googlerImageThreshold: number;
    setGooglerImageThreshold: (val: number) => void;
    refreshAllBalances: () => Promise<void>;
}

const ServiceContext = createContext<ServiceContextType>({
    openRouterBalance: null,
    refreshOpenRouterBalance: async () => { },
    loadingOpenRouter: false,
    elevenLabsBotBalance: null,
    refreshElevenLabsBotBalance: async () => { },
    loadingElevenLabsBot: false,
    elevenLabsUnlimBalance: null,
    refreshElevenLabsUnlimBalance: async () => { },
    loadingElevenLabsUnlim: false,
    voiceMakerBalance: null,
    refreshVoiceMakerBalance: async () => { },
    loadingVoiceMaker: false,
    googlerUsage: {
        account_limits: { video_generation_threads_allowed: 0, img_generation_threads_allowed: 0, video_gen_per_hour_limit: 0, img_gen_per_hour_limit: 0, prompt_tokens_per_hour_limit: 0 },
        current_usage: { active_threads: { video_threads: 0, image_threads: 0 }, hourly_usage: { image_generation: 0, video_generation: 0, prompt_generation: 0 } },
        usage_window: 'per_hour',
        activation_date: 0,
        expiration_date: 0
    },
    refreshGooglerUsage: async () => { },
    loadingGoogler: false,
    elevenLabsBotThreshold: 0,
    setElevenLabsBotThreshold: () => { },
    elevenLabsUnlimThreshold: 0,
    setElevenLabsUnlimThreshold: () => { },
    voiceMakerThreshold: 0,
    setVoiceMakerThreshold: () => { },
    openRouterThreshold: 0,
    setOpenRouterThreshold: () => { },
    googlerVideoThreshold: 0,
    setGooglerVideoThreshold: () => { },
    googlerImageThreshold: 0,
    setGooglerImageThreshold: () => { },
    refreshAllBalances: async () => { },
});

export const useServices = () => useContext(ServiceContext);

export const ServiceProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
    const { addLog } = useLogger();
    const [openRouterBalance, setOpenRouterBalance] = useState<number | null>(null);
    const [loadingOpenRouter, setLoadingOpenRouter] = useState(false);
    const [elevenLabsBotBalance, setElevenLabsBotBalance] = useState<number | null>(null);
    const [loadingElevenLabsBot, setLoadingElevenLabsBot] = useState(false);
    const [elevenLabsUnlimBalance, setElevenLabsUnlimBalance] = useState<number | null>(null);
    const [loadingElevenLabsUnlim, setLoadingElevenLabsUnlim] = useState(false);
    const [voiceMakerBalance, setVoiceMakerBalance] = useState<number | null>(null);
    const [loadingVoiceMaker, setLoadingVoiceMaker] = useState(false);
    const [googlerUsage, setGooglerUsage] = useState<any>({
        account_limits: { video_generation_threads_allowed: 0, img_generation_threads_allowed: 0, video_gen_per_hour_limit: 0, img_gen_per_hour_limit: 0, prompt_tokens_per_hour_limit: 0 },
        current_usage: { active_threads: { video_threads: 0, image_threads: 0 }, hourly_usage: { image_generation: 0, video_generation: 0, prompt_generation: 0 } },
        usage_window: 'per_hour',
        activation_date: 0,
        expiration_date: 0
    });
    const [loadingGoogler, setLoadingGoogler] = useState(false);
    const [elevenLabsBotThreshold, setElevenLabsBotThreshold] = useState(0);
    const [elevenLabsUnlimThreshold, setElevenLabsUnlimThreshold] = useState(0);
    const [voiceMakerThreshold, setVoiceMakerThreshold] = useState(0);
    const [openRouterThreshold, setOpenRouterThreshold] = useState(0);
    const [googlerVideoThreshold, setGooglerVideoThreshold] = useState(0);
    const [googlerImageThreshold, setGooglerImageThreshold] = useState(0);
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

    const refreshElevenLabsBotBalance = async () => {
        if (loadingElevenLabsBot) return;

        setLoadingElevenLabsBot(true);
        addLog('INFO', 'Requesting ElevenLabsBot balance update...');
        try {
            const apiKey = await GetElevenLabsBotAPIKey();
            if (apiKey) {
                const balance = await GetElevenLabsBotBalance(apiKey);
                setElevenLabsBotBalance(balance);
                addLog('INFO', `Received ElevenLabsBot balance: ${balance.toFixed(0)} chars`);
            } else {
                setElevenLabsBotBalance(null);
                addLog('WARN', 'ElevenLabsBot API Key not found');
            }
        } catch (err: any) {
            console.error("Failed to update ElevenLabsBot balance:", err);
            addLog('ERROR', `Failed to fetch ElevenLabsBot balance: ${err?.message || String(err)}`);
        } finally {
            setLoadingElevenLabsBot(false);
        }
    }

    const refreshElevenLabsUnlimBalance = async () => {
        if (loadingElevenLabsUnlim) return;

        setLoadingElevenLabsUnlim(true);
        addLog('INFO', 'Requesting ElevenLabsUnlim balance update...');
        try {
            const apiKey = await GetElevenLabsUnlimAPIKey();
            if (apiKey) {
                const balance = await GetElevenLabsUnlimBalance(apiKey);
                setElevenLabsUnlimBalance(balance);
                addLog('INFO', `Received ElevenLabsUnlim balance: ${balance === -1 ? 'Unlimited' : balance.toFixed(0) + ' chars'}`);
            } else {
                setElevenLabsUnlimBalance(null);
                addLog('WARN', 'ElevenLabsUnlim API Key not found');
            }
        } catch (err: any) {
            console.error("Failed to update ElevenLabsUnlim balance:", err);
            addLog('ERROR', `Failed to fetch ElevenLabsUnlim balance: ${err?.message || String(err)}`);
        } finally {
            setLoadingElevenLabsUnlim(false);
        }
    }

    const refreshAllBalances = async () => {
        await Promise.all([
            refreshOpenRouterBalance(),
            refreshElevenLabsBotBalance(),
            refreshElevenLabsUnlimBalance(),
            refreshGooglerUsage()
        ]);
    };

    // Initial fetch on app start, ensuring it runs only once even in StrictMode
    useEffect(() => {
        if (!hasFetchedRef.current) {
            hasFetchedRef.current = true;

            // Завантажуємо збережений баланс VoiceMaker
            const loadVoiceMakerBalance = async () => {
                const savedBalance = await GetVoiceMakerSavedBalance();
                if (savedBalance > 0) {
                    setVoiceMakerBalance(savedBalance);
                }
            };
            loadVoiceMakerBalance();

            // Завантажуємо пороги попередження
            // @ts-ignore
            const { GetElevenLabsBotAlertThreshold, GetElevenLabsUnlimAlertThreshold, GetVoiceMakerAlertThreshold, GetOpenRouterAlertThreshold } = window.go.main.App;

            if (GetElevenLabsBotAlertThreshold) GetElevenLabsBotAlertThreshold().then(setElevenLabsBotThreshold);
            if (GetElevenLabsUnlimAlertThreshold) GetElevenLabsUnlimAlertThreshold().then(setElevenLabsUnlimThreshold);
            if (GetVoiceMakerAlertThreshold) GetVoiceMakerAlertThreshold().then(setVoiceMakerThreshold);
            if (GetOpenRouterAlertThreshold) GetOpenRouterAlertThreshold().then(setOpenRouterThreshold);

            const { GetGooglerVideoAlertThreshold, GetGooglerImageAlertThreshold } = (window as any).go.main.App;
            if (GetGooglerVideoAlertThreshold) GetGooglerVideoAlertThreshold().then(setGooglerVideoThreshold);
            if (GetGooglerImageAlertThreshold) GetGooglerImageAlertThreshold().then(setGooglerImageThreshold);

            refreshAllBalances();
        }
    }, []);

    const refreshVoiceMakerBalance = async () => {
        if (loadingVoiceMaker) return;

        setLoadingVoiceMaker(true);
        addLog('INFO', 'Requesting VoiceMaker balance update (test conversion)...');
        try {
            const apiKey = await GetVoiceMakerAPIKey();
            if (apiKey) {
                const balance = await GetVoiceMakerBalance(apiKey);
                setVoiceMakerBalance(balance);
                await SaveVoiceMakerBalance(balance); // Зберігаємо в налаштування
                addLog('INFO', `Received VoiceMaker balance: ${balance.toFixed(0)} chars`);
            } else {
                setVoiceMakerBalance(null);
                addLog('WARN', 'VoiceMaker API Key not found');
            }
        } catch (err: any) {
            console.error("Failed to update VoiceMaker balance:", err);
            addLog('ERROR', `Failed to fetch VoiceMaker balance: ${err?.message || String(err)}`);
        } finally {
            setLoadingVoiceMaker(false);
        }
    }

    const refreshGooglerUsage = async () => {
        if (loadingGoogler) return;

        setLoadingGoogler(true);
        addLog('INFO', 'Requesting Googler usage stats...');
        try {
            const apiKey = await GetGooglerAPIKey();
            if (apiKey) {
                const usage = await GetGooglerUsage(apiKey);
                console.log("Googler usage received:", usage);
                setGooglerUsage(usage);
                addLog('INFO', 'Received Googler usage stats');
            } else {
                addLog('WARN', 'Googler API Key not found');
            }
        } catch (err: any) {
            console.error("Failed to update Googler usage:", err);
            const errMsg = err?.message || String(err);
            addLog('ERROR', `Googler error: ${errMsg}`);
        } finally {
            setLoadingGoogler(false);
        }
    }

    return (
        <ServiceContext.Provider value={{
            openRouterBalance,
            refreshOpenRouterBalance,
            loadingOpenRouter,
            elevenLabsBotBalance,
            refreshElevenLabsBotBalance,
            loadingElevenLabsBot,
            elevenLabsUnlimBalance,
            refreshElevenLabsUnlimBalance,
            loadingElevenLabsUnlim,
            voiceMakerBalance,
            refreshVoiceMakerBalance,
            loadingVoiceMaker,
            googlerUsage,
            refreshGooglerUsage,
            loadingGoogler,
            elevenLabsBotThreshold,
            setElevenLabsBotThreshold,
            elevenLabsUnlimThreshold,
            setElevenLabsUnlimThreshold,
            voiceMakerThreshold,
            setVoiceMakerThreshold,
            openRouterThreshold,
            setOpenRouterThreshold,
            googlerVideoThreshold,
            setGooglerVideoThreshold,
            googlerImageThreshold,
            setGooglerImageThreshold,
            refreshAllBalances
        }}>
            {children}
        </ServiceContext.Provider>
    );
};
