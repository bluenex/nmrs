use std::path::PathBuf;
use anyhow::Result;
use inquire::{MultiSelect, Select, Confirm};
use tokio::fs;
use crate::scanner::finder::{find_node_modules_with_sizes, NodeModulesInfo};
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
            println!("No recent cache found, scanning directories...");
            let results = find_node_modules_with_sizes(resolved_path.clone()).await?;
            set_cached_results(&resolved_path, &results).await?;
            results
        }
    };

    if node_modules_infos.is_empty() {
        println!("No node_modules directories found.");
        return Ok(());
    }

    println!("\nFound {} node_modules directories:\n", node_modules_infos.len());

    let choices: Vec<String> = node_modules_infos
        .iter()
        .map(|info| format!("{:>6} {}", format_bytes(info.size), info.path.display()))
        .collect();

    let selected_items = MultiSelect::new("Select directories to remove:", choices)
        .with_page_size(15)
        .prompt()?;

    if selected_items.is_empty() {
        println!("No directories selected. Operation cancelled.");
        return Ok(());
    }

    let selected_dirs: Vec<&NodeModulesInfo> = selected_items
        .iter()
        .filter_map(|selected| {
            node_modules_infos.iter().find(|info| {
                let formatted = format!("{:>6} {}", format_bytes(info.size), info.path.display());
                formatted == *selected
            })
        })
        .collect();

    let total_selected_size: u64 = selected_dirs.iter().map(|dir| dir.size).sum();

    println!("\nSelected {} directories for removal:", selected_dirs.len());
    for dir in &selected_dirs {
        println!("  • {:>6} {}", format_bytes(dir.size), dir.path.display());
    }
    println!("\nTotal space to be freed: {}", format_bytes(total_selected_size));

    let action = Select::new(
        "What would you like to do?",
        vec!["Continue with removal", "Cancel"],
    )
    .prompt()?;

    if action == "Cancel" {
        println!("Operation cancelled.");
        return Ok(());
    }

    let confirmed = Confirm::new(&format!(
        "Are you sure you want to delete these {} directories? This cannot be undone.",
        selected_dirs.len()
    ))
    .with_default(false)
    .prompt()?;

    if !confirmed {
        println!("Operation cancelled.");
        return Ok(());
    }

    remove_directories(&selected_dirs).await?;

    println!(
        "\n✓ Successfully removed {} directories, freed {}",
        selected_dirs.len(),
        format_bytes(total_selected_size)
    );

    Ok(())
}

async fn remove_directories(directories: &[&NodeModulesInfo]) -> Result<()> {
    for (i, dir) in directories.iter().enumerate() {
        let short_path = if dir.path.to_string_lossy().len() > 50 {
            let path_str = dir.path.to_string_lossy();
            if path_str.len() > 47 {
                format!("...{}", path_str.chars().skip(path_str.chars().count() - 47).collect::<String>())
            } else {
                path_str.to_string()
            }
        } else {
            dir.path.to_string_lossy().to_string()
        };

        println!("Removing directories... ({}/{}) {}", i + 1, directories.len(), short_path);

        fs::remove_dir_all(&dir.path).await
            .map_err(|e| anyhow::anyhow!("Failed to remove {}: {}", dir.path.display(), e))?;
    }

    Ok(())
}