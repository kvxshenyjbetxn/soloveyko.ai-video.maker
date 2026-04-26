import { useEffect, useRef, useState, useMemo } from 'react';
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

    // jobId of the remote job currently being processed
    const [activeJob, setActiveJob] = useState<string | null>(null);

    // master task IDs that belong to the current remote job
    const jobTaskIdsRef = useRef<Set<string>>(new Set());
    const totalTasksRef = useRef(0);
    const completedCountRef = useRef(0);
    // guard against accepting two jobs simultaneously
    const acceptingRef = useRef(false);

    // keep latest user in a ref so event callbacks always have fresh value
    const userRef = useRef(user);
    useEffect(() => { userRef.current = user; }, [user]);

    // Listen for incoming jobs when idle
    useEffect(() => {
        if (!user) return;

        const unsub = listenToIncomingJobs(user, currentDeviceId, async (job) => {
            if (activeJob || acceptingRef.current) return;
            acceptingRef.current = true;

            try {
                await acceptJob(user, job.jobId);
                const tasks = await fetchJobTasks(user, job.jobId);

                totalTasksRef.current = tasks.length;
                completedCountRef.current = 0;
                jobTaskIdsRef.current = new Set(tasks.map((t) => t.id));

                for (const task of tasks) {
                    addTask(
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

                setActiveJob(job.jobId);
                await markJobRunning(user, job.jobId);

                // Small delay to let React flush state before queue starts
                setTimeout(() => void startQueue(), 400);
            } catch (err) {
                console.error('[RemoteWorker] Failed to accept job', err);
                acceptingRef.current = false;
            }
        });

        return () => unsub();
    }, [user, currentDeviceId, activeJob]);

    // Once a job is active, forward Wails events to Firestore
    useEffect(() => {
        if (!activeJob) return;
        const jobId = activeJob;

        const unsubTextResult = EventsOn(
            'textResult',
            async (id: string, length: number) => {
                if (!jobTaskIdsRef.current.has(id)) return;
                const u = userRef.current;
                if (!u) return;
                try { await writeTaskStatus(u, jobId, id, { resultLength: length }); } catch { /* non-fatal */ }
            },
        );

        const unsubStage = EventsOn(
            'stageStatus',
            async (id: string, stage: string, status: string, msg?: string) => {
                if (!jobTaskIdsRef.current.has(id)) return;
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
                if (!jobTaskIdsRef.current.has(id)) return;
                const u = userRef.current;
                if (!u) return;

                try { await writeTaskStatus(u, jobId, id, { overallStatus: status }); } catch { /* non-fatal */ }

                if (status === 'completed' || status === 'failed') {
                    completedCountRef.current += 1;
                    if (completedCountRef.current >= totalTasksRef.current) {
                        // Determine final job status: failed if any task failed
                        const allSnap = Array.from(jobTaskIdsRef.current);
                        const _ = allSnap; // used implicitly via completedCount
                        const finalStatus: 'completed' | 'failed' =
                            status === 'failed' ? 'failed' : 'completed';
                        try { await markJobFinished(u, jobId, finalStatus); } catch { /* non-fatal */ }

                        setActiveJob(null);
                        jobTaskIdsRef.current.clear();
                        completedCountRef.current = 0;
                        totalTasksRef.current = 0;
                        acceptingRef.current = false;
                    }
                }
            },
        );

        return () => {
            unsubTextResult();
            unsubStage();
            unsubStatus();
        };
    }, [activeJob]);
}
