import { useEffect, useMemo, useState } from 'react';
import { onSnapshot, collection, doc, setDoc, serverTimestamp } from 'firebase/firestore';
import { onValue, ref, update, onDisconnect, serverTimestamp as rtdbServerTimestamp } from 'firebase/database';
import type { User } from 'firebase/auth';
import { firestore, realtimeDb, refreshAuthForFirestore } from './firebase';
import { defaultDeviceLabel, getOrCreateDeviceId } from './deviceId';

export type DeviceRow = {
    deviceId: string;
    name: string;
    state: 'online' | 'offline';
    lastChanged: number | null;
    isCurrent: boolean;
};

type FirestoreDeviceMeta = {
    name?: string;
};

type RtDeviceState = {
    state?: string;
    lastChanged?: number | { '.sv'?: string };
    name?: string;
};

const normalizeLastChanged = (value: RtDeviceState['lastChanged']): number | null => {
    if (value == null) {
        return null;
    }
    if (typeof value === 'number') {
        return value;
    }
    return null;
};

export const renameMyDevice = async (user: User, deviceId: string, name: string): Promise<void> => {
    await refreshAuthForFirestore(user);
    const trimmed = name.trim().slice(0, 64);
    if (!trimmed) {
        throw new Error('Назва не може бути порожньою.');
    }
    await setDoc(
        doc(firestore, 'users', user.uid, 'devices', deviceId),
        { name: trimmed, lastSeen: serverTimestamp() },
        { merge: true },
    );

    const databaseUrl = import.meta.env.VITE_FIREBASE_DATABASE_URL;
    if (!databaseUrl) {
        return;
    }
    const deviceStatusRef = ref(realtimeDb, `status/${user.uid}/devices/${deviceId}`);
    try {
        await update(deviceStatusRef, { name: trimmed });
        await onDisconnect(deviceStatusRef).set({
            state: 'offline',
            lastChanged: rtdbServerTimestamp(),
            name: trimmed,
        });
    } catch (err) {
        console.warn('Failed to sync device name to Realtime DB', err);
    }
};

export const useMyDevices = (user: User | null): DeviceRow[] => {
    const currentDeviceId = useMemo(() => getOrCreateDeviceId(), []);
    const [meta, setMeta] = useState<Record<string, FirestoreDeviceMeta>>({});
    const [live, setLive] = useState<Record<string, RtDeviceState>>({});

    useEffect(() => {
        if (!user?.uid) {
            setMeta({});
            return;
        }
        const col = collection(firestore, 'users', user.uid, 'devices');
        const unsub = onSnapshot(
            col,
            (snap) => {
                const next: Record<string, FirestoreDeviceMeta> = {};
                snap.forEach((d) => {
                    next[d.id] = d.data() as FirestoreDeviceMeta;
                });
                setMeta(next);
            },
            (err) => console.warn('devices snapshot error', err),
        );
        return () => unsub();
    }, [user?.uid]);

    useEffect(() => {
        if (!user?.uid) {
            setLive({});
            return;
        }
        const databaseUrl = import.meta.env.VITE_FIREBASE_DATABASE_URL;
        if (!databaseUrl) {
            setLive({});
            return;
        }
        const rtRef = ref(realtimeDb, `status/${user.uid}/devices`);
        const unsub = onValue(rtRef, (snap) => {
            setLive((snap.val() as Record<string, RtDeviceState>) || {});
        });
        return () => unsub();
    }, [user?.uid]);

    const rows = useMemo(() => {
        const ids = new Set<string>([...Object.keys(meta), ...Object.keys(live)]);
        const next: DeviceRow[] = Array.from(ids).map((id) => {
            const rt = live[id];
            const rawState = rt?.state;
            const state: 'online' | 'offline' = rawState === 'online' ? 'online' : 'offline';
            const fsNameRaw = meta[id]?.name;
            const fsName = typeof fsNameRaw === 'string' && fsNameRaw.trim() ? fsNameRaw.trim() : '';
            const rtNameRaw = rt?.name;
            const rtName = typeof rtNameRaw === 'string' && rtNameRaw.trim() ? rtNameRaw.trim() : '';
            return {
                deviceId: id,
                name: fsName || rtName || defaultDeviceLabel(id),
                state,
                lastChanged: normalizeLastChanged(rt?.lastChanged),
                isCurrent: id === currentDeviceId,
            };
        });
        next.sort((a, b) => {
            if (a.isCurrent !== b.isCurrent) {
                return a.isCurrent ? -1 : 1;
            }
            if (a.state !== b.state) {
                return a.state === 'online' ? -1 : 1;
            }
            return a.name.localeCompare(b.name, undefined, { sensitivity: 'base' });
        });
        return next;
    }, [meta, live, currentDeviceId]);

    return rows;
};
