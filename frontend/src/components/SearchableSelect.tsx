import React, { useState, useRef, useEffect, useMemo } from 'react';
import './SearchableSelect.css';

interface Option {
    value: string;
    label: string;
    subLabel?: string;
}

interface SearchableSelectProps {
    options: Option[];
    value: string;
    onChange: (value: string) => void;
    placeholder?: string;
    loading?: boolean;
    disabled?: boolean;
    searchPlaceholder?: string;
}

const SearchableSelect: React.FC<SearchableSelectProps> = ({
    options,
    value,
    onChange,
    placeholder = 'Select...',
    loading = false,
    disabled = false,
    searchPlaceholder = 'Search...'
}) => {
    const [isOpen, setIsOpen] = useState(false);
    const [searchTerm, setSearchTerm] = useState('');
    const [dropUp, setDropUp] = useState(false);
    const containerRef = useRef<HTMLDivElement>(null);
    const inputRef = useRef<HTMLInputElement>(null);

    const selectedOption = useMemo(() =>
        options.find(opt => opt.value === value),
        [options, value]);

    const filteredOptions = useMemo(() => {
        if (!searchTerm) return options;
        const lowSearch = searchTerm.toLowerCase();
        return options.filter(opt =>
            opt.label.toLowerCase().includes(lowSearch) ||
            (opt.subLabel && opt.subLabel.toLowerCase().includes(lowSearch)) ||
            opt.value.toLowerCase().includes(lowSearch)
        );
    }, [options, searchTerm]);

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (containerRef.current && !containerRef.current.contains(event.target as Node)) {
                setIsOpen(false);
            }
        };
        document.addEventListener('mousedown', handleClickOutside);
        return () => document.removeEventListener('mousedown', handleClickOutside);
    }, []);

    useEffect(() => {
        if (isOpen && inputRef.current) {
            inputRef.current.focus();
            setSearchTerm('');

            // Check if we should drop up
            if (containerRef.current) {
                const rect = containerRef.current.getBoundingClientRect();
                const spaceBelow = window.innerHeight - rect.bottom;
                const dropdownHeight = 350; // Max expected height
                if (spaceBelow < dropdownHeight && rect.top > dropdownHeight) {
                    setDropUp(true);
                } else {
                    setDropUp(false);
                }
            }
        }
    }, [isOpen]);

    const handleToggle = () => {
        if (!disabled && !loading) {
            setIsOpen(!isOpen);
        }
    };

    const handleSelect = (val: string) => {
        onChange(val);
        setIsOpen(false);
    };

    return (
        <div className={`searchable-select-container ${disabled ? 'disabled' : ''} ${isOpen ? 'is-open' : ''} ${dropUp ? 'is-drop-up' : ''}`} ref={containerRef}>
            <div className="searchable-select-display" onClick={handleToggle}>
                <div className="selected-value">
                    {loading ? (
                        <span className="placeholder">Loading...</span>
                    ) : selectedOption ? (
                        <div className="option-content">
                            <span className="option-label">{selectedOption.label}</span>
                            {selectedOption.subLabel && <span className="option-sublabel">{selectedOption.subLabel}</span>}
                        </div>
                    ) : (
                        <span className="placeholder">{placeholder}</span>
                    )}
                </div>
                <div className="select-chevron">
                    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
                        <path d="m6 9 6 6 6-6" />
                    </svg>
                </div>
            </div>

            {isOpen && (
                <div className="searchable-select-dropdown">
                    {!dropUp && (
                        <div className="search-input-wrapper top">
                            <svg className="search-icon" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                                <circle cx="11" cy="11" r="8" /><path d="m21 21-4.3-4.3" />
                            </svg>
                            <input
                                ref={inputRef}
                                type="text"
                                className="search-input"
                                placeholder={searchPlaceholder}
                                value={searchTerm}
                                onChange={(e) => setSearchTerm(e.target.value)}
                            />
                        </div>
                    )}
                    <div className="options-list">
                        {filteredOptions.length > 0 ? (
                            filteredOptions.map(opt => (
                                <div
                                    key={opt.value}
                                    className={`option-item ${opt.value === value ? 'selected' : ''}`}
                                    onClick={() => handleSelect(opt.value)}
                                >
                                    <div className="option-content">
                                        <span className="option-label">{opt.label}</span>
                                        {opt.subLabel && <span className="option-sublabel">{opt.subLabel}</span>}
                                    </div>
                                    {opt.value === value && (
                                        <svg className="check-icon" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
                                            <path d="M20 6 9 17l-5-5" />
                                        </svg>
                                    )}
                                </div>
                            ))
                        ) : (
                            <div className="no-options">No matches found</div>
                        )}
                    </div>
                    {dropUp && (
                        <div className="search-input-wrapper bottom">
                            <svg className="search-icon" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                                <circle cx="11" cy="11" r="8" /><path d="m21 21-4.3-4.3" />
                            </svg>
                            <input
                                ref={inputRef}
                                type="text"
                                className="search-input"
                                placeholder={searchPlaceholder}
                                value={searchTerm}
                                onChange={(e) => setSearchTerm(e.target.value)}
                            />
                        </div>
                    )}
                </div>
            )}
        </div>
    );
};

export default SearchableSelect;
