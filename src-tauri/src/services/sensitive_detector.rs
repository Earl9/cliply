#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensitivityRisk {
    None,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitivityDetection {
    pub score: u8,
    pub risk: SensitivityRisk,
    pub reason: Option<&'static str>,
}

impl SensitivityDetection {
    fn none() -> Self {
        Self {
            score: 0,
            risk: SensitivityRisk::None,
            reason: None,
        }
    }

    fn medium(score: u8, reason: &'static str) -> Self {
        Self {
            score,
            risk: SensitivityRisk::Medium,
            reason: Some(reason),
        }
    }

    fn high(reason: &'static str) -> Self {
        Self {
            score: 95,
            risk: SensitivityRisk::High,
            reason: Some(reason),
        }
    }
}

pub fn analyze(value: &str) -> SensitivityDetection {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return SensitivityDetection::none();
    }

    let lower = trimmed.to_lowercase();

    if looks_like_private_key(&lower) {
        return SensitivityDetection::high("private_key");
    }

    if looks_like_api_secret(trimmed, &lower) {
        return SensitivityDetection::high("api_secret");
    }

    if looks_like_seed_phrase(trimmed, &lower) {
        return SensitivityDetection::high("seed_phrase");
    }

    if looks_like_government_id(trimmed) {
        return SensitivityDetection::high("government_id");
    }

    if looks_like_payment_card(trimmed) {
        return SensitivityDetection::high("payment_card");
    }

    if looks_like_one_time_code(trimmed, &lower) {
        return SensitivityDetection::medium(60, "one_time_code");
    }

    SensitivityDetection::none()
}

pub fn looks_sensitive(value: &str) -> bool {
    analyze(value).risk == SensitivityRisk::High
}

