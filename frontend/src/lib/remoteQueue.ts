import {
    collection,
    doc,
    setDoc,
    updateDoc,
    onSnapshot,
    getDocs,
    writeBatch,
    query,
    where,
    Unsubscribe,
    deleteDoc,
} from 'firebase/firestore';
import type { User } from 'firebase/auth';
import { firestore, refreshAuthForFirestore } from './firebase';

export type RemoteJobStatus = 'pending' | 'accepted' | 'running' | 'completed' | 'failed';

export type RemoteTaskPayload = {
    id: string;
    type: 'translate' | 'rewrite' | 'voiceover';
    content: string;
    folderName: string;
    subName: string;
    settings: unknown;
    taskNumber: number;
    order: number;
};

export type RemoteTaskStatus = {
    textStatus: string;
    voiceStatus: string;
    imageStatus: string;
    subtitleStatus: string;
    montageStatus: string;
    voiceDuration?: string;
    imagesMessage?: string;
    montageMsg?: string;
    overallStatus: string;
    resultLength?: number;
    updatedAt: number;
};

export type RemoteJob = {
    jobId: string;
    masterDeviceId: string;
    workerDeviceId: string;
    status: RemoteJobStatus;
    createdAt: number;
    totalTasks: number;
};

const MAX_BATCH_OPS = 450;
const FIRESTORE_DOC_LIMIT_BYTES = 900_000;

/** Recursively replaces undefined values with null so Firestore doesn't reject them. */
function sanitizeForFirestore(value: unknown): unknown {
    if (value === undefined) return null;
    if (value === null || typeof value !== 'object') return value;
    if (Array.isArray(value)) return value.map(sanitizeForFirestore);
    return Object.fromEntries(
        Object.entries(value as Record<string, unknown>).map(([k, v]) => [k, sanitizeForFirestore(v)]),
    );
}

export async function dispatchJobToWorker(
    user: User,
    masterDeviceId: string,
    workerDeviceId: string,
    tasks: RemoteTaskPayload[],
): Promise<string> {
    console.log('[RemoteQueue] dispatchJobToWorker start, tasks:', tasks.length);
    await refreshAuthForFirestore(user);
    console.log('[RemoteQueue] auth refreshed');

    const uid = user.uid;
    const jobId = `job_${Date.now()}_${Math.random().toString(36).slice(2, 7)}`;
    const jobRef = doc(firestore, 'users', uid, 'remoteJobs', jobId);

    // Write tasks FIRST so they exist before the worker sees the 'pending' job doc.
    for (let i = 0; i < tasks.length; i += MAX_BATCH_OPS) {
        const batch = writeBatch(firestore);
        const slice = tasks.slice(i, i + MAX_BATCH_OPS);
        for (const task of slice) {
            const contentBytes = new TextEncoder().encode(task.content as string).length;
            if (contentBytes > FIRESTORE_DOC_LIMIT_BYTES) {
                throw new Error(
                    `Задача "${task.folderName}" містить занадто великий текст (${Math.round(contentBytes / 1024)} KB). Ліміт Firestore — 900 KB на документ.`,
                );
            }
            const taskRef = doc(firestore, 'users', uid, 'remoteJobs', jobId, 'tasks', task.id);
            batch.set(taskRef, sanitizeForFirestore(task) as RemoteTaskPayload);
        }
        await batch.commit();
        console.log('[RemoteQueue] batch committed:', Math.min(i + MAX_BATCH_OPS, tasks.length), 'tasks');
    }

    // Only after all tasks are written — create the job doc with status 'pending'.
    // Worker listens for this document and fetches tasks only after it appears.
    await setDoc(jobRef, {
        masterDeviceId,
        workerDeviceId,
        status: 'pending' as RemoteJobStatus,
        createdAt: Date.now(),
        totalTasks: tasks.length,
    });
    console.log('[RemoteQueue] dispatched jobId:', jobId);
    return jobId;
}

function buildJobFromDoc(d: any): RemoteJob | null {
    const data = d.data() as {
        status?: string;
        masterDeviceId?: string;
        workerDeviceId?: string;
        createdAt?: number;
        totalTasks?: number;
    };
    if (data.status !== 'pending') return null;
    console.log('[RemoteQueue] pending job found:', d.id);
    return {
        jobId: d.id,
        masterDeviceId: data.masterDeviceId ?? '',
        workerDeviceId: data.workerDeviceId ?? '',
        status: data.status as RemoteJobStatus,
        createdAt: data.createdAt ?? 0,
        totalTasks: data.totalTasks ?? 0,
    };
}

/**
 * Worker: listens for incoming jobs assigned to this device.
 *
 * Also does a one-time getDocs() immediately after subscribing so jobs that
 * already exist in Firestore are not missed (e.g. if the app started after dispatch).
 */
