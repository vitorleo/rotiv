/// Convert a Rotiv route path to an axum route string.
/// In Phase 2 both use the same `:param` syntax — no translation needed.
/// This function exists for documentation clarity and future-proofing.
pub fn route_to_axum_path(route_path: &str) -> &str {
    route_path
}

/// Check whether `request_path` matches `route_pattern`.
/// Handles `:param` wildcards. Returns extracted params on match.
pub fn matches(
    route_pattern: &str,
    request_path: &str,
) -> Option<std::collections::HashMap<String, String>> {
    let pattern_segs: Vec<&str> = route_pattern.trim_matches('/').split('/').collect();
    let request_segs: Vec<&str> = request_path.trim_matches('/').split('/').collect();

    if pattern_segs.len() != request_segs.len() {
        // Special case: root "/" always matches "/"
        if route_pattern == "/" && request_path == "/" {
            return Some(std::collections::HashMap::new());
        }
        return None;
    }

    let mut params = std::collections::HashMap::new();

    for (pat, req) in pattern_segs.iter().zip(request_segs.iter()) {
        if let Some(param_name) = pat.strip_prefix(':') {
            params.insert(param_name.to_string(), req.to_string());
        } else if *pat != *req {
            return None;
        }
    }

    Some(params)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match() {
        let params = matches("/about", "/about").unwrap();
        assert!(params.is_empty());
    }

    #[test]
    fn param_extraction() {
        let params = matches("/users/:id", "/users/42").unwrap();
        assert_eq!(params.get("id").unwrap(), "42");
    }

    #[test]
    fn root_matches_root() {
        let params = matches("/", "/").unwrap();
        assert!(params.is_empty());
    }

    #[test]
    fn no_match_different_segments() {
        assert!(matches("/about", "/contact").is_none());
    }

    #[test]
    fn no_match_different_depth() {
        assert!(matches("/users/:id", "/users/42/posts").is_none());
    }
}
