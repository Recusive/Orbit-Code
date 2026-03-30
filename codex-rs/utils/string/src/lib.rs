//! Utilities for truncating large chunks of output while preserving a prefix
//! and suffix on UTF-8 boundaries.

const APPROX_BYTES_PER_TOKEN: usize = 4;

/// Truncate a string to `max_bytes` using a character-count marker.
pub fn truncate_middle_chars(s: &str, max_bytes: usize) -> String {
    truncate_with_byte_estimate(s, max_bytes, /*use_tokens*/ false)
}

/// Truncate the middle of a UTF-8 string to at most `max_tokens` approximate
/// tokens, preserving the beginning and the end. Returns the possibly
/// truncated string and `Some(original_token_count)` if truncation occurred;
/// otherwise returns the original string and `None`.
pub fn truncate_middle_with_token_budget(s: &str, max_tokens: usize) -> (String, Option<u64>) {
    if s.is_empty() {
        return (String::new(), None);
    }

    if max_tokens > 0 && s.len() <= approx_bytes_for_tokens(max_tokens) {
        return (s.to_string(), None);
    }

    let truncated = truncate_with_byte_estimate(
        s,
        approx_bytes_for_tokens(max_tokens),
        /*use_tokens*/ true,
    );
    let total_tokens = u64::try_from(approx_token_count(s)).unwrap_or(u64::MAX);

    if truncated == s {
        (truncated, None)
    } else {
        (truncated, Some(total_tokens))
    }
}

fn truncate_with_byte_estimate(s: &str, max_bytes: usize, use_tokens: bool) -> String {
    if s.is_empty() {
        return String::new();
    }

    let total_chars = s.chars().count();

    if max_bytes == 0 {
        return format_truncation_marker(
            use_tokens,
            removed_units(use_tokens, s.len(), total_chars),
        );
    }

    if s.len() <= max_bytes {
        return s.to_string();
    }

    let total_bytes = s.len();
    let (left_budget, right_budget) = split_budget(max_bytes);
    let (removed_chars, left, right) = split_string(s, left_budget, right_budget);
    let marker = format_truncation_marker(
        use_tokens,
        removed_units(
            use_tokens,
            total_bytes.saturating_sub(max_bytes),
            removed_chars,
        ),
    );

    assemble_truncated_output(left, right, &marker)
}

pub fn approx_token_count(text: &str) -> usize {
    let len = text.len();
    len.saturating_add(APPROX_BYTES_PER_TOKEN.saturating_sub(1)) / APPROX_BYTES_PER_TOKEN
}

pub fn approx_bytes_for_tokens(tokens: usize) -> usize {
    tokens.saturating_mul(APPROX_BYTES_PER_TOKEN)
}

pub fn approx_tokens_from_byte_count(bytes: usize) -> u64 {
    let bytes_u64 = bytes as u64;
    bytes_u64.saturating_add((APPROX_BYTES_PER_TOKEN as u64).saturating_sub(1))
        / (APPROX_BYTES_PER_TOKEN as u64)
}

fn split_string(s: &str, beginning_bytes: usize, end_bytes: usize) -> (usize, &str, &str) {
    if s.is_empty() {
        return (0, "", "");
    }

    let len = s.len();
    let tail_start_target = len.saturating_sub(end_bytes);
    let mut prefix_end = 0usize;
    let mut suffix_start = len;
    let mut removed_chars = 0usize;
    let mut suffix_started = false;

    for (idx, ch) in s.char_indices() {
        let char_end = idx + ch.len_utf8();
        if char_end <= beginning_bytes {
            prefix_end = char_end;
            continue;
        }

        if idx >= tail_start_target {
            if !suffix_started {
                suffix_start = idx;
                suffix_started = true;
            }
            continue;
        }

        removed_chars = removed_chars.saturating_add(1);
    }

    if suffix_start < prefix_end {
        suffix_start = prefix_end;
    }

    let before = &s[..prefix_end];
    let after = &s[suffix_start..];

    (removed_chars, before, after)
}

fn split_budget(budget: usize) -> (usize, usize) {
    let left = budget / 2;
    (left, budget - left)
}

fn format_truncation_marker(use_tokens: bool, removed_count: u64) -> String {
    if use_tokens {
        format!("…{removed_count} tokens truncated…")
    } else {
        format!("…{removed_count} chars truncated…")
    }
}

fn removed_units(use_tokens: bool, removed_bytes: usize, removed_chars: usize) -> u64 {
    if use_tokens {
        approx_tokens_from_byte_count(removed_bytes)
    } else {
        u64::try_from(removed_chars).unwrap_or(u64::MAX)
    }
}

fn assemble_truncated_output(prefix: &str, suffix: &str, marker: &str) -> String {
    let mut out = String::with_capacity(prefix.len() + marker.len() + suffix.len() + 1);
    out.push_str(prefix);
    out.push_str(marker);
    out.push_str(suffix);
    out
}

// Truncate a &str to a byte budget at a char boundary (prefix)
#[inline]
pub fn take_bytes_at_char_boundary(s: &str, maxb: usize) -> &str {
    if s.len() <= maxb {
        return s;
    }
    let mut last_ok = 0;
    for (i, ch) in s.char_indices() {
        let nb = i + ch.len_utf8();
        if nb > maxb {
            break;
        }
        last_ok = nb;
    }
    &s[..last_ok]
}

