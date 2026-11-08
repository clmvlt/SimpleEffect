//! Reading whisper-cli's full JSON output (`-oj -ojf`): segments and their tokens with
//! probabilities, heuristic offsets and DTW times.
//!
//! Token texts are read as **raw bytes**: whisper's byte-level tokens may split a UTF-8 character
//! across two tokens, and whisper-cli writes each token's bytes as they are, so a token alone can
//! be invalid UTF-8. Words are decoded once their tokens are joined ([`crate::words`]).

use serde::{Deserialize, Deserializer, de};

/// One segment as whisper cut it.
#[derive(Debug, Clone, PartialEq)]
pub struct RawSegment {
    /// Start in milliseconds (already in the original audio's time, also with voice detection).
    pub from_ms: i64,
    /// End in milliseconds.
    pub to_ms: i64,
    /// Its tokens, special ones included.
    pub tokens: Vec<RawToken>,
}

/// One token.
#[derive(Debug, Clone, PartialEq)]
pub struct RawToken {
    /// Its bytes (UTF-8, possibly a partial character).
    pub bytes: Vec<u8>,
    /// Heuristic start in milliseconds (in the processed audio's time).
    pub from_ms: i64,
    /// Heuristic end in milliseconds.
    pub to_ms: i64,
    /// Probability of the token (0–1).
    pub probability: f64,
    /// DTW time in milliseconds when computed (the end of the token: whisper.cpp places the time
    /// at which the alignment moves to the next token), in the processed audio's time.
    pub dtw_ms: Option<i64>,
}

impl RawToken {
    /// Special tokens (`[_BEG_]`, `[_TT_150]`, `<|endoftext|>`…) carry no text.
    #[must_use]
    pub fn is_special(&self) -> bool {
        let bytes = self.bytes.as_slice();
        (bytes.starts_with(b"[_") && bytes.ends_with(b"]"))
            || (bytes.starts_with(b"<|") && bytes.ends_with(b"|>"))
    }
}

/// What whisper-cli wrote.
#[derive(Debug, Clone, PartialEq)]
pub struct WhisperOutput {
    /// The language it used (forced or detected).
    pub language: Option<String>,
    /// The segments, in order.
    pub segments: Vec<RawSegment>,
}

#[derive(Deserialize)]
struct Document {
    result: Option<ResultInfo>,
    #[serde(default)]
    transcription: Vec<Segment>,
}

#[derive(Deserialize)]
struct ResultInfo {
    language: Option<String>,
}

#[derive(Deserialize)]
struct Segment {
    offsets: Offsets,
    #[serde(default)]
    tokens: Vec<Token>,
}

#[derive(Deserialize)]
struct Offsets {
    from: i64,
    to: i64,
}

#[derive(Deserialize)]
struct Token {
    #[serde(deserialize_with = "raw_bytes")]
    text: Vec<u8>,
    offsets: Offsets,
    #[serde(default)]
    p: f64,
    t_dtw: Option<i64>,
}

/// Parses the JSON file whisper-cli wrote.
///
/// # Errors
/// A message for malformed JSON.
pub fn parse(json: &[u8]) -> Result<WhisperOutput, String> {
    let document: Document = serde_json::from_slice(json)
        .map_err(|error| format!("whisper.cpp wrote an unreadable transcript: {error}"))?;
    Ok(WhisperOutput {
        language: document.result.and_then(|result| result.language),
        segments: document
            .transcription
            .into_iter()
            .map(|segment| RawSegment {
                from_ms: segment.offsets.from,
                to_ms: segment.offsets.to,
                tokens: segment
                    .tokens
                    .into_iter()
                    .map(|token| RawToken {
                        bytes: token.text,
                        from_ms: token.offsets.from,
                        to_ms: token.offsets.to,
                        probability: token.p.clamp(0.0, 1.0),
                        // whisper.cpp counts DTW times in centiseconds; -1 when not computed.
                        dtw_ms: token.t_dtw.filter(|time| *time >= 0).map(|time| time * 10),
                    })
                    .collect(),
            })
            .collect(),
    })
}

/// A JSON string as its unescaped bytes, without UTF-8 validation.
fn raw_bytes<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
    struct Bytes;

    impl de::Visitor<'_> for Bytes {
        type Value = Vec<u8>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a string")
        }

        fn visit_bytes<E: de::Error>(self, bytes: &[u8]) -> Result<Vec<u8>, E> {
            Ok(bytes.to_vec())
        }

        fn visit_byte_buf<E: de::Error>(self, bytes: Vec<u8>) -> Result<Vec<u8>, E> {
            Ok(bytes)
        }

        fn visit_str<E: de::Error>(self, text: &str) -> Result<Vec<u8>, E> {
            Ok(text.as_bytes().to_vec())
        }
    }

    deserializer.deserialize_bytes(Bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_tokens_as_bytes_even_when_a_character_is_split() {
        // « é » (C3 A9) split over two tokens, as whisper can write it.
        let mut json = br#"{"result":{"language":"fr"},"transcription":[{"offsets":{"from":0,"to":900},"tokens":[{"text":"[_BEG_]","offsets":{"from":0,"to":0},"p":0.9,"t_dtw":-1},{"text":" vid"#.to_vec();
        json.push(0xC3);
        json.extend_from_slice(br#"","offsets":{"from":0,"to":300},"p":0.8,"t_dtw":25},{"text":""#);
        json.push(0xA9);
        json.extend_from_slice(
            br#"o \"ok\"","offsets":{"from":300,"to":600},"p":1.2,"t_dtw":51}]}]}"#,
        );
        let output = parse(&json).unwrap();
        assert_eq!(output.language.as_deref(), Some("fr"));
        let tokens = &output.segments[0].tokens;
        assert!(tokens[0].is_special() && tokens[0].dtw_ms.is_none());
        let mut word = tokens[1].bytes.clone();
        word.extend_from_slice(&tokens[2].bytes);
        assert_eq!(String::from_utf8(word).unwrap(), " vidéo \"ok\"");
        assert_eq!((tokens[1].dtw_ms, tokens[2].probability), (Some(250), 1.0));
    }

    #[test]
    fn refuses_unreadable_output() {
        assert!(parse(b"{ not json").unwrap_err().contains("unreadable"));
        assert_eq!(parse(b"{}").unwrap().segments, []);
    }
}
