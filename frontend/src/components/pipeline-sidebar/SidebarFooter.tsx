import React from 'react';
import { useI18n } from '../../contexts/I18nContext';

interface SidebarFooterProps {
    type: string;
    content: string;
    selectedTemplateIds: string[];
    templates: any[];
    setIsModalOpen: (open: boolean) => void;
}

export const SidebarFooter: React.FC<SidebarFooterProps> = ({
    type, content, selectedTemplateIds, templates, setIsModalOpen
}) => {
    const { t } = useI18n();

    const selectedCount = selectedTemplateIds.filter(id => templates.find(t => t.id === id)?.type === type).length;

    return (
        <div className="pipeline-sidebar-footer">
            <div className="footer-actions">
                <button
                    className="add-to-queue-btn"
                    onClick={() => {
                        if (content.trim()) {
                            setIsModalOpen(true);
                        }
                    }}
                >
                    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                        <line x1="12" y1="5" x2="12" y2="19"></line>
                        <line x1="5" y1="12" x2="19" y2="12"></line>
                    </svg>
                    {selectedCount > 0
                        ? `${t('pipeline.add_to_queue')} (${selectedCount})`
                        : t('pipeline.add_to_queue')}
                </button>
            </div>
        </div>
    );
};
