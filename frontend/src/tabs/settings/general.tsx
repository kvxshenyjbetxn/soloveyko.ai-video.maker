import { useRef } from 'react';
import { useI18n } from '../../contexts/I18nContext';
import { useTheme } from '../../contexts/ThemeContext';
import './general.css';

export const General = () => {
    const { t, locale, setLocale } = useI18n();
    const { theme, setTheme, accentColor, setAccentColor } = useTheme();
    const colorInputRef = useRef<HTMLInputElement>(null);

    const presets = ['#0078d4', '#ff4500', '#32cd32', '#9370db', '#ff1493', '#ffd700', '#ffffff'];

    return (
        <div className="content-wrapper animate-fade">
            <div className="settings-container">
                <div className="settings-section">
                    <h3 className="section-title">{t('general.theme')}</h3>
                    <div className="language-selector">
                        <div
                            className={`language-option ${theme === 'dark' ? 'active' : ''}`}
                            onClick={() => setTheme('dark')}
                        >
                            <span className="language-name">{t('general.themeDark')}</span>
                        </div>
                        <div
                            className={`language-option ${theme === 'amoled' ? 'active' : ''}`}
                            onClick={() => setTheme('amoled')}
                        >
                            <span className="language-name">{t('general.themeAmoled')}</span>
                        </div>
                    </div>
                </div>

                <div className="settings-section">
                    <h3 className="section-title">{t('general.accentColor')}</h3>
                    <div className="accent-palette">
                        {presets.map(color => (
                            <div
                                key={color}
                                className={`accent-color-circle ${accentColor === color ? 'active' : ''}`}
                                style={{ backgroundColor: color }}
                                onClick={() => setAccentColor(color)}
                            />
                        ))}

                        <div
                            className={`accent-color-circle custom-picker ${!presets.includes(accentColor) ? 'active' : ''}`}
                            style={{ backgroundColor: !presets.includes(accentColor) ? accentColor : 'transparent' }}
                            onClick={() => colorInputRef.current?.click()}
                        >
                            {!presets.includes(accentColor) ? null : <span className="plus">+</span>}
                        </div>

                        <input
                            type="color"
                            ref={colorInputRef}
                            style={{ display: 'none' }}
                            value={accentColor}
                            onChange={(e) => setAccentColor(e.target.value)}
                        />
                    </div>
                </div>

                <div className="settings-section">
                    <h3 className="section-title">{t('general.language')}</h3>

                    <div className="language-selector">
                        <div
                            className={`language-option ${locale === 'uk' ? 'active' : ''}`}
                            onClick={() => setLocale('uk')}
                        >
                            <span className="language-name">{t('general.ukrainian')}</span>
                        </div>

                        <div
                            className={`language-option ${locale === 'en' ? 'active' : ''}`}
                            onClick={() => setLocale('en')}
                        >
                            <span className="language-name">{t('general.english')}</span>
                        </div>

                        <div
                            className={`language-option ${locale === 'ru' ? 'active' : ''}`}
                            onClick={() => setLocale('ru')}
                        >
                            <span className="language-name">{t('general.russian')}</span>
                        </div>
                    </div>
                </div>

                <div className="settings-section">
                    <div className="settings-controls">
                        <button
                            className="btn-secondary"
                            onClick={() => window.go.main.App.OpenConfigDir()}
                        >
                            {t('general.openConfigDir')}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
};
