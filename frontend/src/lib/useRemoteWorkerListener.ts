import { useEffect, useRef, useState, useMemo } from 'react';
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
) => void;

type StartQueueFn = () => Promise<void>;

export function useRemoteWorkerListener(
    user: User | null,
    addTask: AddTaskFn,
    startQueue: StartQueueFn,
) {
    const currentDeviceId = useMemo(() => getOrCreateDeviceId(), []);

    const [activeJob, setActiveJob] = useState<string | null>(null);

    const jobTaskIdsRef = useRef<Set<string>>(new Set());
    const totalTasksRef = useRef(0);
    const completedCountRef = useRef(0);
    /** Terminal status per task — guard against duplicate Go emissions for the same id */
    const terminalTaskIdsRef = useRef<Set<string>>(new Set());
    const acceptingRef = useRef(false);

    const userRef = useRef(user);
    useEffect(() => { userRef.current = user; }, [user]);

    useEffect(() => {
        if (!user) return;
        console.log('[RemoteWorker] listener active, deviceId:', currentDeviceId);

        const unsub = listenToIncomingJobs(user, currentDeviceId, async (job) => {
            console.log('[RemoteWorker] incoming job:', job.jobId, 'busy:', !!activeJob || acceptingRef.current);
            if (activeJob || acceptingRef.current) return;
            acceptingRef.current = true;

            try {
                await acceptJob(user, job.jobId);
                const tasks = await fetchJobTasks(user, job.jobId);
                console.log('[RemoteWorker] fetched', tasks.length, 'tasks');

                totalTasksRef.current = tasks.length;
                completedCountRef.current = 0;
                terminalTaskIdsRef.current = new Set();
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

                setTimeout(() => {
                    console.log('[RemoteWorker] calling startQueue');
                    void startQueue();
                }, 400);
            } catch (err) {
                console.error('[RemoteWorker] error:', err);
                acceptingRef.current = false;
            }
        });

        return () => unsub();
    }, [user, currentDeviceId, activeJob]);

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

        const unsubRequestControl = EventsOn(
            'requestControl',
            async (id: string, text: string) => {
                if (!jobTaskIdsRef.current.has(id)) return;
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
                    unsub();
                    try { SendControlAction(id, action, approvedText, {}); } catch { /* non-fatal */ }
                });
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

                        setActiveJob(null);
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
    }, [activeJob]);
}
