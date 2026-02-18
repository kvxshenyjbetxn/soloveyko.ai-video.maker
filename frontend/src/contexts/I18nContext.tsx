import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import uk from '../locales/uk.json';
import en from '../locales/en.json';
import ru from '../locales/ru.json';

type Locale = 'uk' | 'en' | 'ru';

interface Translations {
    [key: string]: any;
}

const translations: Record<Locale, Translations> = {
    uk,
    en,
    ru,
};

interface I18nContextType {
    locale: Locale;
    setLocale: (locale: Locale) => void;
    t: (key: string, params?: Record<string, any>) => string;
}

const I18nContext = createContext<I18nContextType | undefined>(undefined);

export const I18nProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
    const [locale, setLocaleState] = useState<Locale>('uk');

    // Завантажуємо мову при старті
    useEffect(() => {
        const loadLanguage = async () => {
            try {
                const savedLanguage = await window.go.main.App.GetLanguage();
                setLocaleState(savedLanguage as Locale);
            } catch (error) {
                console.error('Failed to load language:', error);
            }
        };
        loadLanguage();
    }, []);

    const setLocale = async (newLocale: Locale) => {
        try {
            await window.go.main.App.SetLanguage(newLocale);
            setLocaleState(newLocale);
        } catch (error) {
            console.error('Failed to save language:', error);
        }
    };

    const t = (key: string, params?: Record<string, any>): string => {
        const keys = key.split('.');
        let value: any = translations[locale];

        for (const k of keys) {
            value = value?.[k];
        }

        if (typeof value !== 'string') return value || key;

        if (params) {
            Object.keys(params).forEach(param => {
                value = (value as string).replace(`{{${param}}}`, params[param].toString());
            });
        }

        return value;
    };

    return (
        <I18nContext.Provider value={{ locale, setLocale, t }}>
            {children}
        </I18nContext.Provider>
    );
};

export const useI18n = (): I18nContextType => {
    const context = useContext(I18nContext);
    if (!context) {
        throw new Error('useI18n must be used within I18nProvider');
    }
    return context;
};
