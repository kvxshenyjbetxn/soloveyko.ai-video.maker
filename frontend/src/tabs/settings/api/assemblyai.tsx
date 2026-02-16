import { useI18n } from '../../../contexts/I18nContext';

export const AssemblyAI = () => {
    const { t } = useI18n();

    return (
        <div className="content-wrapper">
            <div className="settings-container">
                <h2 className="settings-title">{t('api.assemblyai')}</h2>
            </div>
        </div>
    );
};
