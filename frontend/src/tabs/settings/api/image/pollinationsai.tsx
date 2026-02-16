import { useI18n } from '../../../../contexts/I18nContext';

export const PollinationsAI = () => {
    const { t } = useI18n();

    return (
        <div className="content-wrapper">
            <div className="settings-container">
                <h2 className="settings-title">{t('image.pollinationsai')}</h2>
            </div>
        </div>
    );
};
