pub fn normalize_optional_text(raw: Option<&str>) -> Option<String> {
    raw.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

#[cfg(test)]
mod tests {
    use super::normalize_optional_text;

    #[test]
    fn normalize_optional_text_trims_and_filters_empty() {
        assert_eq!(normalize_optional_text(None), None);
        assert_eq!(normalize_optional_text(Some("   ")), None);
        assert_eq!(
            normalize_optional_text(Some("  hello world  ")),
            Some("hello world".to_string())
        );
    }
}
