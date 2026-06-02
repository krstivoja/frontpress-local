//! Small pure helpers: slugs, ids, passwords, semantic-version comparison.
//! Kept dependency-light and unit-tested (see `#[cfg(test)]` at the bottom).

use rand::Rng;

/// Filesystem- and URL-safe slug from a human site name.
/// "My Cool Site!" -> "my-cool-site". Always non-empty (falls back to "site").
pub fn slugify(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut prev_dash = false;
    for ch in name.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "site".to_string()
    } else {
        trimmed
    }
}

/// 16-char lowercase-hex random identifier.
pub fn random_id() -> String {
    let mut rng = rand::thread_rng();
    (0..16)
        .map(|_| format!("{:x}", rng.gen_range(0u8..16)))
        .collect()
}

/// Human-typable random password (no ambiguous chars), ~18 chars.
pub fn random_password() -> String {
    const CHARS: &[u8] = b"abcdefghjkmnpqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rng = rand::thread_rng();
    (0..18)
        .map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char)
        .collect()
}

/// URL-safe one-time token for the auto-login handshake.
pub fn random_token() -> String {
    let mut rng = rand::thread_rng();
    (0..32)
        .map(|_| format!("{:x}", rng.gen_range(0u8..16)))
        .collect()
}

/// Parse "8.3.9" / "8.3" / "8" into a (major, minor, patch) tuple,
/// padding missing components with 0. Non-numeric parts become 0.
pub fn parse_version(v: &str) -> (u32, u32, u32) {
    let mut it = v
        .trim()
        .split(['.', '-'])
        .map(|p| p.parse::<u32>().unwrap_or(0));
    (
        it.next().unwrap_or(0),
        it.next().unwrap_or(0),
        it.next().unwrap_or(0),
    )
}

/// True when `have` >= `min` (semver-ish, patch-padded).
pub fn version_at_least(have: &str, min: &str) -> bool {
    parse_version(have) >= parse_version(min)
}

/// "8.3.9" -> "8.3" (major.minor only).
pub fn minor_of(v: &str) -> String {
    let (maj, min, _) = parse_version(v);
    format!("{maj}.{min}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("My Cool Site!"), "my-cool-site");
        assert_eq!(slugify("  Hello   World  "), "hello-world");
        assert_eq!(slugify("...."), "site");
        assert_eq!(slugify("Café_2024"), "caf-2024");
    }

    #[test]
    fn version_parsing_and_compare() {
        assert_eq!(parse_version("8.3.9"), (8, 3, 9));
        assert_eq!(parse_version("8.1"), (8, 1, 0));
        assert!(version_at_least("8.3.9", "8.1"));
        assert!(version_at_least("8.1.0", "8.1"));
        assert!(!version_at_least("8.0.30", "8.1"));
        assert!(!version_at_least("7.4.33", "8.1"));
        assert!(version_at_least("8.4.1", "8.1"));
    }

    #[test]
    fn minor_extraction() {
        assert_eq!(minor_of("8.3.9"), "8.3");
        assert_eq!(minor_of("8.1.34"), "8.1");
    }

    #[test]
    fn ids_have_expected_shape() {
        assert_eq!(random_id().len(), 16);
        assert_eq!(random_token().len(), 32);
        assert!(random_password().len() >= 16);
    }
}
