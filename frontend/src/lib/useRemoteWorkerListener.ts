import { useEffect, useRef, useMemo } from 'react';
import type { User } from 'firebase/auth';
import { EventsOn } from '../../wailsjs/runtime/runtime';
// @ts-ignore
import { SendControlAction } from '../../wailsjs/go/main/App';
import {
    listenToIncomingJobs,
    fetchJobTasks,
    acceptJob,
    markJobRunning,
    markJobFinished,
    writeTaskStatus,
    deleteRemoteJob,
    writeTranslationControlRequest,
    listenToTranslationControlResponse,
    type RemoteTaskStatus,
} from './remoteQueue';
import { getOrCreateDeviceId } from './deviceId';

type AddTaskFn = (
    type: any,
    content: string,
    settings: any,
    name?: string,
    subName?: string,
    skippedStages?: string[],
    existingData?: any,
    taskId?: string,
    taskNumber?: number,
) => void;

type StartQueueFn = () => Promise<void>;

/**
 * Слухач віддалених джобів для воркера.
 * Підписка Firestore тримається стабільною на весь сеанс (deps лише user + deviceId),
 * щоб зміна активного джоба не пересоздавала listener і не ламала автозапуск черги.
 */
export function useRemoteWorkerListener(
    user: User | null,
    addTask: AddTaskFn,
    startQueue: StartQueueFn,
    onControlResponded?: (id: string, text: string, action: string) => void,
) {
    const currentDeviceId = useMemo(() => getOrCreateDeviceId(), []);

    const addTaskRef = useRef(addTask);
    const startQueueRef = useRef(startQueue);
    const onControlRespondedRef = useRef(onControlResponded);
    
    useEffect(() => { addTaskRef.current = addTask; }, [addTask]);
    useEffect(() => { startQueueRef.current = startQueue; }, [startQueue]);
    useEffect(() => { onControlRespondedRef.current = onControlResponded; }, [onControlResponded]);

    const jobTaskIdsRef = useRef<Set<string>>(new Set());
    const totalTasksRef = useRef(0);
    const completedCountRef = useRef(0);
    /** Terminal status per task — guard against duplicate Go emissions for the same id */
    const terminalTaskIdsRef = useRef<Set<string>>(new Set());
    const activeJobRef = useRef<string | null>(null);
    const acceptingRef = useRef(false);

    const userRef = useRef(user);
    useEffect(() => { userRef.current = user; }, [user]);

    useEffect(() => {
        if (!user) return;
        console.log('[RemoteWorker] listener active, deviceId:', currentDeviceId);

        const unsub = listenToIncomingJobs(user, currentDeviceId, async (job) => {
            console.log(
                '[RemoteWorker] incoming job:',
                job.jobId,
                'activeJob:',
                activeJobRef.current,
                'accepting:',
                acceptingRef.current,
            );
            if (activeJobRef.current || acceptingRef.current) return;
            acceptingRef.current = true;

            try {
                await acceptJob(user, job.jobId);
                const tasks = await fetchJobTasks(user, job.jobId);
                console.log('[RemoteWorker] fetched', tasks.length, 'tasks');

                totalTasksRef.current = tasks.length;
                completedCountRef.current = 0;
                terminalTaskIdsRef.current = new Set();
                jobTaskIdsRef.current = new Set(tasks.map((t) => t.id));
                activeJobRef.current = job.jobId;

                for (const task of tasks) {
                    addTaskRef.current(
                        task.type,
                        task.content,
                        task.settings,
                        task.folderName,
                        task.subName,
                        undefined,
                        undefined,
                        task.id,
                        task.taskNumber,
                    );
                }

                await markJobRunning(user, job.jobId);

                setTimeout(() => {
                    console.log('[RemoteWorker] calling startQueue');
                    void startQueueRef.current();
                }, 400);
            } catch (err) {
                console.error('[RemoteWorker] error:', err);
                acceptingRef.current = false;
                activeJobRef.current = null;
            }
        });

        return () => unsub();
    }, [user, currentDeviceId]);

    useEffect(() => {
        const unsubTextResult = EventsOn(
            'textResult',
            async (id: string, length: number) => {
                const jobId = activeJobRef.current;
                if (!jobId || !jobTaskIdsRef.current.has(id)) return;
                const u = userRef.current;
                if (!u) return;
                try { await writeTaskStatus(u, jobId, id, { resultLength: length }); } catch { /* non-fatal */ }
            },
        );

        const unsubRequestControl = EventsOn(
            'requestControl',
            async (id: string, text: string) => {
                const jobId = activeJobRef.current;
                if (!jobId || !jobTaskIdsRef.current.has(id)) return;
                const u = userRef.current;
                if (!u) return;

                try {
                    await writeTranslationControlRequest(u, jobId, id, text);
                    await writeTaskStatus(u, jobId, id, { textStatus: 'waiting' });
                } catch (err) {
                    console.error('[RemoteWorker] writeTranslationControlRequest failed:', err);
                    return;
                }

                const unsub = listenToTranslationControlResponse(u, jobId, id, (action, approvedText) => {
                    console.log(`[RemoteWorker] listenToTranslationControlResponse fired for ${id}: action=${action}, approvedText length=${approvedText?.length}`);
                    unsub();
                    if (onControlRespondedRef.current) {
                        onControlRespondedRef.current(id, approvedText, action);
                    } else {
                        try { SendControlAction(id, action, approvedText, {}); } catch { /* non-fatal */ }
                    }
                });
            },
        );

        const unsubStage = EventsOn(
            'stageStatus',
            async (id: string, stage: string, status: string, msg?: string) => {
                const jobId = activeJobRef.current;
                if (!jobId || !jobTaskIdsRef.current.has(id)) return;
                const u = userRef.current;
                if (!u) return;

                const patch: Partial<RemoteTaskStatus> = {};
                if (stage === 'text') patch.textStatus = status;
                else if (stage === 'voice') {
                    patch.voiceStatus = status;
                    if (msg) patch.voiceDuration = msg;
                } else if (stage === 'image') {
                    patch.imageStatus = status;
                    if (msg) patch.imagesMessage = msg;
                } else if (stage === 'subtitle') patch.subtitleStatus = status;
                else if (stage === 'montage') {
                    patch.montageStatus = status;
                    if (msg) patch.montageMsg = msg;
                }
                try { await writeTaskStatus(u, jobId, id, patch); } catch { /* non-fatal */ }
            },
        );

        const unsubStatus = EventsOn(
            'taskStatus',
            async (id: string, status: string) => {
                const jobId = activeJobRef.current;
                if (!jobId || !jobTaskIdsRef.current.has(id)) return;
                const u = userRef.current;
                if (!u) return;

                try { await writeTaskStatus(u, jobId, id, { overallStatus: status }); } catch { /* non-fatal */ }

                if (status === 'completed' || status === 'failed') {
                    if (terminalTaskIdsRef.current.has(id)) {
                        return;
                    }
                    terminalTaskIdsRef.current.add(id);
                    completedCountRef.current += 1;
                    if (completedCountRef.current >= totalTasksRef.current) {
                        const finalStatus: 'completed' | 'failed' =
                            status === 'failed' ? 'failed' : 'completed';
                        try { await markJobFinished(u, jobId, finalStatus); } catch { /* non-fatal */ }
                        void deleteRemoteJob(u, jobId);

                        activeJobRef.current = null;
                        jobTaskIdsRef.current.clear();
                        terminalTaskIdsRef.current.clear();
                        completedCountRef.current = 0;
                        totalTasksRef.current = 0;
                        acceptingRef.current = false;
                    }
                }
            },
        );

        return () => {
            unsubTextResult();
            unsubRequestControl();
            unsubStage();
            unsubStatus();
        };
    }, []);
}
