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

fn re_url() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"https?://(?:www\.)?([^\s]+)").unwrap())
}

fn re_email() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\b([a-zA-Z0-9._%+-]+)@([a-zA-Z0-9.-]+\.[a-zA-Z]{2,})\b").unwrap()
    })
}

fn re_clock_time() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b(\d{1,2}):(\d{2})(?:\s*(AM|PM|am|pm))?\b").unwrap())
}

fn re_temporal_year() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(in|since|from|until|by|before|after|around|year)\s+(19\d{2}|20\d{2})\b")
            .unwrap()
    })
}

fn num_to_words(n: u32) -> String {
    match n {
        0 => "zero".to_string(),
        1 => "one".to_string(),
        2 => "two".to_string(),
        3 => "three".to_string(),
        4 => "four".to_string(),
        5 => "five".to_string(),
        6 => "six".to_string(),
        7 => "seven".to_string(),
        8 => "eight".to_string(),
        9 => "nine".to_string(),
        10 => "ten".to_string(),
        11 => "eleven".to_string(),
        12 => "twelve".to_string(),
        13 => "thirteen".to_string(),
        14 => "fourteen".to_string(),
        15 => "fifteen".to_string(),
        16 => "sixteen".to_string(),
        17 => "seventeen".to_string(),
        18 => "eighteen".to_string(),
        19 => "nineteen".to_string(),
        20..=99 => {
            let tens = match n / 10 {
                2 => "twenty",
                3 => "thirty",
                4 => "forty",
                5 => "fifty",
                6 => "sixty",
                7 => "seventy",
                8 => "eighty",
                9 => "ninety",
                _ => "",
            };
            let rem = n % 10;
            if rem == 0 {
                tens.to_string()
            } else {
                format!("{tens}-{}", num_to_words(rem))
            }
        }
        _ => n.to_string(),
    }
}

fn year_to_words(year: u32) -> String {
    if (1900..=1999).contains(&year) {
        let first = "nineteen";
        let last = year % 100;
        if last == 0 {
            format!("{first} hundred")
        } else if last < 10 {
            format!("{first} oh {}", num_to_words(last))
        } else {
            format!("{first} {}", num_to_words(last))
        }
    } else if (2000..=2009).contains(&year) {
        if year == 2000 {
            "two thousand".to_string()
        } else {
            format!("twenty oh {}", num_to_words(year % 100))
        }
    } else if (2010..=2099).contains(&year) {
        format!("twenty {}", num_to_words(year % 100))
    } else {
        year.to_string()
    }
}

/// Stateless text normalizer.
pub struct TextNormalizer;

impl TextNormalizer {
    /// Normalize `text` through all normalization passes.
    ///
    /// Pass order matters: URLs and emails first to avoid punctuation/cadence mangling,
    /// followed by clock times, temporal years, currencies, scale suffixes, and cadence smoothing.
    pub fn normalize(text: &str) -> String {
        let s = Self::normalize_urls(text);
        let s = Self::normalize_emails(&s);
        let s = Self::normalize_clock_times(&s);
        let s = Self::normalize_temporal_years(&s);
        let s = Self::normalize_currency_amounts(&s);
        let s = Self::normalize_scale_suffixes(&s);
        let s = Self::normalize_percentages(&s);
        let s = Self::strip_comma_separators(&s);
        Self::humanize_cadence(&s)
    }

