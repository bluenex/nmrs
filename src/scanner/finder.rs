use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use walkdir::WalkDir;
use anyhow::Result;
use std::fs;
use std::process::Command;
use rayon::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeModulesInfo {
    pub path: PathBuf,
    pub size: u64,
}

pub async fn find_node_modules_with_sizes(target_path: PathBuf) -> Result<Vec<NodeModulesInfo>> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    pb.set_message("Scanning for node_modules directories...");

    let node_modules_paths: Vec<PathBuf> = WalkDir::new(&target_path)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if path.file_name() == Some(std::ffi::OsStr::new("node_modules")) 
                && entry.file_type().is_dir() 
                && !is_inside_node_modules(&path, &target_path) {
                pb.set_message(format!("Found {} node_modules directories...", 1));
                Some(path.to_path_buf())
            } else {
                let short_path = if path.to_string_lossy().len() > 50 {
                    let path_str = path.to_string_lossy();
                    if path_str.len() > 47 {
                        format!("...{}", &path_str.chars().skip(path_str.chars().count() - 47).collect::<String>())
                    } else {
                        path_str.to_string()
                    }
                } else {
                    path.to_string_lossy().to_string()
                };
                pb.set_message(format!("Scanning: {}", short_path));
                None
            }
        })
        .collect();

    pb.set_message(format!("Calculating sizes for {} directories...", node_modules_paths.len()));

    let pb_arc = Arc::new(pb);
    let pb_clone = pb_arc.clone();

    let results: Result<Vec<NodeModulesInfo>, anyhow::Error> = tokio::task::spawn_blocking(move || {
        let results: Result<Vec<_>, _> = node_modules_paths
            .par_iter()
            .enumerate()
            .map(|(i, path)| {
                pb_clone.set_message(format!("Calculating size of node_modules #{}/{}", i + 1, node_modules_paths.len()));
                let size = calculate_directory_size_sync(path)?;
                Ok(NodeModulesInfo {
                    path: path.clone(),
                    size,
                })
            })
            .collect();
        results
    }).await?;

    pb_arc.finish_with_message("Scan complete!");

    let mut node_modules_infos = results?;
    node_modules_infos.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(node_modules_infos)
}

fn is_inside_node_modules(path: &std::path::Path, root: &std::path::Path) -> bool {
    let relative_path = path.strip_prefix(root).unwrap_or(path);
    let components: Vec<_> = relative_path.components().collect();
    
    for (i, component) in components.iter().enumerate() {
        if component.as_os_str() == "node_modules" && i < components.len() - 1 {
            return true;
        }
    }
    false
}

fn calculate_directory_size_sync(dir_path: &std::path::Path) -> Result<u64> {
    match try_du_command_sync(dir_path) {
        Ok(size) => Ok(size),
        Err(_) => calculate_size_manually_sync(dir_path),
    }
}

fn try_du_command_sync(dir_path: &std::path::Path) -> Result<u64> {
    let output = Command::new("du")
        .arg("-sb")
        .arg(dir_path)
        .output()?;
    
    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)?;
        let size_str = stdout.split('\t').next().unwrap_or("0");
        Ok(size_str.parse::<u64>()?)
    } else {
        let output = Command::new("du")
            .arg("-sk")
            .arg(dir_path)
            .output()?;
        
        if output.status.success() {
            let stdout = String::from_utf8(output.stdout)?;
            let size_str = stdout.split('\t').next().unwrap_or("0");
            Ok(size_str.parse::<u64>()? * 1024)
        } else {
            anyhow::bail!("du command failed")
        }
    }
}

fn calculate_size_manually_sync(dir_path: &std::path::Path) -> Result<u64> {
    let total_size: u64 = WalkDir::new(dir_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| fs::metadata(entry.path()).ok())
        .map(|metadata| metadata.len())
        .sum();
    
    Ok(total_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_is_inside_node_modules() {
        let root = Path::new("/project");
        
        // Not inside node_modules
        assert!(!is_inside_node_modules(Path::new("/project/src/node_modules"), root));
        assert!(!is_inside_node_modules(Path::new("/project/node_modules"), root));
        
        // Inside node_modules  
        assert!(is_inside_node_modules(Path::new("/project/node_modules/package/node_modules"), root));
        assert!(is_inside_node_modules(Path::new("/project/src/node_modules/dep/node_modules"), root));
    }

    #[test]
    fn test_node_modules_info_serialization() {
        let info = NodeModulesInfo {
            path: PathBuf::from("/test/node_modules"),
            size: 1024,
        };
        
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: NodeModulesInfo = serde_json::from_str(&json).unwrap();
        
        assert_eq!(info.path, deserialized.path);
        assert_eq!(info.size, deserialized.size);
    }
}