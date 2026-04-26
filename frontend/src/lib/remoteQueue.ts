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

/**
 * Master dispatches tasks to a remote worker device.
 * Writes job doc + all task docs in batches.
 */
const FIRESTORE_DOC_LIMIT_BYTES = 900_000;

/** Recursively replaces undefined values with null so Firestore doesn't reject them. */
function sanitizeForFirestore(value: unknown): unknown {
    if (value === undefined) return null;
    if (value === null || typeof value !== 'object') return value;
    if (Array.isArray(value)) return value.map(sanitizeForFirestore);
    return Object.fromEntries(
        Object.entries(value as Record<string, unknown>).map(([k, v]) => [k, sanitizeForFirestore(v)]),
    );
} // 900KB — safe margin under 1MB limit

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

    await setDoc(jobRef, {
        masterDeviceId,
        workerDeviceId,
        status: 'pending' as RemoteJobStatus,
        createdAt: Date.now(),
        totalTasks: tasks.length,
    });
    console.log('[RemoteQueue] job doc written:', jobId);

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
        console.log('[RemoteQueue] batch committed, tasks written:', Math.min(i + MAX_BATCH_OPS, tasks.length));
    }

    console.log('[RemoteQueue] all tasks dispatched, jobId:', jobId);
    return jobId;
}

/**
 * Worker: listens for incoming jobs assigned to this device.
 * Calls onJob when a new 'pending' job arrives.
 *
 * Query uses only `workerDeviceId` (single equality) so no Firestore composite index
 * is required; `status` is filtered client-side.
 */
export function listenToIncomingJobs(
    user: User,
    workerDeviceId: string,
    onJob: (job: RemoteJob) => void,
): Unsubscribe {
    const uid = user.uid;
    const col = collection(firestore, 'users', uid, 'remoteJobs');
    // NOTE: do NOT add `where('status'...)` here without a composite index in
    // `firestore.indexes.json` + `firebase deploy --only firestore`.
    const q = query(col, where('workerDeviceId', '==', workerDeviceId));
    return onSnapshot(
        q,
        (snap) => {
            snap.docChanges().forEach((change) => {
                if (change.type !== 'added') return;
                const d = change.doc;
                const data = d.data() as {
                    status?: string;
                    masterDeviceId?: string;
                    workerDeviceId?: string;
                    createdAt?: number;
                    totalTasks?: number;
                };
                if (data.status !== 'pending') return;
                onJob({
                    jobId: d.id,
                    masterDeviceId: data.masterDeviceId ?? '',
                    workerDeviceId: data.workerDeviceId ?? '',
                    status: data.status as RemoteJobStatus,
                    createdAt: data.createdAt ?? 0,
                    totalTasks: data.totalTasks ?? 0,
                });
            });
        },
        (err) => {
            console.error('[RemoteQueue] listenToIncomingJobs onSnapshot error', err);
        },
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

/**
 * Worker: marks job as accepted so master knows work has begun.
 */
export async function acceptJob(user: User, jobId: string): Promise<void> {
    await refreshAuthForFirestore(user);
    const ref = doc(firestore, 'users', user.uid, 'remoteJobs', jobId);
    await updateDoc(ref, { status: 'accepted' as RemoteJobStatus });
}

/**
 * Worker: marks job as running.
 */
export async function markJobRunning(user: User, jobId: string): Promise<void> {
    await refreshAuthForFirestore(user);
    const ref = doc(firestore, 'users', user.uid, 'remoteJobs', jobId);
    await updateDoc(ref, { status: 'running' as RemoteJobStatus });
}

/**
 * Worker: marks job as completed or failed.
 */
export async function markJobFinished(
    user: User,
    jobId: string,
    status: 'completed' | 'failed',
): Promise<void> {
    await refreshAuthForFirestore(user);
    const ref = doc(firestore, 'users', user.uid, 'remoteJobs', jobId);
    await updateDoc(ref, { status });
}

/**
 * Worker: writes / merges task status update so master can see it in real-time.
 */
export async function writeTaskStatus(
    user: User,
    jobId: string,
    taskId: string,
    patch: Partial<RemoteTaskStatus>,
): Promise<void> {
    const ref = doc(firestore, 'users', user.uid, 'remoteJobs', jobId, 'statuses', taskId);
    await setDoc(ref, { ...patch, updatedAt: Date.now() }, { merge: true });
}

/**
 * Master: subscribes to real-time status updates for all tasks in a job.
 * onUpdate is called with the taskId and partial status on every change.
 */
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
        (err) => console.error('[RemoteQueue] listenToJobStatuses onSnapshot error', err),
    );
}

/**
 * Master: subscribes to job-level status changes (pending → accepted → running → completed).
 */
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
        (err) => console.error('[RemoteQueue] listenToJobStatus onSnapshot error', err),
    );
}
