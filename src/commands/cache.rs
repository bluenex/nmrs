use anyhow::Result;
use crate::cache::storage::clear_cache;

pub async fn clear() -> Result<()> {
    clear_cache().await?;
    Ok(())
}