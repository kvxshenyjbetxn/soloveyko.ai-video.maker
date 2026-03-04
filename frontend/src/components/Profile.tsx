import React, { useState, useRef, useEffect } from 'react';
import { useI18n } from '../contexts/I18nContext';
import { api } from '../../wailsjs/go/models';
import './Profile.css';

interface ProfileProps {
    authResponse: api.AuthResponse | null;
    onLogout: () => void;
}

const UserIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"></path>
        <circle cx="12" cy="7" r="4"></circle>
    </svg>
);

const LogOutIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"></path>
        <polyline points="16 17 21 12 16 7"></polyline>
        <line x1="21" y1="12" x2="9" y2="12"></line>
    </svg>
);

const CopyIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
        <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
    </svg>
);

const CheckIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
        <polyline points="20 6 9 17 4 12"></polyline>
    </svg>
);

export const Profile: React.FC<ProfileProps> = ({ authResponse, onLogout }) => {
    const { t } = useI18n();
    const [isOpen, setIsOpen] = useState(false);
    const [copied, setCopied] = useState(false);
    const dropdownRef = useRef<HTMLDivElement>(null);

    const handleCopy = (e: React.MouseEvent) => {
        e.stopPropagation();
        if (authResponse?.telegram_id) {
            navigator.clipboard.writeText(authResponse.telegram_id.toString());
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        }
    };

    // Close on click outside
    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
                setIsOpen(false);
            }
        };

        if (isOpen) {
            document.addEventListener('mousedown', handleClickOutside);
        }

        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
        };
    }, [isOpen]);

    if (!authResponse) return null;

    // Calculate days left
    let daysLeftDisplay = t('auth.expired');
    if (authResponse.is_unlimited) {
        daysLeftDisplay = t('auth.unlimited');
    } else if (authResponse.expires_at) {
        // Assume expires_at is ISO string or easily parseable date
        const expiresDate = new Date(authResponse.expires_at);
        const now = new Date();
        const diffTime = expiresDate.getTime() - now.getTime();
        const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));

        if (diffDays > 0) {
            daysLeftDisplay = t('auth.days_left').replace('{{days}}', diffDays.toString());
        }
    }

    return (
        <div className="profile-container" ref={dropdownRef}>
            <div className="profile-trigger" onClick={() => setIsOpen(!isOpen)}>
                <UserIcon />
            </div>

            {isOpen && (
                <div className="profile-dropdown animate-fade">
                    <div className="profile-header">
                        <h4>{t('auth.profile')}</h4>
                    </div>
                    <div className="profile-content">
                        <div className="profile-item subscription">
                            <span className="profile-label">Subscription:</span>
                            <span className="profile-value highlight-value">{daysLeftDisplay}</span>
                        </div>
                        <div className="profile-item telegram">
                            <span className="profile-label">Telegram ID:</span>
                            <div className="profile-value-wrapper">
                                <span className="profile-value">{authResponse.telegram_id || 'N/A'}</span>
                                {authResponse.telegram_id && (
                                    <button
                                        className={`profile-copy-btn ${copied ? 'copied' : ''}`}
                                        onClick={handleCopy}
                                        title="Copy Telegram ID"
                                    >
                                        {copied ? <CheckIcon /> : <CopyIcon />}
                                    </button>
                                )}
                            </div>
                        </div>
                    </div>
                    <div className="profile-footer">
                        <button className="logout-button" onClick={onLogout}>
                            <LogOutIcon />
                            <span>{t('auth.logout')}</span>
                        </button>
                    </div>
                </div>
            )}
        </div>
    );
};
