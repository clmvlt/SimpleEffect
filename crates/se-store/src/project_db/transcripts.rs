//! Transcripts (schema v5): one `transcripts` row per asset (content, language, model, backend),
//! its words in `transcript_words`, its segments in `transcript_segments`, and the segments' text
//! in the FTS5 index `transcript_search` (`unicode61`, accents removed). Saving replaces the
//! asset's previous transcript in one SQLite transaction.

use rusqlite::{Connection, OptionalExtension, params};
use se_app::ports::{StorageError, StoredTranscript, TranscriptHit};
use se_domain::{AssetId, Flicks, Transcript, TranscriptSegment, TranscriptWord};

use super::ProjectDatabase;
use crate::io_errors::sqlite_error;

/// Version of the rows of a transcript. Rows of a newer version are ignored (the asset reads as
/// not transcribed); bump it when their meaning changes.
const TRANSCRIPT_FORMAT_VERSION: u32 = 1;

impl ProjectDatabase {
    /// Saves `stored`, replacing the asset's previous transcript (rows and index).
    ///
    /// # Errors
    /// IO errors (nothing changes then).
    pub fn save_transcript(&self, stored: &StoredTranscript) -> Result<(), StorageError> {
        let error =
            |error: rusqlite::Error| sqlite_error("save a transcript in", &self.path, &error);
        let transaction = self.connection.unchecked_transaction().map_err(error)?;
        write_transcript(&transaction, stored).map_err(error)?;
        transaction.commit().map_err(error)
    }

    /// The transcript of `asset_id`; `None` when absent or written by a newer version.
    ///
    /// # Errors
    /// [`StorageError::Corrupted`] for unreadable rows; IO errors.
    pub fn load_transcript(
        &self,
        asset_id: AssetId,
    ) -> Result<Option<StoredTranscript>, StorageError> {
        let error =
            |error: rusqlite::Error| sqlite_error("read a transcript of", &self.path, &error);
        let id = asset_id.to_string();
        let header = self
            .connection
            .query_row(
                "SELECT content_key, language, model, backend, created_at, format_version
                 FROM transcripts WHERE asset_id = ?1",
                [&id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, i64>(4)?,
                        row.get::<_, u32>(5)?,
                    ))
                },
            )
            .optional()
            .map_err(error)?;
        let Some((content_key, language, model, backend, created_at, version)) = header else {
            return Ok(None);
        };
        if version > TRANSCRIPT_FORMAT_VERSION {
            return Ok(None);
        }
        let words = self.load_words(&id).map_err(error)?;
        let segments = self.load_segments(&id).map_err(error)?;
        let transcript = Transcript {
            language,
            model,
            words: words.ok_or_else(|| self.corrupted(asset_id, "a word time is out of range"))?,
            segments: segments
                .ok_or_else(|| self.corrupted(asset_id, "a segment time is out of range"))?,
        };
        if let Some(problem) = transcript.problems().first() {
            return Err(self.corrupted(asset_id, problem));
        }
        Ok(Some(StoredTranscript {
            asset_id,
            content_key,
            created_at,
            backend,
            transcript,
        }))
    }

    /// Segments matching every word of `query` (the last as a prefix), best first.
    ///
    /// # Errors
    /// IO errors.
    pub fn search_transcripts(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<TranscriptHit>, StorageError> {
        let Some(expression) = match_expression(query) else {
            return Ok(Vec::new());
        };
        let error =
            |error: rusqlite::Error| sqlite_error("search the transcripts of", &self.path, &error);
        let mut statement = self
            .connection
            .prepare(
                "SELECT s.asset_id, s.segment, g.start_flicks, g.end_flicks, s.text
                 FROM transcript_search s
                 JOIN transcript_segments g ON g.asset_id = s.asset_id AND g.segment = s.segment
                 WHERE transcript_search MATCH ?1 ORDER BY bm25(transcript_search) LIMIT ?2",
            )
            .map_err(error)?;
        let limit = i64::try_from(limit).unwrap_or(i64::MAX);
        let rows = statement
            .query_map(params![expression, limit], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, u32>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })
            .map_err(error)?;
        let mut hits = Vec::new();
        for row in rows {
            let (asset, segment, start, end, text) = row.map_err(error)?;
            let (Ok(asset_id), Ok(start), Ok(end)) =
                (AssetId::parse(&asset), Flicks::new(start), Flicks::new(end))
            else {
                continue;
            };
            hits.push(TranscriptHit {
                asset_id,
                segment,
                start,
                end,
                text,
            });
        }
        Ok(hits)
    }

    fn load_words(&self, asset_id: &str) -> rusqlite::Result<Option<Vec<TranscriptWord>>> {
        let mut statement = self.connection.prepare(
            "SELECT text, start_flicks, end_flicks, confidence, segment FROM transcript_words
             WHERE asset_id = ?1 ORDER BY word",
        )?;
        let rows = statement.query_map([asset_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, u32>(4)?,
            ))
        })?;
        let mut words = Vec::new();
        for row in rows {
            let (text, start, end, confidence, segment) = row?;
            let (Ok(start), Ok(end)) = (Flicks::new(start), Flicks::new(end)) else {
                return Ok(None);
            };
            words.push(TranscriptWord {
                text,
                start,
                end,
                confidence,
                segment,
            });
        }
        Ok(Some(words))
    }

    fn load_segments(&self, asset_id: &str) -> rusqlite::Result<Option<Vec<TranscriptSegment>>> {
        let mut statement = self.connection.prepare(
            "SELECT start_flicks, end_flicks FROM transcript_segments
             WHERE asset_id = ?1 ORDER BY segment",
        )?;
        let rows = statement.query_map([asset_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })?;
        let mut segments = Vec::new();
        for row in rows {
            let (start, end) = row?;
            let (Ok(start), Ok(end)) = (Flicks::new(start), Flicks::new(end)) else {
                return Ok(None);
            };
            segments.push(TranscriptSegment { start, end });
        }
        Ok(Some(segments))
    }

    fn corrupted(&self, asset_id: AssetId, problem: &str) -> StorageError {
        StorageError::Corrupted {
            path: self.path.clone(),
            reason: format!("the transcript of asset {asset_id} is unreadable: {problem}"),
        }
    }
}

