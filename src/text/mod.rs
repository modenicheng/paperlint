pub mod chars;
pub mod language;
pub mod sentence;
pub mod terminology;

pub use chars::SentenceStats;
pub use language::{Language, detect_language};
pub use sentence::{Sentence, segment_sentences};
