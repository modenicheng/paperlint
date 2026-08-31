pub mod chars;
pub mod language;
pub mod sentence;
pub mod terminology;

pub use chars::SentenceStats;
pub use language::{detect_language, Language};
pub use sentence::{segment_sentences, Sentence};