export function listenToIncomingJobs(
    user: User,
    workerDeviceId: string,
    onJob: (job: RemoteJob) => void,
): Unsubscribe {
    const uid = user.uid;
    const col = collection(firestore, 'users', uid, 'remoteJobs');
    const q = query(col, where('workerDeviceId', '==', workerDeviceId));

    // One-time catch for already-existing pending jobs
    getDocs(q)
        .then((snap) => {
            console.log('[RemoteQueue] getDocs: found', snap.size, 'docs for deviceId:', workerDeviceId);
            snap.forEach((d) => {
                const job = buildJobFromDoc(d);
                if (job) onJob(job);
            });
        })
        .catch((err) => console.error('[RemoteQueue] getDocs error:', err));

    // Real-time listener for future jobs
    return onSnapshot(
        q,
        (snap) => {
            snap.docChanges().forEach((change) => {
                // 'added' — new job; 'modified' — e.g. status reset to pending
                if (change.type !== 'added' && change.type !== 'modified') return;
                const job = buildJobFromDoc(change.doc);
                if (job) onJob(job);
            });
        },
        (err) => console.error('[RemoteQueue] onSnapshot error:', err),
    );
}

/**
 * Worker: reads all task payloads from the job's tasks subcollection.
 */
export async function fetchJobTasks(
    user: User,
    jobId: string,
): Promise<RemoteTaskPayload[]> {
    const uid = user.uid;
    const col = collection(firestore, 'users', uid, 'remoteJobs', jobId, 'tasks');
    const snap = await getDocs(col);
    const tasks: RemoteTaskPayload[] = [];
    snap.forEach((d) => tasks.push(d.data() as RemoteTaskPayload));
    tasks.sort((a, b) => a.order - b.order);
    return tasks;
}

export async function acceptJob(user: User, jobId: string): Promise<void> {
    await refreshAuthForFirestore(user);
    const ref = doc(firestore, 'users', user.uid, 'remoteJobs', jobId);
    await updateDoc(ref, { status: 'accepted' as RemoteJobStatus });
}

export async function markJobRunning(user: User, jobId: string): Promise<void> {
    await refreshAuthForFirestore(user);
    const ref = doc(firestore, 'users', user.uid, 'remoteJobs', jobId);
    await updateDoc(ref, { status: 'running' as RemoteJobStatus });
}

export async function markJobFinished(
    user: User,
    jobId: string,
    status: 'completed' | 'failed',
): Promise<void> {
    await refreshAuthForFirestore(user);
    const ref = doc(firestore, 'users', user.uid, 'remoteJobs', jobId);
    await updateDoc(ref, { status });
}

const DELETE_BATCH = 500;

/** Deletes a job document and all subcollection docs (tasks, statuses). Firestore does not cascade. */
async function deleteSubcollection(uid: string, jobId: string, sub: 'tasks' | 'statuses'): Promise<void> {
    const col = collection(firestore, 'users', uid, 'remoteJobs', jobId, sub);
    const snap = await getDocs(col);
    const ids: string[] = [];
    snap.forEach((d) => ids.push(d.id));
    for (let i = 0; i < ids.length; i += DELETE_BATCH) {
        const batch = writeBatch(firestore);
        for (const id of ids.slice(i, i + DELETE_BATCH)) {
            batch.delete(doc(firestore, 'users', uid, 'remoteJobs', jobId, sub, id));
        }
        await batch.commit();
    }
}

export async function deleteRemoteJob(user: User, jobId: string): Promise<void> {
    try {
        await refreshAuthForFirestore(user);
    } catch {
        return;
    }
    const uid = user.uid;
    try {
        await deleteSubcollection(uid, jobId, 'statuses');
        await deleteSubcollection(uid, jobId, 'tasks');
        await deleteDoc(doc(firestore, 'users', uid, 'remoteJobs', jobId));
        console.log('[RemoteQueue] deleted job from Firestore:', jobId);
    } catch (err) {
        console.warn('[RemoteQueue] deleteRemoteJob failed:', err);
    }
}

export async function writeTaskStatus(
    user: User,
    jobId: string,
    taskId: string,
    patch: Partial<RemoteTaskStatus>,
): Promise<void> {
    const ref = doc(firestore, 'users', user.uid, 'remoteJobs', jobId, 'statuses', taskId);
    await setDoc(ref, { ...patch, updatedAt: Date.now() }, { merge: true });
}

export function listenToJobStatuses(
    user: User,
    jobId: string,
    onUpdate: (taskId: string, status: RemoteTaskStatus) => void,
): Unsubscribe {
    const uid = user.uid;
    const col = collection(firestore, 'users', uid, 'remoteJobs', jobId, 'statuses');
    return onSnapshot(
        col,
        (snap) => {
            snap.docChanges().forEach((change) => {
                if (change.type === 'added' || change.type === 'modified') {
                    onUpdate(change.doc.id, change.doc.data() as RemoteTaskStatus);
                }
            });
        },
        (err) => console.error('[RemoteQueue] listenToJobStatuses error:', err),
    );
}

export function listenToJobStatus(
    user: User,
    jobId: string,
    onUpdate: (status: RemoteJobStatus) => void,
): Unsubscribe {
    const ref = doc(firestore, 'users', user.uid, 'remoteJobs', jobId);
    return onSnapshot(
        ref,
        (snap) => {
            if (snap.exists()) {
                onUpdate(snap.data().status as RemoteJobStatus);
            }
        },
        (err) => console.error('[RemoteQueue] listenToJobStatus error:', err),
    );
}
