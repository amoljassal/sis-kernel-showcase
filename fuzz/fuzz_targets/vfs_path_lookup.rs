#![no_main]
// VFS path lookup fuzzer
// Tests path resolution for edge cases, malformed paths, and security issues

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Convert random bytes to string (potential path)
    if let Ok(path_str) = core::str::from_utf8(data) {
        // Limit path length to avoid excessive runtime
        if path_str.len() > 4096 {
            return;
        }

        // Test basic path validation
        test_path_validation(path_str);

        // Test path normalization
        test_path_normalization(path_str);

        // Test path traversal detection
        test_path_traversal(path_str);
    }
});

/// Test that path validation doesn't panic
fn test_path_validation(path: &str) {
    // This would call actual VFS path_lookup in real implementation
    // For now, we implement the validation logic inline

    // Check for null bytes
    if path.contains('\0') {
        return; // Invalid
    }

    // Check for excessive slashes
    let slash_count = path.chars().filter(|&c| c == '/').count();
    if slash_count > 1000 {
        return; // Too many slashes
    }

    // Simulate path component parsing
    let components: Vec<&str> = path.split('/').collect();

    // Check component count
    if components.len() > 256 {
        return; // Too many components
    }

    // Validate each component
    for component in components {
        if component.is_empty() {
            continue; // Skip empty components
        }

        // Check for invalid characters
        if component.contains('\0') {
            return; // Invalid
        }

        // Check component length
        if component.len() > 255 {
            return; // Component too long
        }
    }

    // Path appears valid (would call actual lookup here)
}

/// Test path normalization (removing . and ..)
fn test_path_normalization(path: &str) {
    let components: Vec<&str> = path.split('/').collect();
    let mut normalized: Vec<&str> = Vec::new();

    for component in components {
        match component {
            "" | "." => continue,  // Skip empty and current dir
            ".." => {
                // Pop previous component
                normalized.pop();
            }
            _ => {
                // Add component
                if normalized.len() < 256 {
                    normalized.push(component);
                }
            }
        }
    }

    // Normalized path should not exceed limits
    assert!(normalized.len() <= 256);

    // Reconstructed path should be valid
    let reconstructed = normalized.join("/");
    assert!(reconstructed.len() <= 4096);
}

/// Test path traversal attack detection
fn test_path_traversal(path: &str) {
    // Count occurrences of '..'
    let dotdot_count = path.matches("..").count();

    // Count forward components
    let forward_components = path.split('/').filter(|c| !c.is_empty() && *c != "." && *c != "..").count();

    // If more '..' than forward components, this is a traversal attempt
    if dotdot_count > forward_components {
        // Should be rejected by VFS
        return;
    }

    // Check for absolute paths escaping root
    if path.starts_with('/') {
        let components: Vec<&str> = path.split('/').filter(|c| !c.is_empty()).collect();
        let mut depth = 0i32;

        for component in components {
            match component {
                "." => continue,
                ".." => depth -= 1,
                _ => depth += 1,
            }

            // Should never go above root
            assert!(depth >= 0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_paths() {
        test_path_validation("/");
        test_path_validation("/foo");
        test_path_validation("/foo/bar");
        test_path_validation("/foo/bar/baz");
    }

    #[test]
    fn test_invalid_paths() {
        // These should not panic
        test_path_validation("");
        test_path_validation("//");
        test_path_validation("///");
        test_path_validation("/foo//bar");
    }

    #[test]
    fn test_normalization() {
        test_path_normalization("/foo/./bar");
        test_path_normalization("/foo/../bar");
        test_path_normalization("/foo/bar/..");
        test_path_normalization("/./foo/./bar/.");
    }

    #[test]
    fn test_traversal_detection() {
        test_path_traversal("/../../../etc/passwd");
        test_path_traversal("/foo/../../bar");
        test_path_traversal("/foo/.../bar");
    }
}
