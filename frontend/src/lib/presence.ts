import { onValue, onDisconnect, ref, serverTimestamp, set } from 'firebase/database';
import type { User } from 'firebase/auth';
import { realtimeDb } from './firebase';

type PresenceState = {
  state: 'online' | 'offline';
  lastChanged: object;
};

export const trackUserPresence = (user: User) => {
  const databaseUrl = import.meta.env.VITE_FIREBASE_DATABASE_URL;
  if (!databaseUrl) {
    console.warn('VITE_FIREBASE_DATABASE_URL is not set. Presence tracking is disabled.');
    return () => undefined;
  }

  const connectedRef = ref(realtimeDb, '.info/connected');
  const userStatusRef = ref(realtimeDb, `status/${user.uid}`);

  const offlineState: PresenceState = {
    state: 'offline',
    lastChanged: serverTimestamp()
  };

  const onlineState: PresenceState = {
    state: 'online',
    lastChanged: serverTimestamp()
  };

  const unsubscribe = onValue(connectedRef, async (snapshot) => {
    if (!snapshot.val()) {
      return;
    }

    await onDisconnect(userStatusRef).set(offlineState);
    await set(userStatusRef, onlineState);
  });

  return () => {
    unsubscribe();
    void set(userStatusRef, offlineState);
  };
};
