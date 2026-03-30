import React, { createContext, useContext, useMemo, useState, ReactNode } from 'react';

type DraftTab = 'translate' | 'rewrite';

interface EditorDraftContextType {
    translateText: string;
    rewriteText: string;
    setTranslateText: React.Dispatch<React.SetStateAction<string>>;
    setRewriteText: React.Dispatch<React.SetStateAction<string>>;
    getTextForTab: (tab: DraftTab) => string;
    setTextForTab: (tab: DraftTab, text: string) => void;
}

const EditorDraftContext = createContext<EditorDraftContextType | undefined>(undefined);

export const EditorDraftProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
    const [translateText, setTranslateText] = useState("");
    const [rewriteText, setRewriteText] = useState("");

    const value = useMemo<EditorDraftContextType>(() => ({
        translateText,
        rewriteText,
        setTranslateText,
        setRewriteText,
        getTextForTab: (tab: DraftTab) => tab === 'translate' ? translateText : rewriteText,
        setTextForTab: (tab: DraftTab, text: string) => {
            if (tab === 'translate') {
                setTranslateText(text);
                return;
            }
            setRewriteText(text);
        },
    }), [translateText, rewriteText]);

    return (
        <EditorDraftContext.Provider value={value}>
            {children}
        </EditorDraftContext.Provider>
    );
};

export const useEditorDrafts = (): EditorDraftContextType => {
    const context = useContext(EditorDraftContext);
    if (!context) {
        throw new Error('useEditorDrafts must be used within EditorDraftProvider');
    }
    return context;
};
