import { useI18n } from '../../../contexts/I18nContext';

export const OpenRouter = () => {
    const { t } = useI18n();

    return (
        <div className="content-wrapper">
            <div className="settings-container">
                <h2 className="settings-title">{t('api.openrouter')}</h2>
            </div>
        </div>
    );
};
