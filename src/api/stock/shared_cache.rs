use rusqlite::{Connection, OptionalExtension, params};
use std::fs;
use std::path::{Path, PathBuf};

const DB_FILE_NAME: &str = "shared_stock_cache.sqlite3";

/// Відновлює медіа з shared cache у локальну папку проєкту.
pub fn restore_to_project(
    cache_dir: &str,
    kind: &str,
    provider: &str,
    asset_id: &str,
    project_dest: &Path,
) -> Result<bool, String> {
    let Some(cache_root) = normalize_cache_dir(cache_dir) else {
        return Ok(false);
    };

    let conn = open_db(&cache_root)?;
    let Some(relative_path) = lookup_relative_path(&conn, kind, provider, asset_id)? else {
        return Ok(false);
    };

    let cached_path = cache_root.join(&relative_path);
    if !cached_path.exists() {
        delete_missing_entry(&conn, kind, provider, asset_id)?;
        return Ok(false);
    }

    if cached_path == project_dest {
        touch_entry(&conn, kind, provider, asset_id)?;
        return Ok(true);
    }

    if let Some(parent) = project_dest.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Не вдалося створити папку проєкту: {e}"))?;
    }

    if let Err(error) = fs::copy(&cached_path, project_dest) {
        let _ = fs::remove_file(project_dest);
        return Err(format!(
            "Не вдалося скопіювати файл з shared cache: {error}"
        ));
    }

    touch_entry(&conn, kind, provider, asset_id)?;
    Ok(true)
}

/// Зберігає щойно завантажений файл у shared cache та реєструє його в SQLite.
pub fn store_from_project(
    cache_dir: &str,
    kind: &str,
    provider: &str,
    asset_id: &str,
    source_file: &Path,
    preferred_ext: &str,
) -> Result<(), String> {
    let Some(cache_root) = normalize_cache_dir(cache_dir) else {
        return Ok(());
    };
    if !source_file.exists() {
        return Ok(());
    }

    let conn = open_db(&cache_root)?;
    let relative_path = build_relative_path(kind, provider, asset_id, preferred_ext);
    let cached_path = cache_root.join(&relative_path);

    if let Some(parent) = cached_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Не вдалося створити папку shared cache: {e}"))?;
    }

    if cached_path != source_file {
        if let Err(error) = fs::copy(source_file, &cached_path) {
            let _ = fs::remove_file(&cached_path);
            return Err(format!("Не вдалося записати файл у shared cache: {error}"));
        }
    }

    conn.execute(
        "INSERT INTO stock_assets (kind, provider, asset_id, relative_path, updated_at, last_used_at)
         VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT(kind, provider, asset_id) DO UPDATE SET
             relative_path = excluded.relative_path,
             updated_at = CURRENT_TIMESTAMP,
             last_used_at = CURRENT_TIMESTAMP",
        params![kind, provider, asset_id, path_to_db_string(&relative_path)],
    )
    .map_err(|e| format!("Не вдалося оновити індекс shared cache: {e}"))?;

    Ok(())
}

fn normalize_cache_dir(cache_dir: &str) -> Option<PathBuf> {
    let trimmed = cache_dir.trim();
    (!trimmed.is_empty()).then(|| PathBuf::from(trimmed))
}

fn open_db(cache_root: &Path) -> Result<Connection, String> {
    fs::create_dir_all(cache_root)
        .map_err(|e| format!("Не вдалося створити корінь shared cache: {e}"))?;

    let db_path = cache_root.join(DB_FILE_NAME);
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Не вдалося відкрити SQLite shared cache: {e}"))?;

    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         CREATE TABLE IF NOT EXISTS stock_assets (
             kind TEXT NOT NULL,
             provider TEXT NOT NULL,
             asset_id TEXT NOT NULL,
             relative_path TEXT NOT NULL,
             updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
             last_used_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
             PRIMARY KEY(kind, provider, asset_id)
         );",
    )
    .map_err(|e| format!("Не вдалося ініціалізувати SQLite shared cache: {e}"))?;

    Ok(conn)
}

fn lookup_relative_path(
    conn: &Connection,
    kind: &str,
    provider: &str,
    asset_id: &str,
) -> Result<Option<String>, String> {
    conn.query_row(
        "SELECT relative_path FROM stock_assets
         WHERE kind = ?1 AND provider = ?2 AND asset_id = ?3",
        params![kind, provider, asset_id],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .map_err(|e| format!("Не вдалося прочитати shared cache: {e}"))
}

fn delete_missing_entry(
    conn: &Connection,
    kind: &str,
    provider: &str,
    asset_id: &str,
) -> Result<(), String> {
    conn.execute(
        "DELETE FROM stock_assets WHERE kind = ?1 AND provider = ?2 AND asset_id = ?3",
        params![kind, provider, asset_id],
    )
    .map_err(|e| format!("Не вдалося очистити битий запис shared cache: {e}"))?;
    Ok(())
}

fn touch_entry(
    conn: &Connection,
    kind: &str,
    provider: &str,
    asset_id: &str,
) -> Result<(), String> {
    conn.execute(
        "UPDATE stock_assets
         SET last_used_at = CURRENT_TIMESTAMP
         WHERE kind = ?1 AND provider = ?2 AND asset_id = ?3",
        params![kind, provider, asset_id],
    )
    .map_err(|e| format!("Не вдалося оновити час використання shared cache: {e}"))?;
    Ok(())
}

fn build_relative_path(kind: &str, provider: &str, asset_id: &str, preferred_ext: &str) -> PathBuf {
    let ext = normalize_extension(preferred_ext, kind);
    PathBuf::from("files")
        .join(sanitize_component(kind))
        .join(sanitize_component(provider))
        .join(format!("{}.{}", sanitize_component(asset_id), ext))
}

fn sanitize_component(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect();

    if sanitized.is_empty() {
        "unknown".to_string()
    } else {
        sanitized
    }
}

fn normalize_extension(preferred_ext: &str, kind: &str) -> String {
    let cleaned = preferred_ext
        .trim()
        .trim_start_matches('.')
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase();

    if !cleaned.is_empty() {
        cleaned
    } else if kind == "video" {
        "mp4".to_string()
    } else {
        "jpg".to_string()
    }
}

fn path_to_db_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
