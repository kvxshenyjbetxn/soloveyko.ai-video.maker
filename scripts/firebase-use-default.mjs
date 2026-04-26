#!/usr/bin/env node
/**
 * Sets the Firebase CLI active project for this repository to the same id
 * as frontend/.env (VITE_FIREBASE_PROJECT_ID) and .firebaserc (projects.default).
 * Run from repo root: npm run firebase:use
 */
import { spawnSync } from 'child_process';
import { getSyncedFirebaseProjectId, repoRoot } from './firebase-project-sync.mjs';

const project = getSyncedFirebaseProjectId();
console.error(
    `firebase use: activating project "${project}" (synced: .firebaserc ↔ frontend/.env)…`,
);

const result = spawnSync(
    'firebase',
    ['use', project],
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
