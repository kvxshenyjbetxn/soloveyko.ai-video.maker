import { useState } from 'react';
import './AuthModal.css';
import { useAuth } from '../contexts/AuthContext';

const mapAuthError = (code: string) => {
    switch (code) {
        case 'auth/invalid-email':
            return 'Невірний формат email.';
        case 'auth/invalid-credential':
        case 'auth/user-not-found':
        case 'auth/wrong-password':
            return 'Невірний email або пароль.';
        case 'auth/email-already-in-use':
            return 'Цей email вже використовується.';
        case 'auth/weak-password':
            return 'Пароль має бути мінімум 6 символів.';
        default:
            return 'Сталася помилка авторизації. Спробуйте ще раз.';
    }
};

export const AuthModal = () => {
    const { signIn, signUp, isLoading } = useAuth();
    const [isRegisterMode, setIsRegisterMode] = useState(false);
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [rememberMe, setRememberMe] = useState(true);
    const [isSubmitting, setIsSubmitting] = useState(false);
    const [errorText, setErrorText] = useState('');

    const handleSubmit = async (event: React.FormEvent) => {
        event.preventDefault();

        const normalizedEmail = email.trim();
        if (!normalizedEmail || !password) {
            setErrorText('Вкажіть email і пароль.');
            return;
        }

        setErrorText('');
        setIsSubmitting(true);

        try {
            if (isRegisterMode) {
                await signUp(normalizedEmail, password, rememberMe);
            } else {
                await signIn(normalizedEmail, password, rememberMe);
            }
            setPassword('');
        } catch (error) {
            const code = (error as { code?: string }).code ?? '';
            setErrorText(mapAuthError(code));
        } finally {
            setIsSubmitting(false);
        }
    };

    return (
        <div className="auth-modal-overlay">
            <div className="auth-modal-container animate-modal-in">
                <h2>{isRegisterMode ? 'Реєстрація' : 'Вхід у програму'}</h2>
                <p className="auth-modal-subtitle">
                    {isRegisterMode
                        ? 'Створіть обліковий запис за email і паролем'
                        : 'Увійдіть за допомогою email і пароля'}
                </p>

                <form onSubmit={handleSubmit} className="auth-form">
                    <label className="auth-label">
                        Email
                        <input
                            type="email"
                            autoComplete="email"
                            value={email}
                            onChange={(e) => setEmail(e.target.value)}
                            disabled={isSubmitting || isLoading}
                        />
                    </label>

                    <label className="auth-label">
                        Пароль
                        <input
                            type="password"
                            autoComplete={isRegisterMode ? 'new-password' : 'current-password'}
                            value={password}
                            onChange={(e) => setPassword(e.target.value)}
                            disabled={isSubmitting || isLoading}
                        />
                    </label>

                    <label className="auth-checkbox">
                        <input
                            type="checkbox"
                            checked={rememberMe}
                            onChange={(e) => setRememberMe(e.target.checked)}
                            disabled={isSubmitting || isLoading}
                        />
                        <span>Запамʼятати мене (автовхід)</span>
                    </label>

                    {errorText && <div className="auth-error">{errorText}</div>}

                    <button
                        className="auth-submit-btn"
                        type="submit"
                        disabled={isSubmitting || isLoading}
                    >
                        {isSubmitting
                            ? 'Зачекайте...'
                            : isRegisterMode
                                ? 'Зареєструватись'
                                : 'Увійти'}
                    </button>
                </form>

                <button
                    className="auth-switch-mode"
                    type="button"
                    onClick={() => {
                        setIsRegisterMode((prev) => !prev);
                        setErrorText('');
                    }}
                    disabled={isSubmitting || isLoading}
                >
                    {isRegisterMode
                        ? 'Вже є акаунт? Увійти'
                        : 'Немає акаунта? Зареєструватися'}
                </button>
            </div>
        </div>
    );
};