fn looks_like_private_key(lower: &str) -> bool {
    [
        "-----begin private key-----",
        "-----begin rsa private key-----",
        "-----begin dsa private key-----",
        "-----begin ec private key-----",
        "-----begin openssh private key-----",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn looks_like_api_secret(value: &str, lower: &str) -> bool {
    lower.contains("sk-") && has_token_tail(value, "sk-", 16)
        || lower.contains("bearer ") && has_long_secret_like_run(value, 20)
        || contains_secret_assignment(lower) && has_long_secret_like_run(value, 8)
}

fn contains_secret_assignment(lower: &str) -> bool {
    [
        "api_key",
        "apikey",
        "api-key",
        "access_token",
        "auth_token",
        "refresh_token",
        "secret",
        "client_secret",
        "password",
        "passwd",
        "pwd",
    ]
    .iter()
    .any(|marker| {
        lower.find(marker).is_some_and(|index| {
            let after_marker = lower[index + marker.len()..]
                .chars()
                .take(16)
                .collect::<String>();
            after_marker.contains('=') || after_marker.contains(':')
        })
    })
}

fn has_token_tail(value: &str, marker: &str, min_tail_len: usize) -> bool {
    let lower = value.to_lowercase();
    let Some(index) = lower.find(marker) else {
        return false;
    };

    lower[index + marker.len()..]
        .chars()
        .take_while(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
        })
        .count()
        >= min_tail_len
}

fn has_long_secret_like_run(value: &str, min_len: usize) -> bool {
    value
        .split(|character: char| {
            !(character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.'))
        })
        .any(|part| {
            part.len() >= min_len && part.chars().any(|character| character.is_ascii_digit())
        })
}

fn looks_like_one_time_code(value: &str, lower: &str) -> bool {
    let digits = value
        .chars()
        .filter(|character| character.is_ascii_digit())
        .count();
    let exactly_six_digits =
        value.chars().all(|character| character.is_ascii_digit()) && digits == 6;
    let labeled_code = [
        "otp",
        "2fa",
        "mfa",
        "验证码",
        "verification code",
        "one-time code",
    ]
    .iter()
    .any(|marker| lower.contains(marker));

    exactly_six_digits || (labeled_code && (4..=8).contains(&digits))
}

fn looks_like_payment_card(value: &str) -> bool {
    let digits = value
        .chars()
        .filter(|character| character.is_ascii_digit())
        .collect::<String>();

    (13..=19).contains(&digits.len()) && passes_luhn(&digits)
}

fn passes_luhn(digits: &str) -> bool {
    let mut sum = 0u32;
    let mut double = false;

    for character in digits.chars().rev() {
        let Some(mut digit) = character.to_digit(10) else {
            return false;
        };

        if double {
            digit *= 2;
            if digit > 9 {
                digit -= 9;
            }
        }

        sum += digit;
        double = !double;
    }

    sum > 0 && sum % 10 == 0
}

fn looks_like_government_id(value: &str) -> bool {
    let compact = value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect::<String>();

    if compact.len() != 18 {
        return false;
    }

    compact[..17]
        .chars()
        .all(|character| character.is_ascii_digit())
        && compact
            .chars()
            .last()
            .is_some_and(|character| character.is_ascii_digit() || matches!(character, 'x' | 'X'))
}

fn looks_like_seed_phrase(value: &str, lower: &str) -> bool {
    let words = seed_candidate_words(value);
    if lower.contains("seed phrase") || lower.contains("mnemonic") || lower.contains("助记词") {
        return (12..=24).contains(&words.len());
    }

    if !matches!(words.len(), 12 | 15 | 18 | 21 | 24) {
        return false;
    }

    let unique_count = unique_word_count(&words);
    let common_prose_count = words
        .iter()
        .filter(|word| is_common_prose_word(word))
        .count();
    let anchor_count = words
        .iter()
        .filter(|word| is_seed_anchor_word(word))
        .count();

    unique_count >= words.len() * 3 / 4
        && common_prose_count <= words.len() / 4
        && anchor_count >= 2
}

fn seed_candidate_words(value: &str) -> Vec<String> {
    value
        .split_whitespace()
        .map(|word| word.trim_matches(|character: char| !character.is_ascii_alphabetic()))
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .filter(|word| word.len() >= 3 && word.len() <= 10)
        .collect()
}

fn unique_word_count(words: &[String]) -> usize {
    let mut unique = Vec::<&str>::new();
    for word in words {
        if !unique.contains(&word.as_str()) {
            unique.push(word);
        }
    }

    unique.len()
}

fn is_seed_anchor_word(word: &str) -> bool {
    matches!(
        word,
        "abandon"
            | "ability"
            | "absent"
            | "absorb"
            | "abstract"
            | "absurd"
            | "access"
            | "accident"
            | "account"
            | "acid"
            | "acoustic"
            | "acquire"
            | "adult"
            | "advance"
            | "aerobic"
            | "affair"
            | "amount"
            | "anchor"
            | "answer"
            | "apology"
            | "arrange"
            | "auction"
            | "balance"
            | "basic"
            | "bonus"
            | "bright"
            | "budget"
            | "bundle"
    )
}

fn is_common_prose_word(word: &str) -> bool {
    matches!(
        word,
        "the"
            | "and"
            | "for"
            | "with"
            | "this"
            | "that"
            | "from"
            | "have"
            | "please"
            | "after"
            | "before"
            | "your"
            | "will"
            | "should"
            | "could"
            | "would"
            | "when"
            | "where"
            | "what"
            | "which"
    )
}

#[cfg(test)]
mod tests {
    use super::{analyze, looks_sensitive, SensitivityRisk};

    #[test]
    fn blocks_private_keys() {
        let detection = analyze("-----BEGIN PRIVATE KEY-----\nabc\n-----END PRIVATE KEY-----");

        assert_eq!(detection.risk, SensitivityRisk::High);
        assert!(looks_sensitive("-----BEGIN OPENSSH PRIVATE KEY-----"));
    }

    #[test]
    fn blocks_api_tokens_without_flagging_plain_chat() {
        let detection = analyze("OPENAI_API_KEY=sk-abcdefghijklmnopqrstuvwxyz1234567890");

        assert_eq!(detection.risk, SensitivityRisk::High);
        assert_eq!(
            analyze("Please reset the password after the demo.").risk,
            SensitivityRisk::None
        );
    }

    #[test]
    fn detects_medium_one_time_codes() {
        let detection = analyze("123456");

        assert_eq!(detection.risk, SensitivityRisk::Medium);
        assert!(detection.score >= 50);
    }

    #[test]
    fn does_not_flag_regular_urls() {
        assert_eq!(
            analyze("https://github.com/tauri-apps/tauri").risk,
            SensitivityRisk::None
        );
    }

    #[test]
    fn detects_seed_phrase_shape() {
        let detection = analyze(
            "abandon ability absent absorb abstract absurd access accident account acid acoustic acquire",
        );

        assert_eq!(detection.risk, SensitivityRisk::High);
    }

    #[test]
    fn does_not_flag_plain_sentences_as_seed_phrases() {
        let detection = analyze(
            "please review this design draft after lunch and send your notes before the meeting",
        );

        assert_eq!(detection.risk, SensitivityRisk::None);
    }
}
