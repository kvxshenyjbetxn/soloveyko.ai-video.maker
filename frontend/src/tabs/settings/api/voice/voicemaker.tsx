import { useI18n } from '../../../../contexts/I18nContext';

export const VoiceMaker = () => {
    const { t } = useI18n();

    return (
        <div className="content-wrapper">
            <div className="settings-container">
                <h2 className="settings-title">{t('voice.voicemaker')}</h2>
            </div>
        </div>
    );
};
