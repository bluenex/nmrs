use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::Result;
use tokio::fs;
use dirs::cache_dir;
use base64::Engine;
use crate::scanner::finder::NodeModulesInfo;

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    path: String,
    results: Vec<NodeModulesInfo>,
    timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct CacheData {
    entries: HashMap<String, CacheEntry>,
}

const CACHE_TTL_MS: u64 = 10 * 60 * 1000; // 10 minutes

fn get_cache_dir() -> Result<PathBuf> {
    let cache_dir = cache_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine cache directory"))?
        .join("nmrs");
    Ok(cache_dir)
}

fn get_cache_file() -> Result<PathBuf> {
    Ok(get_cache_dir()?.join("cache.json"))
}

async fn ensure_cache_dir() -> Result<()> {
    let cache_dir = get_cache_dir()?;
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir).await?;
    }
    Ok(())
}

async fn load_cache() -> Result<CacheData> {
    let cache_file = get_cache_file()?;
    
    if cache_file.exists() {
        let content = fs::read_to_string(&cache_file).await?;
        let cache_data: CacheData = serde_json::from_str(&content)?;
        Ok(cache_data)
    } else {
        Ok(CacheData::default())
    }
}

async fn save_cache(cache_data: &CacheData) -> Result<()> {
    ensure_cache_dir().await?;
    let cache_file = get_cache_file()?;
    let content = serde_json::to_string_pretty(cache_data)?;
    fs::write(&cache_file, content).await?;
    Ok(())
}

fn get_cache_key(path: &PathBuf) -> String {
    let path_str = path.to_string_lossy();
    base64::engine::general_purpose::STANDARD.encode(path_str.as_bytes())
}

fn is_entry_valid(entry: &CacheEntry) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    now - entry.timestamp < CACHE_TTL_MS
}

pub async fn get_cached_results(path: &PathBuf) -> Result<Option<Vec<NodeModulesInfo>>> {
    let cache_data = load_cache().await?;
    let key = get_cache_key(path);
    
    if let Some(entry) = cache_data.entries.get(&key) {
        if is_entry_valid(entry) {
            return Ok(Some(entry.results.clone()));
        }
    }
    
    Ok(None)
}

pub async fn set_cached_results(path: &PathBuf, results: &[NodeModulesInfo]) -> Result<()> {
    let mut cache_data = load_cache().await.unwrap_or_default();
    let key = get_cache_key(path);
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    let entry = CacheEntry {
        path: path.to_string_lossy().to_string(),
        results: results.to_vec(),
        timestamp,
    };
    
    cache_data.entries.insert(key, entry);
    
    cache_data.entries.retain(|_, entry| is_entry_valid(entry));
    
    save_cache(&cache_data).await?;
    Ok(())
}

pub async fn clear_cache() -> Result<()> {
    let cache_data = CacheData::default();
    save_cache(&cache_data).await?;
    Ok(())
}