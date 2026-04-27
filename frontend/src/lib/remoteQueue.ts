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
import { firestore, refreshAuthForFirestore, storage } from './firebase';
import { ref as storageRef, uploadBytes, getDownloadURL } from 'firebase/storage';

export type ImageControlStatus = 'pending' | 'resolved';

export async function uploadImageControlRequest(
    user: User,
    jobId: string,
    taskId: string,
    localPaths: string[],
): Promise<void> {
    await refreshAuthForFirestore(user);

    const uploadedUrls: string[] = [];
    
    // Завантажуємо всі файли паралельно
    await Promise.all(localPaths.map(async (localPath, index) => {
        try {
            // Використовуємо Wails local endpoint для завантаження файлу
            const cleanPath = localPath.replace(/\\/g, '/');
            const response = await fetch(`local/${encodeURIComponent(cleanPath)}`);
            if (!response.ok) throw new Error(`Failed to read file ${localPath}`);
            const blob = await response.blob();

            // Завантажуємо в Firebase Storage
            const ext = cleanPath.split('.').pop();
            const fileName = `previews/${jobId}/${taskId}/${index}.${ext}`;
            const sRef = storageRef(storage, `users/${user.uid}/${fileName}`);
            await uploadBytes(sRef, blob);
            const url = await getDownloadURL(sRef);
            uploadedUrls.push(url);
        } catch (e) {
            console.error(`[RemoteWorker] Error uploading preview ${localPath}:`, e);
        }
    }));

    // Зберігаємо запит у Firestore
    const ref = doc(firestore, 'users', user.uid, 'remoteJobs', jobId, 'imageControls', taskId);
    await setDoc(ref, {
        status: 'pending' as ImageControlStatus,
        previewUrls: uploadedUrls,
        requestedAt: Date.now(),
    });
}

export function listenToImageControlResponse(
    user: User,
    jobId: string,
    taskId: string,
    onResponded: (action: string) => void,
): () => void {
    const refPath = doc(firestore, 'users', user.uid, 'remoteJobs', jobId, 'imageControls', taskId);
    return onSnapshot(
        refPath,
        (snap) => {
            const data = snap.data();
            if (!data) return;
            if (data.status === 'resolved' && data.action) {
                onResponded(data.action);
            }
        },
        (err) => console.warn('[RemoteWorker] image control listener error:', err),
    );
}

export async function submitImageControlResponse(
    user: User,
    jobId: string,
    taskId: string,
    action: string, // 'confirm', 'regenerate', 'cancel'
): Promise<void> {
    await refreshAuthForFirestore(user);
    const refPath = doc(firestore, 'users', user.uid, 'remoteJobs', jobId, 'imageControls', taskId);
    await updateDoc(refPath, {
        status: 'resolved' as ImageControlStatus,
        action,
        resolvedAt: Date.now(),
    });
}


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

// ─────────────────────────────────────────────────────────────────────────────
// Translation Control (worker ↔ master text review via Firestore)
// ─────────────────────────────────────────────────────────────────────────────

export type TranslationControlStatus = 'pending' | 'resolved';

export type TranslationControlRequest = {
    status: TranslationControlStatus;
    text: string;
    requestedAt: number;
    resolvedAt?: number;
    action?: 'confirm' | 'regenerate' | 'cancel';
    approvedText?: string;
};

/** Worker: writes a translation control request for the master to review. */
export async function writeTranslationControlRequest(
    user: User,
    jobId: string,
    taskId: string,
    text: string,
): Promise<void> {
    await refreshAuthForFirestore(user);
    const ref = doc(firestore, 'users', user.uid, 'remoteJobs', jobId, 'translationControls', taskId);
    await setDoc(ref, {
        status: 'pending' as TranslationControlStatus,
        text,
        requestedAt: Date.now(),
    });
}

/** Master: submits the approved/edited text back to the worker. */
export async function submitTranslationControlResponse(
    user: User,
    jobId: string,
    taskId: string,
    action: 'confirm' | 'regenerate' | 'cancel',
    approvedText: string,
): Promise<void> {
    console.log(`[RemoteQueue] submitTranslationControlResponse: action=${action}, approvedText len=${approvedText?.length}`);
    await refreshAuthForFirestore(user);
    const ref = doc(firestore, 'users', user.uid, 'remoteJobs', jobId, 'translationControls', taskId);
    await updateDoc(ref, {
        status: 'resolved' as TranslationControlStatus,
        action,
        approvedText,
        resolvedAt: Date.now(),
    });
}

/**
 * Worker: listens for the master's response on a single translation control.
 * Calls cb once when status becomes 'resolved', then the caller should unsub.
 */
export function listenToTranslationControlResponse(
    user: User,
    jobId: string,
    taskId: string,
    onResponse: (action: string, approvedText: string) => void,
): Unsubscribe {
    const ref = doc(firestore, 'users', user.uid, 'remoteJobs', jobId, 'translationControls', taskId);
    return onSnapshot(
        ref,
        (snap) => {
            if (!snap.exists()) return;
            const data = snap.data() as TranslationControlRequest;
            if (data.status === 'resolved' && data.action) {
                const finalText = (data.approvedText !== undefined && data.approvedText !== null) ? data.approvedText : data.text;
                console.log(`[RemoteQueue] Translation resolved. action: ${data.action}, approvedText len: ${data.approvedText?.length}, finalText len: ${finalText?.length}`);
                onResponse(data.action, finalText);
            }
        },
        (err) => console.error('[RemoteQueue] listenToTranslationControlResponse error:', err),
    );
}

/**
 * Master: listens for incoming translation control requests from a worker.
 * Fires cb for every 'added' or 'modified' document that has status='pending'.
 */
export interface ImageControlRequest {
    status: ImageControlStatus;
    previewUrls?: string[];
    requestedAt: number;
    action?: string;
}

export function listenToImageControls(
    user: User,
    jobId: string,
    onControl: (taskId: string, request: ImageControlRequest) => void,
): Unsubscribe {
    const col = collection(firestore, 'users', user.uid, 'remoteJobs', jobId, 'imageControls');
    return onSnapshot(
        col,
        (snap) => {
            snap.docChanges().forEach((change) => {
                if (change.type !== 'added' && change.type !== 'modified') return;
                const data = change.doc.data() as ImageControlRequest;
                if (data.status === 'pending') {
                    onControl(change.doc.id, data);
                }
            });
        },
        (err) => console.error('[RemoteQueue] listenToImageControls error:', err),
    );
}

export function listenToTranslationControls(
    user: User,
    jobId: string,
    onControl: (taskId: string, request: TranslationControlRequest) => void,
): Unsubscribe {
    const col = collection(firestore, 'users', user.uid, 'remoteJobs', jobId, 'translationControls');
    return onSnapshot(
        col,
        (snap) => {
            snap.docChanges().forEach((change) => {
                if (change.type !== 'added' && change.type !== 'modified') return;
                const data = change.doc.data() as TranslationControlRequest;
                if (data.status === 'pending') {
                    onControl(change.doc.id, data);
                }
            });
        },
        (err) => console.error('[RemoteQueue] listenToTranslationControls error:', err),
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
