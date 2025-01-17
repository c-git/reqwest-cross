use tracing::warn;

use crate::{data_state::CanMakeProgress, Awaiting, DataState, ErrorBounds};
use std::fmt::Debug;
use std::ops::Range;

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
    next_allowed_attempt: u128,
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

    /// The number of millis after the epoch that an attempt is allowed
    pub fn next_allowed_attempt(&self) -> u128 {
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
    pub fn egui_start_or_poll<F, R>(
        &mut self,
        ui: &mut egui::Ui,
        retry_msg: Option<&str>,
        fetch_fn: F,
    ) -> CanMakeProgress
    where
        F: FnOnce() -> R,
        R: Into<Awaiting<T, E>>,
    {
        match self.inner.as_ref() {
            DataState::None | DataState::AwaitingResponse(_) => {
                self.ui_spinner_with_attempt_count(ui);
                self.start_or_poll(fetch_fn)
            }
            DataState::Present(_data) => {
                // Does nothing as data is already present
                CanMakeProgress::UnableToMakeProgress
            }
            DataState::Failed(e) => {
                if self.attempts_left == 0 {
                    ui.colored_label(
                        ui.visuals().error_fg_color,
                        format!("No attempts left from {}. {e}", self.max_attempts),
                    );
                    if ui.button(retry_msg.unwrap_or("Restart Requests")).clicked() {
                        self.reset_attempts();
                        self.inner = DataState::default();
                    }
                } else {
                    let wait_left = wait_before_next_attempt(self.next_allowed_attempt);
                    ui.colored_label(
                        ui.visuals().error_fg_color,
                        format!(
                            "{} attempt(s) left. {} seconds before retry. {e}",
                            self.attempts_left,
                            wait_left / 1000
                        ),
                    );
                    let can_make_progress = self.start_or_poll(fetch_fn);
                    debug_assert!(
                        can_make_progress.is_able_to_make_progress(),
                        "This should be able to make progress"
                    );
                    if ui.button("Stop Trying").clicked() {
                        self.attempts_left = 0;
                    }
                }
                CanMakeProgress::AbleToMakeProgress
            }
        }
    }

    /// Attempts to load the data and returns if it is able to make progress.
    #[must_use]
    pub fn start_or_poll<F, R>(&mut self, fetch_fn: F) -> CanMakeProgress
    where
        F: FnOnce() -> R,
        R: Into<Awaiting<T, E>>,
    {
        match self.inner.as_mut() {
            DataState::None => {
                // Going to make an attempt, set when the next attempt is allowed
                use rand::Rng as _;
                let wait_time_in_millis =
                    rand::thread_rng().gen_range(self.retry_delay_millis.clone());
                self.next_allowed_attempt = millis_since_epoch() + wait_time_in_millis as u128;

                self.inner.start_request(fetch_fn)
            }
            DataState::AwaitingResponse(_) => {
                if self.inner.poll().is_present() {
                    // Data was successfully received because before it was Awaiting
                    self.reset_attempts();
                }
                CanMakeProgress::AbleToMakeProgress
            }
            DataState::Present(_) => CanMakeProgress::UnableToMakeProgress,
            DataState::Failed(err_msg) => {
                if self.attempts_left == 0 {
                    CanMakeProgress::UnableToMakeProgress
                } else {
                    let wait_left = wait_before_next_attempt(self.next_allowed_attempt);
                    if wait_left == 0 {
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
        self.next_allowed_attempt = millis_since_epoch();
    }

    /// Clear stored data
    pub fn clear(&mut self) {
        self.inner = DataState::default();
        self.reset_attempts();
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

    #[cfg(feature = "egui")]
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
            next_allowed_attempt: millis_since_epoch(),
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

/// The duration before the next attempt will be made
fn wait_before_next_attempt(next_allowed_attempt: u128) -> u128 {
    next_allowed_attempt.saturating_sub(millis_since_epoch())
}

fn millis_since_epoch() -> u128 {
    web_time::SystemTime::UNIX_EPOCH
        .elapsed()
        .expect("expected date on system to be after the epoch")
        .as_millis()
}

// TODO 4: Use mocking to add tests ensuring retires are executed
