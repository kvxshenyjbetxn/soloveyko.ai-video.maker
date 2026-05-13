const STORAGE_KEY = 'soloveyko.deviceId.v1';

export const getOrCreateDeviceId = (): string => {
    try {
        const existing = localStorage.getItem(STORAGE_KEY);
        if (existing) {
            return existing;
        }
        const id = crypto.randomUUID();
        localStorage.setItem(STORAGE_KEY, id);
        return id;
    } catch {
        return `ephemeral-${Math.random().toString(36).slice(2)}`;
    }
};

export const defaultDeviceLabel = (deviceId: string): string => {
    const short = deviceId.replace(/-/g, '').slice(0, 8);
    return `PC ${short}`;
};
