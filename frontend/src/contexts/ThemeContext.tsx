import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';

type Theme = 'dark' | 'amoled';

interface ThemeContextType {
    theme: Theme;
    setTheme: (theme: Theme) => void;
    accentColor: string;
    setAccentColor: (color: string) => void;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

declare global {
    interface Window {
        go: {
            main: {
                App: {
                    GetLanguage: () => Promise<string>;
                    SetLanguage: (language: string) => Promise<void>;
                    OpenConfigDir: () => Promise<void>;
                    GetTheme: () => Promise<string>;
                    SetTheme: (theme: string) => Promise<void>;
                    GetAccentColor: () => Promise<string>;
                    SetAccentColor: (color: string) => Promise<void>;
                };
            };
        };
    }
}

export const ThemeProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
    const [theme, setThemeState] = useState<Theme>('dark');
    const [accentColor, setAccentColorState] = useState<string>('#0078d4');

    useEffect(() => {
        const loadSettings = async () => {
            try {
                const [savedTheme, savedAccent] = await Promise.all([
                    window.go.main.App.GetTheme(),
                    window.go.main.App.GetAccentColor()
                ]);
                setThemeState(savedTheme as Theme);
                setAccentColorState(savedAccent);
            } catch (error) {
                console.error('Failed to load theme settings:', error);
            }
        };
        loadSettings();
    }, []);

    const setTheme = async (newTheme: Theme) => {
        try {
            await window.go.main.App.SetTheme(newTheme);
            setThemeState(newTheme);
        } catch (error) {
            console.error('Failed to save theme:', error);
        }
    };

    const setAccentColor = async (newColor: string) => {
        try {
            await window.go.main.App.SetAccentColor(newColor);
            setAccentColorState(newColor);
        } catch (error) {
            console.error('Failed to save accent color:', error);
        }
    };

    useEffect(() => {
        // Apply theme class to body
        document.body.className = `theme-${theme}`;
        // Apply accent color as CSS variable
        document.documentElement.style.setProperty('--accent-primary', accentColor);
        document.documentElement.style.setProperty('--accent-color', accentColor);
        // Calculate a hover color (simpler version: same color for now or slightly transparent)
        document.documentElement.style.setProperty('--accent-hover', accentColor + 'ee');
    }, [theme, accentColor]);

    return (
        <ThemeContext.Provider value={{ theme, setTheme, accentColor, setAccentColor }}>
            {children}
        </ThemeContext.Provider>
    );
};

export const useTheme = (): ThemeContextType => {
    const context = useContext(ThemeContext);
    if (!context) {
        throw new Error('useTheme must be used within ThemeProvider');
    }
    return context;
};
