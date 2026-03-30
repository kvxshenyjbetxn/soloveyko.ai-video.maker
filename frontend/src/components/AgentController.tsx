import { useEffect } from 'react';
import { useQueue } from '../contexts/QueueContext';
import { useTemplates } from '../contexts/TemplateContext';
import { useEditorDrafts } from '../contexts/EditorDraftContext';
import { useI18n } from '../contexts/I18nContext';
// @ts-ignore
import { AddToHistory, CheckExistingTasks, GetGalleryImages, GetPipelineSettings } from '../../wailsjs/go/main/App';
// @ts-ignore
import { EventsOn } from '../../wailsjs/runtime/runtime';

type AgentTextTab = 'translate' | 'rewrite';

interface AgentControllerProps {
    currentPath: string;
    setCurrentPath: (path: string) => void;
}

interface AgentRequestEnvelope {
    id: string;
    action: string;
    params?: any;
}

const isTextTab = (value: string): value is AgentTextTab => value === 'translate' || value === 'rewrite';

const getTaskTypeFromPath = (path: string): AgentTextTab => {
    if (path === 'text.rewrite') return 'rewrite';
    return 'translate';
};

const uniqueStrings = (values: string[]) => Array.from(new Set(values.filter(Boolean)));

const VIDEO_EXTENSIONS = new Set(['.mp4', '.mov', '.avi', '.mkv', '.webm', '.m4v']);

const getMediaType = (path: string, url?: string) => {
    const value = (path || url || '').toLowerCase();
    const dotIndex = value.lastIndexOf('.');
    const extension = dotIndex >= 0 ? value.slice(dotIndex) : '';
    return VIDEO_EXTENSIONS.has(extension) ? 'video' : 'image';
};

