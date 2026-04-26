import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
export const repoRoot = path.resolve(__dirname, '..');

const readFirebasercDefault = () => {
    const p = path.join(repoRoot, '.firebaserc');
    if (!fs.existsSync(p)) {
        throw new Error('Missing .firebaserc in repo root.');
    }
    const json = JSON.parse(fs.readFileSync(p, 'utf8'));
    const id = json?.projects?.default;
    if (!id || typeof id !== 'string') {
        throw new Error('.firebaserc must define projects.default (Firebase project id).');
    }
    return id.trim();
};

const parseEnvFile = (text) => {
    const out = new Map();
    for (const rawLine of text.split(/\r?\n/)) {
        const line = rawLine.trim();
        if (!line || line.startsWith('#')) {
            continue;
        }
        const eq = line.indexOf('=');
        if (eq === -1) {
            continue;
        }
        const key = line.slice(0, eq).trim();
        let val = line.slice(eq + 1).trim();
        if (
            (val.startsWith('"') && val.endsWith('"')) ||
            (val.startsWith("'") && val.endsWith("'"))
        ) {
            val = val.slice(1, -1);
        }
        const hash = val.indexOf('#');
        if (hash !== -1) {
            val = val.slice(0, hash).trim();
        }
        out.set(key, val);
    }
    return out;
};

const readViteFirebaseProjectId = () => {
    const envPath = path.join(repoRoot, 'frontend', '.env');
    if (!fs.existsSync(envPath)) {
        throw new Error(
            'Missing frontend/.env — add VITE_FIREBASE_PROJECT_ID so it matches .firebaserc default.',
        );
    }
    const vars = parseEnvFile(fs.readFileSync(envPath, 'utf8'));
    const id = vars.get('VITE_FIREBASE_PROJECT_ID');
    if (!id) {
        throw new Error('frontend/.env must define VITE_FIREBASE_PROJECT_ID.');
    }
    return id.trim();
};

/**
 * Ensures the Firebase CLI default project matches the Vite app config.
 * @returns {string} project id to pass to `firebase deploy --project`
 */
export const getSyncedFirebaseProjectId = () => {
    const fromRc = readFirebasercDefault();
    const fromEnv = readViteFirebaseProjectId();
    if (fromRc !== fromEnv) {
        throw new Error(
            `Firebase project mismatch:\n` +
                `  .firebaserc projects.default = "${fromRc}"\n` +
                `  frontend/.env VITE_FIREBASE_PROJECT_ID = "${fromEnv}"\n` +
                `Align them before deploying rules (same id the app uses at runtime).`,
        );
    }
    return fromRc;
};
