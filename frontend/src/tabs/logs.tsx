import { useI18n } from '../contexts/I18nContext';

export const Logs = () => {
    const { t } = useI18n();

    return (
        <div className="content-wrapper">
            <div className="settings-container">
                <h2 className="settings-title">{t('tabs.logs')}</h2>
                <div className="logs-container" style={{
                    backgroundColor: 'var(--bg-secondary)',
                    border: '1px solid var(--border-color)',
                    borderRadius: '4px',
                    padding: '16px',
                    fontFamily: 'var(--font-mono)',
                    fontSize: '12px',
                    color: 'var(--text-secondary)',
                    height: '100%',
                    overflowY: 'auto'
                }}>
                    <div>[INFO] Application started</div>
                    <div>[INFO] Loader initialized</div>
                    <div>[DEBUG] System language: {t('general.ukrainian')}</div>
                </div>
            </div>
        </div>
    );
};
