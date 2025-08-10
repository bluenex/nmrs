use std::path::PathBuf;
use anyhow::Result;
use crate::scanner::finder::find_node_modules_with_sizes;
use crate::cache::storage::{get_cached_results, set_cached_results};
use crate::utils::format::format_bytes;

pub async fn execute(target_path: PathBuf) -> Result<()> {
    let resolved_path = target_path.canonicalize()
        .unwrap_or_else(|_| target_path.clone());

    let node_modules_infos = match get_cached_results(&resolved_path).await? {
        Some(cached_results) => {
            println!("Using cached results...");
            cached_results
        }
        None => {
            let results = find_node_modules_with_sizes(resolved_path.clone()).await?;
            set_cached_results(&resolved_path, &results).await?;
            results
        }
    };

    if node_modules_infos.is_empty() {
        println!("No node_modules directories found.");
        return Ok(());
    }

    let mut total_size = 0u64;
    for info in &node_modules_infos {
        println!("{}\t{}", format_bytes(info.size), info.path.display());
        total_size += info.size;
    }

    println!();
    println!("{}\tTOTAL", format_bytes(total_size));

    Ok(())
}