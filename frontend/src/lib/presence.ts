import { onValue, onDisconnect, ref, serverTimestamp, set, get } from 'firebase/database';
import type { User } from 'firebase/auth';
import { doc, getDoc, serverTimestamp as fsServerTimestamp, setDoc } from 'firebase/firestore';
import { realtimeDb, firestore, refreshAuthForFirestore } from './firebase';
import { defaultDeviceLabel, getOrCreateDeviceId } from './deviceId';

type PresenceState = {
    state: 'online' | 'offline';
    lastChanged: object;
    name?: string;
};

const resolveDeviceDisplayName = async (user: User, deviceId: string): Promise<string> => {
    const deviceRef = doc(firestore, 'users', user.uid, 'devices', deviceId);
    const snap = await getDoc(deviceRef);
    if (snap.exists()) {
        const data = snap.data();
        const raw = data?.name;
        if (typeof raw === 'string' && raw.trim()) {
            return raw.trim().slice(0, 64);
        }
    }
    return defaultDeviceLabel(deviceId);
};

async function touchFirestoreDevice(user: User, deviceId: string): Promise<void> {
    try {
        await refreshAuthForFirestore(user);
        const deviceRef = doc(firestore, 'users', user.uid, 'devices', deviceId);
        const snap = await getDoc(deviceRef);
        const payload: Record<string, unknown> = {
            lastSeen: fsServerTimestamp(),
        };
        if (!snap.exists()) {
            payload.firstSeen = fsServerTimestamp();
            payload.name = defaultDeviceLabel(deviceId);
        }
        await setDoc(deviceRef, payload, { merge: true });
    } catch (err) {
        console.warn('Failed to register device in Firestore', err);
    }
}

export const trackUserPresence = (user: User) => {
    const deviceId = getOrCreateDeviceId();
    void touchFirestoreDevice(user, deviceId);

    const databaseUrl = import.meta.env.VITE_FIREBASE_DATABASE_URL;
    if (!databaseUrl) {
        console.warn('VITE_FIREBASE_DATABASE_URL is not set. Realtime presence is disabled.');
        return () => undefined;
    }

    const connectedRef = ref(realtimeDb, '.info/connected');
    const deviceStatusRef = ref(realtimeDb, `status/${user.uid}/devices/${deviceId}`);

    const unsubscribe = onValue(connectedRef, async (snapshot) => {
        if (!snapshot.val()) {
            return;
        }

        const label = await resolveDeviceDisplayName(user, deviceId);
        const onlineState: PresenceState = {
            state: 'online',
            lastChanged: serverTimestamp(),
            name: label,
        };
        const offlineState: PresenceState = {
            state: 'offline',
            lastChanged: serverTimestamp(),
            name: label,
        };

        await onDisconnect(deviceStatusRef).set(offlineState);
        await set(deviceStatusRef, onlineState);
    });

    return () => {
        unsubscribe();
        void (async () => {
            try {
                const snap = await get(deviceStatusRef);
                const cur = snap.val() as { name?: string } | null;
                let name: string;
                if (cur && typeof cur.name === 'string' && cur.name.trim()) {
                    name = cur.name.trim().slice(0, 64);
                } else {
                    name = await resolveDeviceDisplayName(user, deviceId);
                }
                await set(deviceStatusRef, {
                    state: 'offline',
                    lastChanged: serverTimestamp(),
                    name,
                });
            } catch (err) {
                console.warn('Presence cleanup failed', err);
            }
        })();
    };
};
