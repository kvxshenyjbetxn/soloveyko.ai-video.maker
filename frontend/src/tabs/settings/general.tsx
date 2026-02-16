import { useI18n } from '../../contexts/I18nContext';
import './general.css';

export const General = () => {
    const { t, locale, setLocale } = useI18n();

    return (
        <div className="content-wrapper">
            <div className="settings-container">
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
                    <h3 className="section-title">{t('general.openConfigDir')}</h3>
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
