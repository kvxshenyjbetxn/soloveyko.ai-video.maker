import React, { useState, useEffect, useRef } from 'react';
import './TaskNameModal.css';
import { useI18n } from '../contexts/I18nContext';

interface TaskNameModalProps {
    isOpen: boolean;
    onClose: () => void;
    onConfirm: (name: string) => void;
    defaultName?: string;
}

export const TaskNameModal: React.FC<TaskNameModalProps> = ({ isOpen, onClose, onConfirm, defaultName = "" }) => {
    const { t } = useI18n();
    const [name, setName] = useState("");
    const inputRef = useRef<HTMLInputElement>(null);

    useEffect(() => {
        if (isOpen) {
            setName(defaultName || "");
            setTimeout(() => inputRef.current?.focus(), 100);
        }
    }, [isOpen, defaultName]);

    if (!isOpen) return null;

    const handleConfirm = () => {
        onConfirm(name);
        onClose();
    };

    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === 'Enter') {
            handleConfirm();
        } else if (e.key === 'Escape') {
            onClose();
        }
    };

    return (
        <div className="modal-overlay" onClick={onClose}>
            <div className="task-name-modal" onClick={e => e.stopPropagation()}>
                <div className="modal-header">
                    <h3>{t('pipeline.task_name_title') || 'Назва завдання'}</h3>
                    <button className="modal-close" onClick={onClose}>&times;</button>
                </div>
                <div className="modal-body">
                    <p>{t('pipeline.task_name_description') || 'Введіть назву для цього завдання або залиште порожнім для авто-назви'}</p>
                    <input
                        ref={inputRef}
                        type="text"
                        className="modal-input"
                        value={name}
                        onChange={e => setName(e.target.value)}
                        onKeyDown={handleKeyDown}
                        placeholder={defaultName || "Task name..."}
                        autoFocus
                    />
                </div>
                <div className="modal-footer">
                    <button className="modal-btn secondary" onClick={onClose}>
                        {t('common.cancel') || 'Скасувати'}
                    </button>
                    <button className="modal-btn primary" onClick={handleConfirm}>
                        {t('common.add') || 'Додати'}
                    </button>
                </div>
            </div>
        </div>
    );
};
