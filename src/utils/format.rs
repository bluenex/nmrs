pub fn format_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0B".to_string();
    }

    let k = 1024u64;
    let sizes = ["B", "K", "M", "G", "T"];
    let i = (bytes as f64).log(k as f64).floor() as usize;
    
    if i >= sizes.len() {
        return format!("{}T", (bytes / k.pow(4)));
    }

    let size = bytes as f64 / k.pow(i as u32) as f64;
    
    if i == 0 {
        format!("{}{}", size.round() as u64, sizes[i])
    } else {
        format!("{}{}", size.round() as u64, sizes[i])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0B");
        assert_eq!(format_bytes(512), "512B");
        assert_eq!(format_bytes(1024), "1K");
        assert_eq!(format_bytes(1536), "2K");
        assert_eq!(format_bytes(1048576), "1M");
        assert_eq!(format_bytes(1073741824), "1G");
        assert_eq!(format_bytes(1099511627776), "1T");
    }
}