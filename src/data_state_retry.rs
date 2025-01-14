use crate::{DataState, ErrorBounds};
use std::ops::Range;
use std::time::Instant;

/// Automatically retries with a delay on failure until attempts are exhausted
#[derive(Debug)]
pub struct DataStateRetry<T, E: ErrorBounds = anyhow::Error> {
    /// The wrapped [`DataState`]
    pub inner: DataState<T, E>,
    /// Number of attempts that the retries get reset to
    pub max_attempts: u8,
    /// The range of milliseconds to select a random value from to set the delay
    /// to retry
    pub retry_delay_ms: Range<u16>,
    attempts_left: u8,
    last_attempt: Option<Instant>,
}

impl<T, E: ErrorBounds> DataStateRetry<T, E> {
    /// Creates a new instance of [DataStateRetry]
    pub fn new(max_attempts: u8, retry_delay_ms: Range<u16>) -> Self {
        Self {
            max_attempts,
            retry_delay_ms,
            ..Default::default()
        }
    }

    /// The number times left to retry before stopping trying
    pub fn attempts_left(&self) -> u8 {
        self.attempts_left
    }

    /// If an attempt was made the instant that it happened at
    pub fn last_attempt(&self) -> Option<Instant> {
        self.last_attempt
    }
}

impl<T, E: ErrorBounds> Default for DataStateRetry<T, E> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            max_attempts: 3,
            retry_delay_ms: 1000..5000,
            attempts_left: 3,
            last_attempt: Default::default(),
        }
    }
}

impl<T, E: ErrorBounds> AsRef<DataStateRetry<T, E>> for DataStateRetry<T, E> {
    fn as_ref(&self) -> &DataStateRetry<T, E> {
        self
    }
}

impl<T, E: ErrorBounds> AsMut<DataStateRetry<T, E>> for DataStateRetry<T, E> {
    fn as_mut(&mut self) -> &mut DataStateRetry<T, E> {
        self
    }
}
