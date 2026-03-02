// @ts-nocheck
import React, { useEffect, useRef, useState, useMemo } from 'react';
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

    // Fatal check to prevent crash on bad data early
    if (!isOpen || !data || !Array.isArray(data)) {
        return null;
    }

    const safeT = (key: string, def: string) => {
        try {
            const val = t(key);
            if (typeof val === 'string' && val) return val;
            return def;
        } catch (e) {
            return def;
        }
    };

    // Collect all unique stages found across all tasks with extreme safety
    const allFoundStages = useMemo(() => {
        const stages = new Set<string>();
        try {
            data.forEach(item => {
                if (item && item.foundStages && Array.isArray(item.foundStages)) {
                    item.foundStages.forEach((s: any) => {
                        if (typeof s === 'string' && s) stages.add(s);
                    });
                }
            });
        } catch (e) {
            console.error("Error calculating allFoundStages:", e);
        }
        return Array.from(stages);
    }, [data]);

    const [selectedStages, setSelectedStages] = useState<string[]>([]);

    // Initialize selected stages once data is ready
    useEffect(() => {
        if (allFoundStages && allFoundStages.length > 0) {
            setSelectedStages([...allFoundStages]);
        }
    }, [allFoundStages]);

    const toggleStage = (stage: string) => {
        if (!stage) return;
        setSelectedStages(prev => {
            const current = Array.isArray(prev) ? prev : [];
            const isRemoving = current.includes(stage);

            let next: string[];
            if (isRemoving) {
                next = current.filter(s => s !== stage);

                // If we remove TEXT, we MUST remove everything that depends on it
                if (stage === 'text') {
                    next = next.filter(s => s !== 'voice' && s !== 'subtitle');
                }

                // Dependency: If we remove voice, we MUST remove subtitles
                // because subtitles depend on the exact voice timing
                if (stage === 'voice') {
                    next = next.filter(s => s !== 'subtitle');
                }
            } else {
                next = [...current, stage];

                // If we enable subtitles, we MUST enable voice and text
                if (stage === 'subtitle') {
                    if (allFoundStages.includes('voice') && !next.includes('voice')) next.push('voice');
                    if (allFoundStages.includes('text') && !next.includes('text')) next.push('text');
                }

                // If we enable voice, we MUST enable text
                if (stage === 'voice') {
                    if (allFoundStages.includes('text') && !next.includes('text')) next.push('text');
                }
            }
            // Use Set to ensure uniqueness when adding multiple
            return Array.from(new Set(next));
        });
    };

    useEffect(() => {
        const handleEscape = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                e.preventDefault();
                onCancel();
            }
            if (e.key === 'Enter') {
                e.preventDefault();
                onConfirm(selectedStages || []);
            }
        };

        if (isOpen) {
            window.addEventListener('keydown', handleEscape);
            document.body.style.overflow = 'hidden';
        }

        return () => {
            window.removeEventListener('keydown', handleEscape);
            document.body.style.overflow = '';
        };
    }, [isOpen, onCancel, onConfirm, selectedStages]);

    return (
        <div className="confirm-modal-overlay" onClick={() => onCancel()}>
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
                        <h3 style={{ margin: 0 }}>{safeT('queue.existing_files_found', "Знайдено існуючі файли")}</h3>
                        <p style={{ margin: '4px 0 0 0', fontSize: '11px', opacity: 0.6 }}>
                            {data.length > 1 ? `Завдань: ${data.length}` : ""}
                        </p>
                    </div>
                </div>

                <div className="confirm-modal-body" style={{ maxHeight: '350px', overflowY: 'auto' }}>
                    <p style={{ marginBottom: '16px', fontSize: '13px', lineHeight: '1.4', opacity: 0.9 }}>
                        {safeT('queue.existing_files_message', "Деякі файли вже існують. Виберіть, що відновити:")}
                    </p>

                    <div className="existing-tasks-list" style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                        {data.map((item, idx) => {
                            if (!item) return null;
                            const itemStages = Array.isArray(item.foundStages) ? item.foundStages : [];

                            return (
                                <div key={`item-${idx}`} className="batch-existing-item" style={{
                                    background: 'rgba(255,255,255,0.03)',
                                    border: '1px solid rgba(255,255,255,0.06)',
                                    borderRadius: '8px',
                                    padding: '8px 12px'
                                }}>
                                    <div style={{
                                        fontWeight: 700, fontSize: '12px', color: 'var(--accent-primary)',
                                        marginBottom: '8px', display: 'flex', alignItems: 'center', gap: '6px'
                                    }}>
                                        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>
                                        {item.id || item.subName || `Task ${idx + 1}`}
                                    </div>

                                    <div style={{ display: 'flex', gap: '6px', flexWrap: 'wrap' }}>
                                        {itemStages.map((stage: string) => {
                                            if (!stage) return null;
                                            const isSelected = Array.isArray(selectedStages) && selectedStages.includes(stage);
                                            let icon = "📝";
                                            let color = "rgba(63, 81, 181, 0.15)";
                                            let borderColor = "rgba(63, 81, 181, 0.2)";
                                            let detailsStr = "";
                                            let detailsNode = null;

                                            if (stage === 'voice') {
                                                icon = "🎙️";
                                                color = "rgba(156, 39, 176, 0.15)";
                                                borderColor = "rgba(156, 39, 176, 0.2)";
                                                if (item.voiceDuration) detailsStr = String(item.voiceDuration);
                                            } else if (stage === 'image') {
                                                icon = "🖼️";
                                                color = "rgba(76, 175, 80, 0.15)";
                                                borderColor = "rgba(76, 175, 80, 0.2)";
                                                detailsNode = (
                                                    <div style={{ display: 'flex', gap: '4px', opacity: 0.8, fontSize: '9px' }}>
                                                        {Number(item.promptCount) > 0 && <span>P:{item.promptCount}</span>}
                                                        {Number(item.imageCount) > 0 && <span>I:{item.imageCount}</span>}
                                                        {Number(item.videoCount) > 0 && <span>V:{item.videoCount}</span>}
                                                    </div>
                                                );
                                            } else if (stage === 'subtitle') {
                                                icon = "💬";
                                                color = "rgba(255, 152, 0, 0.15)";
                                                borderColor = "rgba(255, 152, 0, 0.2)";
                                            } else if (stage === 'text') {
                                                if (item.textChars) detailsStr = `${item.textChars} ${safeT('queue.chars_short', 'симв.')}`;
                                            }

                                            return (
                                                <div
                                                    key={`${idx}-${stage}`}
                                                    onClick={(e) => { e.stopPropagation(); toggleStage(stage); }}
                                                    style={{
                                                        fontSize: '10px',
                                                        background: isSelected ? color : 'rgba(255,255,255,0.05)',
                                                        padding: '4px 10px',
                                                        borderRadius: '6px',
                                                        border: `1px solid ${isSelected ? borderColor : 'rgba(255,255,255,0.1)'}`,
                                                        display: 'flex',
                                                        alignItems: 'center',
                                                        gap: '8px',
                                                        cursor: 'pointer',
                                                        transition: 'all 0.1s ease',
                                                        opacity: isSelected ? 1 : 0.4,
                                                        textDecoration: isSelected ? 'none' : 'line-through',
                                                        userSelect: 'none'
                                                    }}
                                                >
                                                    <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
                                                        <span>{icon}</span>
                                                        <span style={{ fontWeight: 600 }}>
                                                            {stage === 'voice' ? safeT('stages.voiceover', "Озвучка") :
                                                                stage === 'image' ? safeT('stages.image', "Зображення") :
                                                                    stage === 'subtitle' ? safeT('stages.subtitles', "Субтитри") :
                                                                        stage === 'text' ? safeT('tabs.text', "Текст") :
                                                                            stage === 'custom' ? safeT('pipeline.custom_stages.title', "Кастомні етапи") : stage}
                                                        </span>
                                                    </div>

                                                    {(detailsStr || detailsNode) && (
                                                        <div style={{ borderLeft: '1px solid rgba(255,255,255,0.1)', paddingLeft: '8px' }}>
                                                            {detailsNode || <span>{detailsStr}</span>}
                                                        </div>
                                                    )}

                                                    <span style={{ marginLeft: '4px', opacity: 0.5, fontSize: '12px' }}>{isSelected ? '✕' : '+'}</span>
                                                </div>
                                            );
                                        })}
                                    </div>
                                </div>
                            );
                        })}
                    </div>
                </div>

                <div className="confirm-modal-footer" style={{ marginTop: '20px', display: 'flex', gap: '10px', justifyContent: 'flex-end' }}>
                    <button className="confirm-btn-cancel" onClick={() => onCancel()} style={{ padding: '8px 16px', fontSize: '13px' }}>
                        {safeT('common.no_all', "Ні, переробити все")}
                    </button>
                    <button className="confirm-btn-action info" onClick={() => onConfirm(selectedStages || [])} style={{ padding: '8px 16px', fontSize: '13px' }}>
                        {selectedStages.length === allFoundStages.length ? safeT('common.yes', "Так, відновити") : "Відновити обрані"}
                    </button>
                </div>
            </div>
        </div>
    );
};
