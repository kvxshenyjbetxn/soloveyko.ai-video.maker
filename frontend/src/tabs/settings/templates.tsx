import { useI18n } from '../../contexts/I18nContext';

export const Templates = () => {
    const { t } = useI18n();

    return (
        <div className="content-wrapper">
            <div className="settings-container">
                <h2 className="settings-title">{t('settings.templates')}</h2>
            </div>
        </div>
    );
};
