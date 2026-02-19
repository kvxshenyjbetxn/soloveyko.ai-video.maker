import React, { createContext, useContext, useState, useEffect, useRef } from 'react';
// @ts-ignore
import { GetOpenRouterCredits, GetOpenRouterAPIKey, GetOpenRouterKeys, GetElevenLabsBotBalance, GetElevenLabsBotAPIKey, GetElevenLabsUnlimBalance, GetElevenLabsUnlimAPIKey, GetVoiceMakerBalance, GetVoiceMakerAPIKey, GetVoiceMakerSavedBalance, SaveVoiceMakerBalance, GetGooglerUsage, GetGooglerAPIKey } from '../../wailsjs/go/main/App';
import { useLogger } from './LoggerContext';

interface ServiceContextType {
    openRouterBalances: Record<string, number | null>;
    openRouterKeys: any[];
    refreshOpenRouterBalance: () => Promise<void>;
    loadingOpenRouter: boolean;
    elevenLabsBotBalances: Record<string, number | null>;
    elevenLabsBotKeys: any[];
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
    openRouterBalances: {},
    openRouterKeys: [],
    refreshOpenRouterBalance: async () => { },
    loadingOpenRouter: false,
    elevenLabsBotBalances: {},
    elevenLabsBotKeys: [],
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
    const [openRouterBalances, setOpenRouterBalances] = useState<Record<string, number | null>>({});
    const [openRouterKeys, setOpenRouterKeys] = useState<any[]>([]);
    const [loadingOpenRouter, setLoadingOpenRouter] = useState(false);
    const [elevenLabsBotBalances, setElevenLabsBotBalances] = useState<Record<string, number | null>>({});
    const [elevenLabsBotKeys, setElevenLabsBotKeys] = useState<any[]>([]);
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
        try {
            const keys = await GetOpenRouterKeys();
            setOpenRouterKeys(keys || []);
            if (keys && keys.length > 0) {
                const newBalances: Record<string, number | null> = {};
                await Promise.all(keys.map(async (item: any) => {
                    try {
                        const credit = await GetOpenRouterCredits(item.key);
                        newBalances[item.id] = Math.max(0, credit);
                    } catch (e) {
                        newBalances[item.id] = null;
                    }
                }));
                setOpenRouterBalances(newBalances);
            } else {
                setOpenRouterBalances({});
                addLog('WARN', 'No OpenRouter API Keys found');
            }
        } catch (err: any) {
            console.error("Failed to update balances:", err);
        } finally {
            setLoadingOpenRouter(false);
        }
    };

    const refreshElevenLabsBotBalance = async () => {
        if (loadingElevenLabsBot) return;

        setLoadingElevenLabsBot(true);
        try {
            // @ts-ignore
            const { GetElevenLabsBotKeys } = window.go.main.App;
            const keys = await GetElevenLabsBotKeys();
            setElevenLabsBotKeys(keys || []);

            if (keys && keys.length > 0) {
                const newBalances: Record<string, number | null> = {};
                await Promise.all(keys.map(async (item: any) => {
                    try {
                        const balance = await GetElevenLabsBotBalance(item.key);
                        newBalances[item.id] = balance;
                    } catch (e) {
                        newBalances[item.id] = null;
                    }
                }));
                setElevenLabsBotBalances(newBalances);
            } else {
                setElevenLabsBotBalances({});
                addLog('WARN', 'No ElevenLabsBot API Keys found');
            }
        } catch (err: any) {
            console.error("Failed to update ElevenLabsBot balance:", err);
        } finally {
            setLoadingElevenLabsBot(false);
        }
    }

    const refreshElevenLabsUnlimBalance = async () => {
        if (loadingElevenLabsUnlim) return;

        setLoadingElevenLabsUnlim(true);
        try {
            const apiKey = await GetElevenLabsUnlimAPIKey();
            if (apiKey) {
                const balance = await GetElevenLabsUnlimBalance(apiKey);
                setElevenLabsUnlimBalance(balance);
            } else {
                setElevenLabsUnlimBalance(null);
                addLog('WARN', 'ElevenLabsUnlim API Key not found');
            }
        } catch (err: any) {
            console.error("Failed to update ElevenLabsUnlim balance:", err);
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
        try {
            const apiKey = await GetVoiceMakerAPIKey();
            if (apiKey) {
                const balance = await GetVoiceMakerBalance(apiKey);
                setVoiceMakerBalance(balance);
                await SaveVoiceMakerBalance(balance); // Зберігаємо в налаштування
            } else {
                setVoiceMakerBalance(null);
                addLog('WARN', 'VoiceMaker API Key not found');
            }
        } catch (err: any) {
            console.error("Failed to update VoiceMaker balance:", err);
        } finally {
            setLoadingVoiceMaker(false);
        }
    }

    const refreshGooglerUsage = async () => {
        if (loadingGoogler) return;

        setLoadingGoogler(true);
        try {
            const apiKey = await GetGooglerAPIKey();
            if (apiKey) {
                const usage = await GetGooglerUsage(apiKey);
                console.log("Googler usage received:", usage);
                setGooglerUsage(usage);
            } else {
                addLog('WARN', 'Googler API Key not found');
            }
        } catch (err: any) {
            console.error("Failed to update Googler usage:", err);
            const errMsg = err?.message || String(err);
        } finally {
            setLoadingGoogler(false);
        }
    }

    return (
        <ServiceContext.Provider value={{
            openRouterBalances,
            openRouterKeys,
            refreshOpenRouterBalance,
            loadingOpenRouter,
            elevenLabsBotBalances,
            elevenLabsBotKeys,
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