/// Replaces the asset's rows (cascade) and index entries with `stored`.
fn write_transcript(connection: &Connection, stored: &StoredTranscript) -> rusqlite::Result<()> {
    let id = stored.asset_id.to_string();
    connection.execute("DELETE FROM transcript_search WHERE asset_id = ?1", [&id])?;
    connection.execute("DELETE FROM transcripts WHERE asset_id = ?1", [&id])?;
    let transcript = &stored.transcript;
    connection.execute(
        "INSERT INTO transcripts (asset_id, content_key, language, model, backend, created_at,
         format_version) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            id,
            stored.content_key,
            transcript.language,
            transcript.model,
            stored.backend,
            stored.created_at,
            TRANSCRIPT_FORMAT_VERSION
        ],
    )?;
    let mut segment_row = connection.prepare(
        "INSERT INTO transcript_segments (asset_id, segment, start_flicks, end_flicks, text)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;
    let mut search_row = connection
        .prepare("INSERT INTO transcript_search (asset_id, segment, text) VALUES (?1, ?2, ?3)")?;
    for (index, segment) in transcript.segments.iter().enumerate() {
        let index = u32::try_from(index).unwrap_or(u32::MAX);
        let text = transcript.segment_text(index);
        segment_row.execute(params![
            id,
            index,
            segment.start.get(),
            segment.end.get(),
            text
        ])?;
        search_row.execute(params![id, index, text])?;
    }
    let mut word_row = connection.prepare(
        "INSERT INTO transcript_words (asset_id, word, text, start_flicks, end_flicks, confidence,
         segment) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )?;
    for (index, word) in transcript.words.iter().enumerate() {
        word_row.execute(params![
            id,
            i64::try_from(index).unwrap_or(i64::MAX),
            word.text,
            word.start.get(),
            word.end.get(),
            word.confidence,
            word.segment
        ])?;
    }
    Ok(())
}

/// The FTS5 expression of a user's query: every word quoted (so punctuation and operators are
/// plain text), the last one as a prefix; `None` without any word.
fn match_expression(query: &str) -> Option<String> {
    let words: Vec<String> = query
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(|word| format!("\"{word}\""))
        .collect();
    let (last, others) = words.split_last()?;
    let mut expression = others.join(" ");
    if !expression.is_empty() {
        expression.push(' ');
    }
    expression.push_str(last);
    expression.push('*');
    Some(expression)
}

#[cfg(test)]
mod tests {
    use super::match_expression;

    #[test]
    fn queries_become_quoted_words_with_a_final_prefix() {
        assert_eq!(
            match_expression("abonnez-vous").as_deref(),
            Some("\"abonnez\" \"vous\"*")
        );
        assert_eq!(
            match_expression("l'IA OR NOT").as_deref(),
            Some("\"l\" \"IA\" \"OR\" \"NOT\"*")
        );
        assert_eq!(match_expression(" ?! "), None);
    }
}
