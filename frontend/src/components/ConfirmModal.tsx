import React, { useEffect, useRef } from 'react';
import { useI18n } from '../contexts/I18nContext';
import './ConfirmModal.css';

interface ConfirmModalProps {
    isOpen: boolean;
    onClose: () => void;
    onConfirm: () => void;
    title: string;
    message?: string;
    confirmText?: string;
    cancelText?: string;
    isDanger?: boolean;
    type?: 'warning' | 'info' | 'error';
    extraAction?: () => void;
    extraText?: string;
}

export const ConfirmModal: React.FC<ConfirmModalProps> = ({
    isOpen,
    onClose,
    onConfirm,
    title,
    message,
    confirmText,
    cancelText,
    isDanger = true,
    type,
    extraAction,
    extraText
}) => {
    const { t } = useI18n();
    const modalRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const handleEscape = (e: KeyboardEvent) => {
            if (e.key === 'Escape') onClose();
            if (e.key === 'Enter') onConfirm();
        };

        if (isOpen) {
            window.addEventListener('keydown', handleEscape);
            document.body.style.overflow = 'hidden';
        }

        return () => {
            window.removeEventListener('keydown', handleEscape);
            document.body.style.overflow = '';
        };
    }, [isOpen, onClose, onConfirm]);

    if (!isOpen) return null;

    return (
        <div className="confirm-modal-overlay" onClick={onClose}>
            <div
                className="confirm-modal-container animate-modal-in"
                onClick={e => e.stopPropagation()}
                ref={modalRef}
            >
                <div className="confirm-modal-header">
                    <div className={`confirm-icon-circle ${isDanger ? 'danger' : ''} ${type || ''}`}>
                        {type === 'info' ? (
                            <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                                <circle cx="12" cy="12" r="10"></circle>
                                <line x1="12" y1="16" x2="12" y2="12"></line>
                                <line x1="12" y1="8" x2="12.01" y2="8"></line>
                            </svg>
                        ) : (
                            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                                <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"></path>
                                <line x1="12" y1="9" x2="12" y2="13"></line>
                                <line x1="12" y1="17" x2="12.01" y2="17"></line>
                            </svg>
                        )}
                    </div>
                    <h3>{title}</h3>
                </div>

                {message && <div className="confirm-modal-body">{message}</div>}

                <div className="confirm-modal-footer">
                    <button className="confirm-btn-cancel" onClick={onClose}>
                        {cancelText || t('common.cancel')}
                    </button>
                    {extraAction && (
                        <button className="confirm-btn-extra" onClick={extraAction}>
                            {extraText}
                        </button>
                    )}
                    <button className={`confirm-btn-action ${isDanger ? 'danger' : ''}`} onClick={onConfirm}>
                        {confirmText || t('common.delete')}
                    </button>
                </div>
            </div>
        </div>
    );
};
