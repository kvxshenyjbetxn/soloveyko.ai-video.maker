import { getApp, FirebaseError } from 'firebase/app';
import { writeBatch, collection, doc, getDocs } from 'firebase/firestore';
import type { User } from 'firebase/auth';
import { auth, firestore } from './firebase';

export const PIPELINE_TEMPLATES_COLLECTION = 'pipelineTemplates';

type PipelineTemplateForSync = {
    id: string;
    type: string;
    name: string;
    createdAt: number;
    settings: unknown;
};

const MAX_BATCH_OPS = 500;

type BatchOp =
    | { kind: 'set'; tpl: PipelineTemplateForSync }
    | { kind: 'delete'; id: string };

/**
 * Pushes the local template list to Firestore: upserts all local documents and
 * deletes cloud documents whose ids are not present locally. One-way only.
 */
export async function syncLocalTemplatesToCloud(
    user: User,
    localTemplates: PipelineTemplateForSync[]
): Promise<void> {
    if (!user?.uid) {
        return;
    }

    // Firestore чіпляє id token тільки до getAuth().currentUser (не до довільного User).
    const session = auth.currentUser;
    if (!session) {
        throw new Error('Firebase Auth: немає активної сесії. Вийдіть і увійдіть знову.');
    }
    if (session.uid !== user.uid) {
        throw new Error('Firebase Auth: uid з контексту і з сесії не збігаються.');
    }
    await session.getIdToken(true);

    const uid = session.uid;
    const col = collection(firestore, 'users', uid, PIPELINE_TEMPLATES_COLLECTION);
    const localIds = new Set(localTemplates.map((t) => t.id).filter(Boolean));

    const remoteSnap = await getDocs(col);
    const toDelete: string[] = [];
    remoteSnap.forEach((d) => {
        if (d.id && !localIds.has(d.id)) {
            toDelete.push(d.id);
        }
    });

    const ops: BatchOp[] = [
        ...localTemplates
            .filter((t) => t.id)
            .map((tpl) => ({ kind: 'set' as const, tpl })),
        ...toDelete.map((id) => ({ kind: 'delete' as const, id }))
    ];

    for (let i = 0; i < ops.length; i += MAX_BATCH_OPS) {
        const batch = writeBatch(firestore);
        const slice = ops.slice(i, i + MAX_BATCH_OPS);
        for (const op of slice) {
            if (op.kind === 'set') {
                const ref = doc(firestore, 'users', uid, PIPELINE_TEMPLATES_COLLECTION, op.tpl.id);
                batch.set(ref, {
                    id: op.tpl.id,
                    type: op.tpl.type,
                    name: op.tpl.name,
                    createdAt: op.tpl.createdAt,
                    settings: op.tpl.settings ?? {},
                    updatedAt: Date.now()
                });
            } else {
                const ref = doc(firestore, 'users', uid, PIPELINE_TEMPLATES_COLLECTION, op.id);
                batch.delete(ref);
            }
        }
        await batch.commit();
    }
}

export function formatTemplateSyncError(err: unknown): string {
    if (err && typeof err === 'object' && 'code' in err) {
        const fe = err as Partial<FirebaseError>;
        if (fe.code === 'permission-denied') {
            const vitePid = typeof import.meta.env.VITE_FIREBASE_PROJECT_ID === 'string' ? import.meta.env.VITE_FIREBASE_PROJECT_ID : '';
            let appPid = '';
            try {
                appPid = (getApp().options as { projectId?: string }).projectId ?? '';
            } catch {
                appPid = '';
            }
            return (
                'Firebase: permission-denied. Має бути той самий project у: Console, .firebaserc, VITE, перезбірка. ' +
                (vitePid ? `VITE_PROJECT_ID=${vitePid}. ` : '') +
                (appPid ? `Активний getApp().projectId=${appPid}. ` : '') +
                'Правила: users/{uid}/pipelineTemplates, auth.uid==uid. Переклади: Firebase Console → Firestore → Правила (дата публікації оновились).'
            );
        }
        if (fe.code === 'unavailable' || fe.code === 'failed-precondition') {
            return 'Firebase тимчасово недоступний. Перевірте мережу.';
        }
        if (typeof fe.message === 'string' && fe.message.length > 0) {
            return fe.message;
        }
    }
    if (err instanceof Error && err.message) {
        return err.message;
    }
    return 'Помилка синхронізації шаблонів з хмарою';
}

