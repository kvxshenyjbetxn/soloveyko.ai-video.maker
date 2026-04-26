import { useEffect, useRef, useMemo } from 'react';
import type { User } from 'firebase/auth';
import { EventsOn } from '../../wailsjs/runtime/runtime';
import {
    listenToIncomingJobs,
    fetchJobTasks,
    acceptJob,
    markJobRunning,
    markJobFinished,
    writeTaskStatus,
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
) => void;

type StartQueueFn = () => Promise<void>;

/**
 * Mounted on every device after login.
 * Listens for remote jobs assigned to this device, creates local tasks,
 * auto-starts the queue, and writes stage/task status updates back to Firestore
 * so the master can see real-time progress.
 */
export function useRemoteWorkerListener(
    user: User | null,
    addTask: AddTaskFn,
    startQueue: StartQueueFn,
) {
    const currentDeviceId = useMemo(() => getOrCreateDeviceId(), []);

    const addTaskRef = useRef(addTask);
    const startQueueRef = useRef(startQueue);
    useEffect(() => { addTaskRef.current = addTask; }, [addTask]);
    useEffect(() => { startQueueRef.current = startQueue; }, [startQueue]);

    const jobTaskIdsRef = useRef<Set<string>>(new Set());
    const totalTasksRef = useRef(0);
    const completedCountRef = useRef(0);
    const activeJobRef = useRef<string | null>(null);
    const acceptingRef = useRef(false);

    const userRef = useRef(user);
    useEffect(() => { userRef.current = user; }, [user]);

    // Listener stays stable for the lifetime of the user session.
    // activeJobRef/acceptingRef prevent double-processing without tearing down the subscription.
    useEffect(() => {
        if (!user) return;
        console.log('[RemoteWorker] setting up incoming job listener, deviceId:', currentDeviceId);

        const unsub = listenToIncomingJobs(user, currentDeviceId, async (job) => {
            console.log('[RemoteWorker] incoming job:', job.jobId, 'activeJob:', activeJobRef.current, 'accepting:', acceptingRef.current);
            if (activeJobRef.current || acceptingRef.current) {
                console.log('[RemoteWorker] already busy, ignoring job');
                return;
            }
            acceptingRef.current = true;

            try {
                console.log('[RemoteWorker] accepting job:', job.jobId);
                await acceptJob(user, job.jobId);

                const tasks = await fetchJobTasks(user, job.jobId);
                console.log('[RemoteWorker] fetched', tasks.length, 'tasks for job:', job.jobId);

                totalTasksRef.current = tasks.length;
                completedCountRef.current = 0;
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
                    );
                }
                console.log('[RemoteWorker] all tasks added to queue');

                await markJobRunning(user, job.jobId);

                setTimeout(() => {
                    console.log('[RemoteWorker] calling startQueue');
                    void startQueueRef.current();
                }, 600);
            } catch (err) {
                console.error('[RemoteWorker] error processing job:', err);
                acceptingRef.current = false;
                activeJobRef.current = null;
            }
        });

        return () => {
            console.log('[RemoteWorker] tearing down listener');
            unsub();
        };
    }, [user, currentDeviceId]);

    // Forward Wails events to Firestore whenever a job is active.
    // Uses refs so this effect never needs to re-mount.
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
                    completedCountRef.current += 1;
                    if (completedCountRef.current >= totalTasksRef.current) {
                        const finalStatus: 'completed' | 'failed' =
                            status === 'failed' ? 'failed' : 'completed';
                        try { await markJobFinished(u, jobId, finalStatus); } catch { /* non-fatal */ }

                        activeJobRef.current = null;
                        acceptingRef.current = false;
                        jobTaskIdsRef.current.clear();
                        completedCountRef.current = 0;
                        totalTasksRef.current = 0;
                        console.log('[RemoteWorker] job finished:', finalStatus);
                    }
                }
            },
        );

        return () => {
            unsubTextResult();
            unsubStage();
            unsubStatus();
        };
    }, []);
}
