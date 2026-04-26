#!/usr/bin/env node
import { getSyncedFirebaseProjectId } from './firebase-project-sync.mjs';

try {
    const id = getSyncedFirebaseProjectId();
    console.error(`Firebase project OK: ${id} (.firebaserc ↔ frontend/.env)`);
    if (process.argv.includes('--print-project')) {
        process.stdout.write(`${id}\n`);
    }
} catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    console.error(message);
    process.exit(1);
}