// Take a suffix of a &str within a byte budget at a char boundary
#[inline]
pub fn take_last_bytes_at_char_boundary(s: &str, maxb: usize) -> &str {
    if s.len() <= maxb {
        return s;
    }
    let mut start = s.len();
    let mut used = 0usize;
    for (i, ch) in s.char_indices().rev() {
        let nb = ch.len_utf8();
        if used + nb > maxb {
            break;
        }
        start = i;
        used += nb;
        if start == 0 {
            break;
        }
    }
    &s[start..]
}

/// Sanitize a tag value to comply with metric tag validation rules:
/// only ASCII alphanumeric, '.', '_', '-', and '/' are allowed.
pub fn sanitize_metric_tag_value(value: &str) -> String {
    const MAX_LEN: usize = 256;
    let sanitized: String = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-' | '/') {
                ch
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = sanitized.trim_matches('_');
    if trimmed.is_empty() || trimmed.chars().all(|ch| !ch.is_ascii_alphanumeric()) {
        return "unspecified".to_string();
    }
    if trimmed.len() <= MAX_LEN {
        trimmed.to_string()
    } else {
        trimmed[..MAX_LEN].to_string()
    }
}

/// Find all UUIDs in a string.
#[allow(clippy::unwrap_used)]
pub fn find_uuids(s: &str) -> Vec<String> {
    static RE: std::sync::OnceLock<regex_lite::Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| {
        regex_lite::Regex::new(
            r"[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}",
        )
        .unwrap() // Unwrap is safe thanks to the tests.
    });

    re.find_iter(s).map(|m| m.as_str().to_string()).collect()
}

/// Convert a markdown-style `#L..` location suffix into a terminal-friendly
/// `:line[:column][-line[:column]]` suffix.
pub fn normalize_markdown_hash_location_suffix(suffix: &str) -> Option<String> {
    let fragment = suffix.strip_prefix('#')?;
    let (start, end) = match fragment.split_once('-') {
        Some((start, end)) => (start, Some(end)),
        None => (fragment, None),
    };
    let (start_line, start_column) = parse_markdown_hash_location_point(start)?;
    let mut normalized = String::from(":");
    normalized.push_str(start_line);
    if let Some(column) = start_column {
        normalized.push(':');
        normalized.push_str(column);
    }
    if let Some(end) = end {
        let (end_line, end_column) = parse_markdown_hash_location_point(end)?;
        normalized.push('-');
        normalized.push_str(end_line);
        if let Some(column) = end_column {
            normalized.push(':');
            normalized.push_str(column);
        }
    }
    Some(normalized)
}

fn parse_markdown_hash_location_point(point: &str) -> Option<(&str, Option<&str>)> {
    let point = point.strip_prefix('L')?;
    match point.split_once('C') {
        Some((line, column)) => Some((line, Some(column))),
        None => Some((point, None)),
    }
}

#[cfg(test)]
#[allow(warnings, clippy::all)]
mod tests {
    use super::find_uuids;
    use super::normalize_markdown_hash_location_suffix;
    use super::sanitize_metric_tag_value;
    use pretty_assertions::assert_eq;

    #[test]
    fn find_uuids_finds_multiple() {
        let input =
            "x 00112233-4455-6677-8899-aabbccddeeff-k y 12345678-90ab-cdef-0123-456789abcdef";
        assert_eq!(
            find_uuids(input),
            vec![
                "00112233-4455-6677-8899-aabbccddeeff".to_string(),
                "12345678-90ab-cdef-0123-456789abcdef".to_string(),
            ]
        );
    }

    #[test]
    fn find_uuids_ignores_invalid() {
        let input = "not-a-uuid-1234-5678-9abc-def0-123456789abc";
        assert_eq!(find_uuids(input), Vec::<String>::new());
    }

    #[test]
    fn find_uuids_handles_non_ascii_without_overlap() {
        let input = "🙂 55e5d6f7-8a7f-4d2a-8d88-123456789012abc";
        assert_eq!(
            find_uuids(input),
            vec!["55e5d6f7-8a7f-4d2a-8d88-123456789012".to_string()]
        );
    }

    #[test]
    fn sanitize_metric_tag_value_trims_and_fills_unspecified() {
        let msg = "///";
        assert_eq!(sanitize_metric_tag_value(msg), "unspecified");
    }

    #[test]
    fn sanitize_metric_tag_value_replaces_invalid_chars() {
        let msg = "bad value!";
        assert_eq!(sanitize_metric_tag_value(msg), "bad_value");
    }

    #[test]
    fn normalize_markdown_hash_location_suffix_converts_single_location() {
        assert_eq!(
            normalize_markdown_hash_location_suffix("#L74C3"),
            Some(":74:3".to_string())
        );
    }

    #[test]
    fn normalize_markdown_hash_location_suffix_converts_ranges() {
        assert_eq!(
            normalize_markdown_hash_location_suffix("#L74C3-L76C9"),
            Some(":74:3-76:9".to_string())
        );
    }
}
