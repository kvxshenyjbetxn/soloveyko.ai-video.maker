import { createContext, useContext, useEffect, useMemo, useState, type ReactNode } from 'react';
import {
    browserLocalPersistence,
    browserSessionPersistence,
    createUserWithEmailAndPassword,
    onAuthStateChanged,
    setPersistence,
    signInWithEmailAndPassword,
    signOut,
    type User
} from 'firebase/auth';
import { auth } from '../lib/firebase';
import { trackUserPresence } from '../lib/presence';

interface AuthContextValue {
    user: User | null;
    isLoading: boolean;
    signIn: (email: string, password: string, rememberMe: boolean) => Promise<void>;
    signUp: (email: string, password: string, rememberMe: boolean) => Promise<void>;
    logOut: () => Promise<void>;
}

const AuthContext = createContext<AuthContextValue | undefined>(undefined);

export const AuthProvider = ({ children }: { children: ReactNode }) => {
    const [user, setUser] = useState<User | null>(null);
    const [isLoading, setIsLoading] = useState(true);

    useEffect(() => {
        const unsubscribe = onAuthStateChanged(auth, (nextUser) => {
            setUser(nextUser);
            setIsLoading(false);
        });

        return unsubscribe;
    }, []);

    useEffect(() => {
        if (!user) {
            return;
        }

        const stopTracking = trackUserPresence(user);

        return () => {
            stopTracking();
        };
    }, [user]);

    const signIn = async (email: string, password: string, rememberMe: boolean) => {
        await setPersistence(auth, rememberMe ? browserLocalPersistence : browserSessionPersistence);
        await signInWithEmailAndPassword(auth, email, password);
    };

    const signUp = async (email: string, password: string, rememberMe: boolean) => {
        await setPersistence(auth, rememberMe ? browserLocalPersistence : browserSessionPersistence);
        await createUserWithEmailAndPassword(auth, email, password);
    };

    const logOut = async () => {
        await signOut(auth);
    };

    const value = useMemo<AuthContextValue>(() => ({
        user,
        isLoading,
        signIn,
        signUp,
        logOut
    }), [user, isLoading]);

    return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};

export const useAuth = () => {
    const context = useContext(AuthContext);
    if (!context) {
        throw new Error('useAuth must be used within AuthProvider');
    }
    return context;
};