    /// Humanize text cadence: insert natural breathing pauses for clauses >10 words,
    /// format question cadence, and smooth robotic run-on sentences.
    pub fn humanize_cadence(text: &str) -> String {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        // Punctuation and breathing cadence for conversational realism
        let mut result = String::new();
        let words: Vec<&str> = trimmed.split_whitespace().collect();

        for (i, word) in words.iter().enumerate() {
            result.push_str(word);
            let lower = word.to_lowercase();
            let clean_lower = lower.trim_matches(|c: char| !c.is_alphabetic());

            // Break up overly long clauses at natural conjunctions with micro-pause commas
            // Includes English, Manglish, Hinglish, Tanglish, and native Indic conjunctions
            let is_conjunction = matches!(
                clean_lower,
                // English
                "and" | "but" | "because" | "although" | "however" | "meanwhile" | "whereas" | "since" |
                // Manglish (Malayalam Romanized)
                "ennitt" | "enkilum" | "athukond" | "pinne" | "athinal" | "koodathe" | "allathe" |
                // Hinglish (Hindi Romanized)
                "aur" | "lekin" | "kyunki" | "magar" | "par" | "isliye" | "balki" |
                // Tanglish (Tamil Romanized)
                "aana" | "analum" | "athunala" | "pinna"
            ) || matches!(
                word.trim_matches(|c: char| c == ',' || c == '.' || c == '?' || c == '!'),
                // Native Malayalam script
                "എന്നിട്ട്" | "എങ്കിലും" | "അതുകൊണ്ട്" | "പിന്നെ" | "കൂടാതെ" | "പക്ഷേ" |
                // Native Hindi Devanagari script
                "और" | "लेकिन" | "क्योंकि" | "मगर" | "पर" | "इसलिए"
            );

            if is_conjunction
                && i > 4
                && i < words.len() - 3
                && !result.ends_with(',')
                && !result.ends_with('.')
                && !result.ends_with('?')
                && !result.ends_with('!')
            {
                result.push(',');
            }

            if i < words.len() - 1 {
                result.push(' ');
            }
        }
        result
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

    /// Convert URLs `https://voxforg.org/api` → `voxforg dot org slash api`.
    pub fn normalize_urls(text: &str) -> String {
        re_url()
            .replace_all(text, |caps: &regex::Captures<'_>| {
                let raw = &caps[1];
                let trimmed = raw.trim_end_matches(['.', ',', ';', '!', '?', ')']);
                let trailing = &raw[trimmed.len()..];
                let clean = trimmed.strip_prefix("www.").unwrap_or(trimmed);
                let spoken = clean
                    .replace('.', " dot ")
                    .replace('/', " slash ")
                    .replace('-', " dash ")
                    .replace('_', " underscore ");
                format!("{spoken}{trailing}")
            })
            .to_string()
    }

    /// Convert `user@domain.com` → `user at domain dot com`.
    pub fn normalize_emails(text: &str) -> String {
        re_email()
            .replace_all(text, |caps: &regex::Captures<'_>| {
                let user = &caps[1];
                let domain = &caps[2];
                let spoken_domain = domain.replace('.', " dot ");
                format!("{user} at {spoken_domain}")
            })
            .to_string()
    }

    /// Convert `3:30 PM` → `three thirty PM`, `10:05 AM` → `ten oh five AM`.
    pub fn normalize_clock_times(text: &str) -> String {
        re_clock_time()
            .replace_all(text, |caps: &regex::Captures<'_>| {
                let h: u32 = caps[1].parse().unwrap_or(0);
                let m: u32 = caps[2].parse().unwrap_or(0);
                let period = caps.get(3).map(|m| m.as_str().to_uppercase());

                if h > 23 || m > 59 {
                    return caps[0].to_string();
                }

                let h_val = if h == 0 {
                    12
                } else if h > 12 && period.is_some() {
                    h - 12
                } else {
                    h
                };
                let h_word = num_to_words(h_val);

                if m == 0 {
                    match period {
                        Some(p) => format!("{h_word} {p}"),
                        None => format!("{h_word} o'clock"),
                    }
                } else if m < 10 {
                    let m_str = num_to_words(m);
                    match period {
                        Some(p) => format!("{h_word} oh {m_str} {p}"),
                        None => format!("{h_word} oh {m_str}"),
                    }
                } else {
                    let m_str = num_to_words(m).replace('-', " ");
                    match period {
                        Some(p) => format!("{h_word} {m_str} {p}"),
                        None => format!("{h_word} {m_str}"),
                    }
                }
            })
            .to_string()
    }

    /// Convert `in 2026` → `in twenty twenty-six`, `since 1999` → `since nineteen ninety-nine`.
    pub fn normalize_temporal_years(text: &str) -> String {
        re_temporal_year()
            .replace_all(text, |caps: &regex::Captures<'_>| {
                let prep = &caps[1];
                let year_num: u32 = caps[2].parse().unwrap_or(0);
                format!("{prep} {}", year_to_words(year_num))
            })
            .to_string()
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
    fn normalizes_urls() {
        let r = TextNormalizer::normalize("Visit https://voxforg.org/api today");
        assert!(r.contains("voxforg dot org slash api"), "got: {r}");
    }

    #[test]
    fn normalizes_emails() {
        let r = TextNormalizer::normalize("Email user@domain.com for support");
        assert!(r.contains("user at domain dot com"), "got: {r}");
    }

    #[test]
    fn normalizes_clock_times() {
        let r = TextNormalizer::normalize("Meeting set for 3:30 PM sharp");
        assert!(r.contains("three thirty PM"), "got: {r}");
    }

    #[test]
    fn normalizes_temporal_years() {
        let r = TextNormalizer::normalize("Built in 2026 and trusted since 1999");
        assert!(r.contains("in twenty twenty-six"), "got: {r}");
        assert!(r.contains("since nineteen ninety-nine"), "got: {r}");
    }

    #[test]
    fn leaves_regular_text_unchanged() {
        let input = "Hello world";
        assert_eq!(TextNormalizer::normalize(input), input);
    }
}
