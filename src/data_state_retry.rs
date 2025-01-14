use tracing::{error, warn};

use crate::{data_state::CanMakeProgress, Awaiting, DataState, ErrorBounds};
use std::fmt::Debug;
use std::ops::Range;
use std::time::{Duration, Instant};

/// Automatically retries with a delay on failure until attempts are exhausted
#[derive(Debug)]
pub struct DataStateRetry<T, E: ErrorBounds = anyhow::Error> {
    /// Number of attempts that the retries get reset to
    pub max_attempts: u8,
    /// The range of milliseconds to select a random value from to set the delay
    /// to retry
    pub retry_delay_millis: Range<u16>,
    attempts_left: u8,
    inner: DataState<T, E>, // Not public to ensure resets happen as they should
    next_allowed_attempt: Instant,
}

impl<T, E: ErrorBounds> DataStateRetry<T, E> {
    /// Creates a new instance of [DataStateRetry]
    pub fn new(max_attempts: u8, retry_delay_millis: Range<u16>) -> Self {
        Self {
            max_attempts,
            retry_delay_millis,
            ..Default::default()
        }
    }

    /// The number times left to retry before stopping trying
    pub fn attempts_left(&self) -> u8 {
        self.attempts_left
    }

    /// The instant that needs to be waited for before another attempt is
    /// allowed
    pub fn next_allowed_attempt(&self) -> Instant {
        self.next_allowed_attempt
    }

    /// Provides access to the inner [`DataState`]
    pub fn inner(&self) -> &DataState<T, E> {
        &self.inner
    }

    /// Consumes self and returns the unwrapped inner
    pub fn into_inner(self) -> DataState<T, E> {
        self.inner
    }

    /// Provides access to the stored data if available (returns Some if
    /// self.inner is `Data::Present(_)`)
    pub fn present(&self) -> Option<&T> {
        if let DataState::Present(data) = self.inner.as_ref() {
            Some(data)
        } else {
            None
        }
    }

    /// Provides mutable access to the stored data if available (returns Some if
    /// self.inner is `Data::Present(_)`)
    pub fn present_mut(&mut self) -> Option<&mut T> {
        if let DataState::Present(data) = self.inner.as_mut() {
            Some(data)
        } else {
            None
        }
    }

    #[cfg(feature = "egui")]
    /// Attempts to load the data and displays appropriate UI if applicable.
    ///
    /// Note see [`DataState::egui_get`] for more info.
    #[must_use]
    pub fn egui_get<F>(
        &mut self,
        ui: &mut egui::Ui,
        retry_msg: Option<&str>,
        fetch_fn: F,
    ) -> CanMakeProgress
    where
        F: FnOnce() -> Awaiting<T, E>,
    {
        match self.inner.as_ref() {
            DataState::None | DataState::AwaitingResponse(_) => {
                self.ui_spinner_with_attempt_count(ui);
                self.get(fetch_fn)
            }
            DataState::Present(_data) => {
                // Does nothing as data is already present
                CanMakeProgress::UnableToMakeProgress
            }
            DataState::Failed(e) => {
                ui.colored_label(
                    ui.visuals().error_fg_color,
                    format!("{} attempts exhausted. {e}", self.max_attempts),
                );
                if ui.button(retry_msg.unwrap_or("Restart Requests")).clicked() {
                    self.reset_attempts();
                    self.inner = DataState::default();
                }
                CanMakeProgress::AbleToMakeProgress
            }
        }
    }

    /// Attempts to load the data and returns if it is able to make progress.
    ///
    /// See [`DataState::get`] for more info.
    #[must_use]
    pub fn get<F>(&mut self, fetch_fn: F) -> CanMakeProgress
    where
        F: FnOnce() -> Awaiting<T, E>,
    {
        match self.inner.as_mut() {
            DataState::None => {
                // Going to make an attempt, set when the next attempt is allowed
                use rand::Rng;
                let wait_time_in_millis = rand::thread_rng()
                    .gen_range(self.retry_delay_millis.clone())
                    .into();
                self.next_allowed_attempt = Instant::now()
                    .checked_add(Duration::from_millis(wait_time_in_millis))
                    .expect("failed to get random delay, value was out of range");

                self.inner.get(fetch_fn)
            }
            DataState::AwaitingResponse(rx) => {
                if let Some(new_state) = DataState::await_data(rx) {
                    // TODO 4: Add some tests to ensure await_data work as this function assumes
                    self.inner = match new_state.as_ref() {
                        DataState::None => {
                            error!("Unexpected new state received of DataState::None");
                            unreachable!("Only expect Failed or Present variants to be returned but got None")
                        }
                        DataState::AwaitingResponse(_) => {
                            error!("Unexpected new state received of AwaitingResponse");
                            unreachable!("Only expect Failed or Present variants to be returned bug got AwaitingResponse")
                        }
                        DataState::Present(_) => {
                            // Data was successfully received
                            self.reset_attempts();
                            new_state
                        }
                        DataState::Failed(_) => new_state,
                    };
                }
                CanMakeProgress::AbleToMakeProgress
            }
            DataState::Present(_) => self.inner.get(fetch_fn),
            DataState::Failed(err_msg) => {
                if self.attempts_left == 0 {
                    self.inner.get(fetch_fn)
                } else {
                    let wait_duration_left = self
                        .next_allowed_attempt
                        .saturating_duration_since(Instant::now());
                    if wait_duration_left.is_zero() {
                        warn!(?err_msg, ?self.attempts_left, "retrying request");
                        self.attempts_left -= 1;
                        self.inner = DataState::None;
                    }
                    CanMakeProgress::AbleToMakeProgress
                }
            }
        }
    }

    /// Resets the attempts taken
    pub fn reset_attempts(&mut self) {
        self.attempts_left = self.max_attempts;
        self.next_allowed_attempt = Instant::now();
    }

    /// Clear stored data
    pub fn clear(&mut self) {
        self.inner = DataState::default();
    }

    /// Returns `true` if the internal data state is [`DataState::Present`].
    #[must_use]
    pub fn is_present(&self) -> bool {
        self.inner.is_present()
    }

    /// Returns `true` if the internal data state is [`DataState::None`].
    #[must_use]
    pub fn is_none(&self) -> bool {
        self.inner.is_none()
    }

    fn ui_spinner_with_attempt_count(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.separator();
            ui.label(format!("{} attempts left", self.attempts_left))
        });
    }
}

impl<T, E: ErrorBounds> Default for DataStateRetry<T, E> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            max_attempts: 3,
            retry_delay_millis: 1000..5000,
            attempts_left: 3,
            next_allowed_attempt: Instant::now(),
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
