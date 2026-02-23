import React, { useEffect, useRef } from 'react';
import { useI18n } from '../contexts/I18nContext';
import './ConfirmModal.css';

interface ExistingFilesModalProps {
    isOpen: boolean;
    data: any;
    onConfirm: (skipStages: string[]) => void;
    onCancel: () => void;
}

export const ExistingFilesModal: React.FC<ExistingFilesModalProps> = ({ isOpen, data, onConfirm, onCancel }) => {
    const { t } = useI18n();
    const modalRef = useRef<HTMLDivElement>(null);

    // Collect all unique stages found across all tasks
    const allFoundStages = new Set<string>();
    if (data && Array.isArray(data)) {
        data.forEach(item => {
            (item.foundStages || []).forEach((s: string) => allFoundStages.add(s));
        });
    }
    const totalStages = Array.from(allFoundStages);

    useEffect(() => {
        const handleEscape = (e: KeyboardEvent) => {
            if (e.key === 'Escape') onCancel();
            if (e.key === 'Enter') onConfirm(totalStages);
        };

        if (isOpen) {
            window.addEventListener('keydown', handleEscape);
            document.body.style.overflow = 'hidden';
        }

        return () => {
            window.removeEventListener('keydown', handleEscape);
            document.body.style.overflow = '';
        };
    }, [isOpen, onCancel, onConfirm, totalStages]);

    if (!isOpen || !data || !Array.isArray(data)) return null;

    return (
        <div className="confirm-modal-overlay" onClick={onCancel}>
            <div
                className="confirm-modal-container animate-modal-in"
                onClick={e => e.stopPropagation()}
                ref={modalRef}
                style={{ maxWidth: '600px', width: '90%' }}
            >
                <div className="confirm-modal-header">
                    <div className="confirm-icon-circle info">
                        <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
                        </svg>
                    </div>
                    <div>
                        <h3 style={{ margin: 0 }}>{t('queue.existing_files_found')}</h3>
                        <p style={{ margin: '4px 0 0 0', fontSize: '13px', opacity: 0.7 }}>
                            {data.length > 1 ? t('queue.existing_files_found_count', { count: data.length }) : t('queue.existing_files_found')}
                        </p>
                    </div>
                </div>

                <div className="confirm-modal-body" style={{ maxHeight: '60vh', overflowY: 'auto' }}>
                    <p style={{ marginBottom: '16px', fontSize: '14px', lineHeight: '1.5' }}>
                        {t('queue.existing_files_message')}
                    </p>

                    <div className="existing-tasks-list" style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
                        {data.map((item, idx) => (
                            <div key={idx} className="batch-existing-item" style={{
                                background: 'rgba(255,255,255,0.02)',
                                border: '1px solid rgba(255,255,255,0.05)',
                                borderRadius: '8px',
                                padding: '6px 12px',
                                display: 'flex',
                                alignItems: 'center',
                                justifyContent: 'space-between',
                                gap: '12px'
                            }}>
                                <div style={{
                                    fontWeight: 700, fontSize: '12px', color: 'var(--accent-primary)',
                                    whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis', maxWidth: '200px',
                                    display: 'flex', alignItems: 'center', gap: '6px'
                                }}>
                                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>
                                    {item.id || t('queue.task_default_name')}
                                </div>

                                <div style={{ display: 'flex', gap: '6px', flexWrap: 'wrap', justifyContent: 'flex-end' }}>
                                    {item.foundStages.includes('text') && (
                                        <div style={{ fontSize: '10px', background: 'rgba(63, 81, 181, 0.15)', padding: '2px 6px', borderRadius: '4px', border: '1px solid rgba(63, 81, 181, 0.2)', display: 'flex', alignItems: 'center', gap: '3px' }}>
                                            <span>📝</span> {item.textChars} {t('queue.chars_short')}
                                        </div>
                                    )}
                                    {item.foundStages.includes('voice') && (
                                        <div style={{ fontSize: '10px', background: 'rgba(156, 39, 176, 0.15)', padding: '2px 6px', borderRadius: '4px', border: '1px solid rgba(156, 39, 176, 0.2)', display: 'flex', alignItems: 'center', gap: '3px' }}>
                                            <span>🎙️</span> {item.voiceDuration}
                                        </div>
                                    )}
                                    {(item.foundStages.includes('image') || (item.promptCount > 0)) && (
                                        <div style={{ fontSize: '10px', background: 'rgba(76, 175, 80, 0.15)', padding: '2px 6px', borderRadius: '4px', border: '1px solid rgba(76, 175, 80, 0.2)', display: 'flex', alignItems: 'center', gap: '6px' }}>
                                            <span>🖼️</span>
                                            <div style={{ display: 'flex', gap: '4px', opacity: 0.9 }}>
                                                {item.promptCount > 0 && <span>P:{item.promptCount}</span>}
                                                {item.imageCount > 0 && <span>I:{item.imageCount}</span>}
                                                {item.videoCount > 0 && <span>V:{item.videoCount}</span>}
                                            </div>
                                        </div>
                                    )}
                                    {item.foundStages.includes('subtitle') && (
                                        <div style={{ fontSize: '10px', background: 'rgba(255, 152, 0, 0.15)', padding: '2px 6px', borderRadius: '4px', border: '1px solid rgba(255, 152, 0, 0.2)' }}>
                                            <span>💬</span> {t('common.ready') || 'SRT'}
                                        </div>
                                    )}
                                </div>
                            </div>
                        ))}
                    </div>
                </div>

                <div className="confirm-modal-footer" style={{ marginTop: '24px' }}>
                    <button className="confirm-btn-cancel" onClick={onCancel}>
                        {t('common.no')}
                    </button>
                    <button className="confirm-btn-action info" onClick={() => onConfirm(totalStages)}>
                        {t('common.yes')}
                    </button>
                </div>
            </div>
        </div>
    );
};
