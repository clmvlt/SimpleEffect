//! [`CaptionItem`]: a subtitle on the timeline.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A subtitle shown for the item's duration.
///
/// TODO(prompt-23): build captions from timed transcript words, with a caption style reference
/// and word-level highlighting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CaptionItem {
    /// The subtitle text.
    pub text: String,
}
