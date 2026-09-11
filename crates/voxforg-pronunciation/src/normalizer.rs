//! Text normalization: symbols, currency, numbers, abbreviations.
//!
//! All transforms are deterministic and pure — no I/O, no state.

use regex::Regex;
use std::sync::OnceLock;

/// Currency symbols mapped to their spoken form.
static CURRENCY_MAP: &[(&str, &str)] = &[
    ("₹", "rupees"),
    ("$", "dollars"),
    ("€", "euros"),
    ("£", "pounds"),
    ("¥", "yen"),
    ("₩", "won"),
    ("₺", "lira"),
    ("฿", "baht"),
];

/// Compiled regex patterns (lazily initialised).
fn re_number_k() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(\d+(?:\.\d+)?)[Kk]\b").unwrap())
}

fn re_number_m() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(\d+(?:\.\d+)?)[Mm]\b").unwrap())
}

fn re_number_b() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(\d+(?:\.\d+)?)[Bb]\b").unwrap())
}

fn re_currency_amount() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // Matches currency symbols immediately followed by a number (e.g. $1,000 or ₹50.00)
    RE.get_or_init(|| Regex::new(r"([₹$€£¥₩₺฿])([\d,]+(?:\.\d{1,2})?)").unwrap())
}

fn re_comma_number() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(\d),(\d{3})").unwrap())
}

fn re_percent() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(\d+(?:\.\d+)?)%").unwrap())
}

/// Stateless text normalizer.
pub struct TextNormalizer;

impl TextNormalizer {
    /// Normalize `text` through all normalization passes.
    ///
    /// Pass order matters: currency-with-symbol must run before bare symbol
    /// replacement to avoid "dollars1,000 dollars" artifacts.
    pub fn normalize(text: &str) -> String {
        let s = Self::normalize_currency_amounts(text);
        let s = Self::normalize_scale_suffixes(&s);
        let s = Self::normalize_percentages(&s);
        Self::strip_comma_separators(&s)
    }

    /// Convert `$1,000` → `1000 dollars`, `₹50.00` → `50 rupees`, etc.
    fn normalize_currency_amounts(text: &str) -> String {
        re_currency_amount()
            .replace_all(text, |caps: &regex::Captures<'_>| {
                let symbol = caps.get(1).map_or("", |m| m.as_str());
                let amount = caps.get(2).map_or("", |m| m.as_str());
                let spoken_currency = CURRENCY_MAP
                    .iter()
                    .find(|(s, _)| *s == symbol)
                    .map(|(_, name)| *name)
                    .unwrap_or("units");
                // Remove comma separators from amount
                let clean_amount = amount.replace(',', "");
                format!("{clean_amount} {spoken_currency}")
            })
            .to_string()
    }

    /// Convert `1K` → `1000`, `2.5M` → `2500000`, `1B` → `1000000000`.
    fn normalize_scale_suffixes(text: &str) -> String {
        let s = re_number_k().replace_all(text, |caps: &regex::Captures<'_>| {
            let n: f64 = caps[1].parse().unwrap_or(0.0);
            format!("{}", (n * 1_000.0) as u64)
        });
        let s = re_number_m().replace_all(&s, |caps: &regex::Captures<'_>| {
            let n: f64 = caps[1].parse().unwrap_or(0.0);
            format!("{}", (n * 1_000_000.0) as u64)
        });
        re_number_b()
            .replace_all(&s, |caps: &regex::Captures<'_>| {
                let n: f64 = caps[1].parse().unwrap_or(0.0);
                format!("{}", (n * 1_000_000_000.0) as u64)
            })
            .to_string()
    }

    /// Convert `95%` → `95 percent`.
    fn normalize_percentages(text: &str) -> String {
        re_percent().replace_all(text, "$1 percent").to_string()
    }

    /// Strip comma-separated thousands: `1,000` → `1000`.
    fn strip_comma_separators(text: &str) -> String {
        // Apply repeatedly until stable (handles 1,000,000)
        let mut prev = text.to_string();
        loop {
            let next = re_comma_number().replace_all(&prev, "$1$2").to_string();
            if next == prev {
                break;
            }
            prev = next;
        }
        prev
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_dollar_amount() {
        let r = TextNormalizer::normalize("earn $1,000 today");
        assert!(r.contains("1000 dollars"), "got: {r}");
    }

    #[test]
    fn normalizes_rupee_amount() {
        let r = TextNormalizer::normalize("costs ₹50,000");
        assert!(r.contains("50000 rupees"), "got: {r}");
    }

    #[test]
    fn normalizes_k_suffix() {
        let r = TextNormalizer::normalize("has 2K followers");
        assert!(r.contains("2000"), "got: {r}");
    }

    #[test]
    fn normalizes_m_suffix() {
        let r = TextNormalizer::normalize("1.5M downloads");
        assert!(r.contains("1500000"), "got: {r}");
    }

    #[test]
    fn normalizes_percent() {
        let r = TextNormalizer::normalize("CPU at 87%");
        assert!(r.contains("87 percent"), "got: {r}");
    }

    #[test]
    fn strips_comma_thousands() {
        let r = TextNormalizer::normalize("value is 1,000,000");
        assert!(r.contains("1000000"), "got: {r}");
    }

    #[test]
    fn leaves_regular_text_unchanged() {
        let input = "Hello world";
        assert_eq!(TextNormalizer::normalize(input), input);
    }
}
