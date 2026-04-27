#!/usr/bin/env node
import { spawnSync } from 'child_process';
import { getSyncedFirebaseProjectId, repoRoot } from './firebase-project-sync.mjs';

try {
    const project = getSyncedFirebaseProjectId();
    console.error(`Deploying Firestore + Realtime + Storage rules to "${project}"…`);

    const result = spawnSync(
        'firebase',
        ['deploy', '--only', 'firestore:rules,database,storage', '--project', project],
        {
            cwd: repoRoot,
            stdio: 'inherit',
            shell: process.platform === 'win32',
        },
    );

    if (result.error) {
        console.error(result.error);
        process.exit(1);
    }
    process.exit(result.status === null ? 1 : result.status);
} catch (err) {
    console.error(err instanceof Error ? err.message : err);
    process.exit(1);
}
