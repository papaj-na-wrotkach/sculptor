use std::path::PathBuf;

use axum::{extract::Path, routing::get, Json, Router};
use indexmap::IndexMap;
use ring::digest::{digest, SHA256};
use serde_json::Value;
use tokio::{fs, io::AsyncReadExt as _};
use walkdir::WalkDir;

use crate::{api::errors::internal_and_log, ApiError, ApiResult, AppState, ASSETS_VAR};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(versions))
        .route("/{version}", get(hashes))
        .route("/{version}/{*path}", get(download))
}

async fn versions() -> ApiResult<Json<Value>> {
    let dir_path = PathBuf::from(&*ASSETS_VAR);
    
    let mut directories = Vec::new();
    
    let mut entries = fs::read_dir(dir_path).await.map_err(internal_and_log)?;
    
    while let Some(entry) = entries.next_entry().await.map_err(internal_and_log)? {
        if entry.metadata().await.map_err(internal_and_log)?.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                let name = name.to_string();
                if !name.starts_with('.') {
                    directories.push(Value::String(name.to_string()));
                }
            }
        }
    }

    Ok(Json(serde_json::Value::Array(directories)))
}

async fn hashes(Path(version): Path<String>) -> ApiResult<Json<IndexMap<String, Value>>> {
    let map = index_assets(&version).await.map_err(internal_and_log)?;
    Ok(Json(map))
}

async fn download(Path((version, path)): Path<(String, String)>) -> ApiResult<Vec<u8>> {
    let mut file = if let Ok(file) = fs::File::open(format!("{}/{version}/{path}", *ASSETS_VAR)).await {
        file
    } else {
        return Err(ApiError::NotFound)
    };
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await.map_err(internal_and_log)?;
    Ok(buffer)
}

// non web

async fn index_assets(version: &str) -> anyhow::Result<IndexMap<String, Value>> {
    let mut map = IndexMap::new();
    let version_path = PathBuf::from(&*ASSETS_VAR).join(version);

    for entry in WalkDir::new(version_path.clone()).into_iter().filter_map(|e| e.ok()) {
        let data = match fs::read(entry.path()).await {
            Ok(d) => d,
            Err(_) => continue
        };

        let path: String = if cfg!(windows) {
            entry.path().strip_prefix(version_path.clone())?.to_string_lossy().to_string().replace("\\", "/")
        } else {
            entry.path().strip_prefix(version_path.clone())?.to_string_lossy().to_string()
        };

        map.insert(path, Value::from(faster_hex::hex_string(digest(&SHA256, &data).as_ref())));
    }

    Ok(map)
}