export const AgentController = ({ currentPath, setCurrentPath }: AgentControllerProps) => {
    const { t } = useI18n();
    const {
        tasks,
        isProcessing,
        isImageBatchReady,
        completionModal,
        addTask,
        addTasks,
        clearQueue,
        startQueue,
        startRemoteQueue,
        resumeTask,
        resumeImageControl,
        updateControlDraft,
    } = useQueue();
    const {
        templates,
        selectedTemplateIds,
        setSelectedTemplateIds,
        flattenSettings,
    } = useTemplates();
    const {
        getTextForTab,
        setTextForTab,
    } = useEditorDrafts();

    useEffect(() => {
        const app = window.go?.main?.App as any;
        if (app?.AgentControllerReady) {
            void app.AgentControllerReady();
        }
    }, []);

    useEffect(() => {
        const respond = async (id: string, payload: any, error?: unknown) => {
            const app = window.go?.main?.App as any;
            if (!app?.ResolveAgentRequest) return;

            const errorText = error instanceof Error ? error.message : (typeof error === 'string' ? error : '');
            const payloadText = payload === undefined ? '' : JSON.stringify(payload);
            await app.ResolveAgentRequest(id, payloadText, errorText);
        };

        const resolveTemplateIds = (taskType: AgentTextTab, templateIds?: string[], templateNames?: string[]) => {
            const typedTemplates = templates.filter(tpl => tpl.type === taskType);
            const idsFromNames = (templateNames || []).map(name => typedTemplates.find(tpl => tpl.name === name)?.id || "");
            const merged = uniqueStrings([...(templateIds || []), ...idsFromNames]);
            const valid = merged.filter(id => typedTemplates.some(tpl => tpl.id === id));
            const missingNames = (templateNames || []).filter(name => !typedTemplates.some(tpl => tpl.name === name));

            if (missingNames.length > 0) {
                throw new Error(`Templates not found: ${missingNames.join(', ')}`);
            }

            return valid;
        };

        const buildExistingResolution = (mode: string | undefined, existingData: any[]) => {
            const found = new Set<string>();
            existingData.forEach(item => (item?.foundStages || []).forEach((stage: string) => found.add(stage)));

            if (mode === 'error') {
                return {
                    shouldContinue: false,
                    skippedStages: [] as string[],
                    settingsOverrides: {},
                    foundStages: Array.from(found),
                };
            }

            if (mode === 'skip_found') {
                return {
                    shouldContinue: true,
                    skippedStages: Array.from(found),
                    settingsOverrides: {},
                    foundStages: Array.from(found),
                };
            }

            const settingsOverrides: Record<string, any> = {};
            if (found.has('voice')) settingsOverrides.voiceoverRegenerate = true;
            if (found.has('image')) {
                settingsOverrides.imageRegeneratePrompts = true;
                settingsOverrides.imageGooglerRegenerateImages = true;
                settingsOverrides.imageElevenLabsImageRegenerate = true;
            }
            if (found.has('subtitle')) settingsOverrides.subtitleRegenerate = true;

            return {
                shouldContinue: true,
                skippedStages: [] as string[],
                settingsOverrides,
                foundStages: Array.from(found),
            };
        };

        const enqueueTask = async (params: any) => {
            const taskType: AgentTextTab = isTextTab(params?.taskType) ? params.taskType : getTaskTypeFromPath(currentPath);
            const taskName = (params?.taskName || '').trim() || `${t('queue.task_default_name')} ${tasks.length + 1}`;
            const content = typeof params?.text === 'string' ? params.text : getTextForTab(taskType);
            const existingMode = typeof params?.onExisting === 'string' ? params.onExisting : 'regenerate';
            const templateIds = params?.templateIds as string[] | undefined;
            const templateNames = params?.templateNames as string[] | undefined;
            const resolvedTemplateIds = (templateIds || templateNames)
                ? resolveTemplateIds(taskType, templateIds, templateNames)
                : selectedTemplateIds.filter(id => templates.some(tpl => tpl.id === id && tpl.type === taskType));

            const settings = await GetPipelineSettings();
            const relevantTemplates = resolvedTemplateIds
                .map(id => templates.find(tpl => tpl.id === id && tpl.type === taskType))
                .filter(Boolean) as typeof templates;

            const tasksToCheck = relevantTemplates.length === 0
                ? [{ taskName, taskType, subName: "", settings }]
                : relevantTemplates.map(tpl => ({
                    taskName,
                    taskType,
                    subName: tpl.name,
                    settings: flattenSettings(tpl.settings),
                }));

            const existingData = await CheckExistingTasks(tasksToCheck);
            const existingResolution = buildExistingResolution(existingMode, existingData || []);

            if (!existingResolution.shouldContinue) {
                return {
                    queued: false,
                    requiresExistingFilesDecision: true,
                    existingFiles: existingData || [],
                };
            }

            const templatesUsed = relevantTemplates.length === 0
                ? [t('common.default') || 'Default']
                : relevantTemplates.map(tpl => tpl.name);

            try {
                await AddToHistory(taskName, taskType, templatesUsed, content);
            } catch (historyError) {
                console.error('Agent AddToHistory failed:', historyError);
            }

            if (relevantTemplates.length === 0) {
                const existing = (existingData || []).find((item: any) => item.id === "");
                addTask(
                    taskType,
                    content,
                    { ...settings, ...existingResolution.settingsOverrides },
                    taskName,
                    "",
                    existingResolution.skippedStages,
                    existing,
                );
            } else {
                const tasksData = relevantTemplates.map(tpl => ({
                    settings: { ...flattenSettings(tpl.settings), ...existingResolution.settingsOverrides },
                    subName: tpl.name,
                    existing: (existingData || []).find((item: any) => item.id === tpl.name),
                }));
                addTasks(taskType, content, tasksData, taskName, existingResolution.skippedStages);
            }

            if (params?.selectTemplates !== false) {
                setSelectedTemplateIds(resolvedTemplateIds);
            }
            if (params?.focusQueue) {
                setCurrentPath('queue');
            }

            return {
                queued: true,
                taskType,
                taskName,
                queuedItems: relevantTemplates.length > 0 ? relevantTemplates.map(tpl => tpl.name) : [taskName],
                usedTemplateIds: resolvedTemplateIds,
                existingResolution: existingMode,
            };
        };

        const getQueueState = () => {
            const summarizedTasks = tasks.map(task => ({
                id: task.id,
                name: task.name,
                folderName: task.folderName,
                subName: task.subName,
                type: task.type,
                status: task.status,
                progress: task.progress,
                isAwaitingControl: !!task.isAwaitingControl,
                isAwaitingImageControl: !!task.isAwaitingImageControl,
                isAwaitingMontageControl: !!task.isAwaitingMontageControl,
                textStatus: task.textStatus,
                voiceStatus: task.voiceStatus,
                imageStatus: task.imageStatus,
                subtitleStatus: task.subtitleStatus,
                montageStatus: task.montageStatus,
            }));

            const pendingTextControls = tasks
                .filter(task => task.isAwaitingControl)
                .map(task => ({
                    id: task.id,
                    name: task.name,
                    folderName: task.folderName,
                    subName: task.subName,
                    type: task.type,
                    text: task.controlContent || '',
                    originalLength: task.originalLength || 0,
                    currentLength: (task.controlContent || '').length,
                }));

            return {
                currentPath,
                isProcessing,
                isImageBatchReady,
                taskCount: tasks.length,
                tasks: summarizedTasks,
                pendingTextControls,
                completionModal,
                allTasksCompleted: tasks.length > 0 && !isProcessing && tasks.every(task => task.status === 'completed' || task.status === 'failed'),
            };
        };

        const getGalleryPreview = async (params: any) => {
            const galleryTasks = await GetGalleryImages();
            const limitPerTask = Math.max(1, Number(params?.limitPerTask) || 3);
            const limitPerTemplate = Math.max(1, Number(params?.limitPerTemplate) || limitPerTask);
            const includePrompts = params?.includePrompts !== false;
            const onlyAwaitingImageControl = params?.onlyAwaitingImageControl !== false;
            const requestedTaskNames = uniqueStrings(
                Array.isArray(params?.taskNames)
                    ? params.taskNames.filter((value: unknown): value is string => typeof value === 'string')
                    : []
            );
            const awaitingTaskNames = uniqueStrings(
                tasks
                    .filter(task => task.isAwaitingImageControl)
                    .map(task => task.folderName)
            );

            let allowedTaskNames = requestedTaskNames;
            if (onlyAwaitingImageControl) {
                const awaitingSet = new Set(awaitingTaskNames);
                allowedTaskNames = requestedTaskNames.length > 0
                    ? requestedTaskNames.filter(name => awaitingSet.has(name))
                    : awaitingTaskNames;
            }

            const allowedSet = allowedTaskNames.length > 0 ? new Set(allowedTaskNames) : null;

            const previewTasks = (galleryTasks || [])
                .filter(task => !allowedSet || allowedSet.has(task.name))
                .map(task => {
                    let taskItemCount = 0;
                    const templatesPreview = (task.templates || [])
                        .map(template => {
                            const remainingForTask = Math.max(0, limitPerTask - taskItemCount);
                            if (remainingForTask === 0) {
                                return null;
                            }

                            const items = (template.images || [])
                                .slice(0, Math.min(limitPerTemplate, remainingForTask))
                                .map(image => {
                                    const mediaType = getMediaType(image.path, image.url);
                                    return {
                                        name: image.name,
                                        path: image.path,
                                        url: image.url,
                                        mediaType,
                                        canRenderInChat: mediaType === 'image',
                                        prompt: includePrompts ? (image.prompt || '') : undefined,
                                    };
                                });

                            taskItemCount += items.length;
                            if (items.length === 0) {
                                return null;
                            }

                            return {
                                name: template.name,
                                itemCount: items.length,
                                items,
                            };
                        })
                        .filter(Boolean);

                    const itemCount = templatesPreview.reduce((sum, template) => sum + (template?.itemCount || 0), 0);
                    return {
                        name: task.name,
                        templateCount: templatesPreview.length,
                        itemCount,
                        templates: templatesPreview,
                    };
                })
                .filter(task => task.itemCount > 0);

            return {
                ok: true,
                onlyAwaitingImageControl,
                requestedTaskNames,
                awaitingTaskNames,
                returnedTaskCount: previewTasks.length,
                limitPerTask,
                limitPerTemplate,
                tasks: previewTasks,
            };
        };

        const handleRequest = async (request: AgentRequestEnvelope) => {
            if (!request?.id || !request?.action) return;

            try {
                switch (request.action) {
                    case 'set_main_text': {
                        const tab: AgentTextTab = isTextTab(request.params?.tab) ? request.params.tab : getTaskTypeFromPath(currentPath);
                        const text = typeof request.params?.text === 'string' ? request.params.text : '';
                        setTextForTab(tab, text);
                        if (request.params?.focusTab) {
                            setCurrentPath(`text.${tab}`);
                        }
                        await respond(request.id, { ok: true, tab, textLength: text.length });
                        return;
                    }
                    case 'get_main_text': {
                        const tab: AgentTextTab = isTextTab(request.params?.tab) ? request.params.tab : getTaskTypeFromPath(currentPath);
                        const text = getTextForTab(tab);
                        await respond(request.id, { ok: true, tab, text, textLength: text.length });
                        return;
                    }
                    case 'select_templates': {
                        const taskType: AgentTextTab = isTextTab(request.params?.taskType) ? request.params.taskType : getTaskTypeFromPath(currentPath);
                        const resolved = resolveTemplateIds(taskType, request.params?.templateIds, request.params?.templateNames);
                        setSelectedTemplateIds(resolved);
                        await respond(request.id, {
                            ok: true,
                            taskType,
                            selectedTemplateIds: resolved,
                            selectedTemplateNames: templates.filter(tpl => resolved.includes(tpl.id)).map(tpl => tpl.name),
                        });
                        return;
                    }
                    case 'enqueue_task': {
                        const result = await enqueueTask(request.params || {});
                        await respond(request.id, result);
                        return;
                    }
                    case 'start_queue': {
                        if (request.params?.workerId) {
                            const workerName = typeof request.params?.workerName === 'string' ? request.params.workerName : 'Remote Worker';
                            await startRemoteQueue(request.params.workerId, workerName);
                        } else {
                            await startQueue();
                        }
                        setCurrentPath('queue');
                        await respond(request.id, { ok: true, mode: request.params?.workerId ? 'remote' : 'local' });
                        return;
                    }
                    case 'continue_image_control': {
                        await resumeImageControl();
                        setCurrentPath('queue');
                        await respond(request.id, { ok: true });
                        return;
                    }
                    case 'get_pending_text_controls': {
                        await respond(request.id, {
                            ok: true,
                            controls: tasks
                                .filter(task => task.isAwaitingControl)
                                .map(task => ({
                                    id: task.id,
                                    name: task.name,
                                    folderName: task.folderName,
                                    subName: task.subName,
                                    type: task.type,
                                    text: task.controlContent || '',
                                    originalLength: task.originalLength || 0,
                                    currentLength: (task.controlContent || '').length,
                                })),
                        });
                        return;
                    }
                    case 'update_text_control': {
                        const taskId = request.params?.taskId;
                        const text = typeof request.params?.text === 'string' ? request.params.text : '';
                        if (!taskId) throw new Error('taskId is required');
                        updateControlDraft(taskId, text);
                        await respond(request.id, { ok: true, taskId, textLength: text.length });
                        return;
                    }
                    case 'confirm_text_control': {
                        const taskId = request.params?.taskId;
                        if (!taskId) throw new Error('taskId is required');
                        const task = tasks.find(item => item.id === taskId);
                        if (!task || !task.isAwaitingControl) {
                            throw new Error(`Task ${taskId} is not awaiting text control`);
                        }
                        const text = typeof request.params?.text === 'string' ? request.params.text : (task.controlContent || '');
                        await resumeTask(taskId, text);
                        await respond(request.id, { ok: true, taskId, textLength: text.length });
                        return;
                    }
                    case 'get_queue_state': {
                        await respond(request.id, getQueueState());
                        return;
                    }
                    case 'get_gallery_preview': {
                        await respond(request.id, await getGalleryPreview(request.params || {}));
                        return;
                    }
                    case 'clear_queue': {
                        clearQueue();
                        setCurrentPath('text.translate');
                        await respond(request.id, { ok: true });
                        return;
                    }
                    case 'navigate': {
                        const path = typeof request.params?.path === 'string' ? request.params.path : 'text.translate';
                        setCurrentPath(path);
                        await respond(request.id, { ok: true, path });
                        return;
                    }
                    default:
                        throw new Error(`Unknown agent action: ${request.action}`);
                }
            } catch (error) {
                await respond(request.id, undefined, error);
            }
        };

        const unsub = EventsOn('agent:request', (request: AgentRequestEnvelope) => {
            void handleRequest(request);
        });

        return () => {
            if (unsub) unsub();
        };
    }, [
        addTask,
        addTasks,
        clearQueue,
        completionModal,
        currentPath,
        flattenSettings,
        getTextForTab,
        isImageBatchReady,
        isProcessing,
        resumeImageControl,
        resumeTask,
        selectedTemplateIds,
        setCurrentPath,
        setSelectedTemplateIds,
        setTextForTab,
        startQueue,
        startRemoteQueue,
        t,
        tasks,
        templates,
        updateControlDraft,
    ]);

    return null;
};
