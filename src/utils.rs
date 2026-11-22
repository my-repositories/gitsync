/// Parse the latest version tag from a GitHub release JSON response.
/// Handles both `"tag_name":"v1.2.3"` and `"tag_name": "v1.2.3"` formats.
pub fn parse_latest_version(response: &str, fallback: &str) -> String {
    response
        .find(r#""tag_name""#)
        .and_then(|pos| {
            let after = &response[pos + r#""tag_name""#.len()..];
            after
                .trim_start()
                .strip_prefix(':')
                .and_then(|s| s.trim_start().strip_prefix('"'))
                .and_then(|v| v.split('"').next())
        })
        .unwrap_or(fallback)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_space_after_colon() {
        // GitHub API returns "tag_name": "v1.1.9" (with space)
        let json = r#"{"tag_name": "v1.1.9", "name": "Release"}"#;
        assert_eq!(parse_latest_version(json, "0.0.0"), "v1.1.9");
    }

    #[test]
    fn handles_no_space_after_colon() {
        // Some APIs return "tag_name":"v1.1.9" (no space)
        let json = r#"{"tag_name":"v2.0.0"}"#;
        assert_eq!(parse_latest_version(json, "0.0.0"), "v2.0.0");
    }

    #[test]
    fn falls_back_on_missing_tag() {
        let json = r#"{"name": "No tag here"}"#;
        assert_eq!(parse_latest_version(json, "fallback"), "fallback");
    }

    #[test]
    fn falls_back_on_empty_response() {
        assert_eq!(parse_latest_version("", "1.0.0"), "1.0.0");
    }

    #[test]
    fn handles_nested_json() {
        let json = r#"{"release":{"tag_name": "v3.2.1"},"url":"https://api.github.com/repos/owner/repo/releases/123"}"#;
        assert_eq!(parse_latest_version(json, "0.0.0"), "v3.2.1");
    }

    #[test]
    fn strips_v_prefix_from_fallback() {
        assert_eq!(
            parse_latest_version(r#"{"tag_name": "v1.2.3"}"#, "v0.0.1"),
            "v1.2.3"
        );
    }
}
