//! Helpers for handling pending data.

use std::fmt::{Debug, Display};

use anyhow::anyhow;
use futures::channel::oneshot;
use thiserror::Error;
use tracing::{error, warn};

/// Provides a common way to specify the bounds errors are expected to meet
pub trait ErrorBounds: Display + Send + Sync + 'static {}
impl<T: Display + Send + Sync + 'static> ErrorBounds for T {}

#[derive(Error, Debug)]
/// Represents the types of errors that can occur while using [DataState]
pub enum DataStateError<E: ErrorBounds> {
    /// Sender was dropped, request cancelled
    #[error("Request sender was dropped")]
    SenderDropped(oneshot::Canceled),

    /// The response received from the request was an error
    #[error("Response received was an error: {0}")]
    ErrorResponse(E),

    /// This variant is supplied for use by application code
    #[error(transparent)]
    FromE(E),
}

#[derive(Debug)]
/// Provides a way to ensure the calling code knows if it is calling a function
/// that cannot do anything useful anymore
pub enum CanMakeProgress {
    AbleToMakeProgress,
    UnableToMakeProgress,
}

/// Used to represent data that is pending being available
#[derive(Debug)]
pub struct Awaiting<T, E: ErrorBounds>(pub oneshot::Receiver<Result<T, E>>);

/// Used to store a type that is not always available and we need to keep
/// polling it to get it ready
#[derive(Debug, Default)]
pub enum DataState<T, E: ErrorBounds = anyhow::Error> {
    /// Represent no data present and not pending
    #[default]
    None,
    /// Represents data has been requested and awaiting it being available
    AwaitingResponse(Awaiting<T, E>), // TODO 4: Add support for a timeout on waiting
    /// Represents data that is available for use
    Present(T),
    /// Represents an error that Occurred
    Failed(DataStateError<E>),
}

impl<T, E: ErrorBounds> DataState<T, E> {
    #[cfg(feature = "egui")]
    /// Attempts to load the data and displays appropriate UI if applicable.
    /// Some branches lead to no UI being displayed, in particular when the data
    /// or an error is received (On the expectation it will show next frame).
    /// When in an error state the error messages will show as applicable.
    ///
    /// If a `retry_msg` is provided then it overrides the default
    ///
    /// Note see [`Self::get`] for more info.
    pub fn egui_get<F>(&mut self, ui: &mut egui::Ui, retry_msg: Option<&str>, fetch_fn: F)
    where
        F: FnOnce() -> Awaiting<T, E>,
    {
        match self {
            DataState::None => {
                ui.spinner();
                self.get(fetch_fn);
            }
            DataState::AwaitingResponse(rx) => {
                if let Some(new_state) = Self::await_data(rx) {
                    *self = new_state;
                } else {
                    ui.spinner();
                }
            }
            DataState::Present(_data) => {
                // Does nothing as data is already present
            }
            DataState::Failed(e) => {
                ui.colored_label(ui.visuals().error_fg_color, e.to_string());
                if ui.button(retry_msg.unwrap_or("Retry Request")).clicked() {
                    *self = DataState::default();
                }
            }
        }
    }

    /// Attempts to load the data and returns if it is able to make progress.
    ///
    /// Note: F needs to return `AwaitingType<T>` and not T because it needs to
    /// be able to be pending if T is not ready.
    #[must_use]
    pub fn get<F>(&mut self, fetch_fn: F) -> CanMakeProgress
    where
        F: FnOnce() -> Awaiting<T, E>,
    {
        match self {
            DataState::None => {
                let rx = fetch_fn();
                *self = DataState::AwaitingResponse(rx);
                CanMakeProgress::AbleToMakeProgress
            }
            DataState::AwaitingResponse(rx) => {
                if let Some(new_state) = Self::await_data(rx) {
                    *self = new_state;
                }
                CanMakeProgress::AbleToMakeProgress
            }
            DataState::Present(_data) => {
                // Does nothing data is already present
                CanMakeProgress::UnableToMakeProgress
            }
            DataState::Failed(_e) => {
                // Have no way to let the user know there is an error nothing we
                // can do here
                CanMakeProgress::UnableToMakeProgress
            }
        }
    }

    /// Checks to see if the data is ready and if it is returns a new [`Self`]
    /// otherwise None
    pub fn await_data(rx: &mut Awaiting<T, E>) -> Option<Self> {
        Some(match rx.0.try_recv() {
            Ok(recv_opt) => match recv_opt {
                Some(outcome_result) => match outcome_result {
                    Ok(data) => DataState::Present(data),
                    Err(e) => {
                        warn!(
                            err_msg = e.to_string(),
                            "Error response received instead of the data"
                        );
                        DataState::Failed(DataStateError::ErrorResponse(e))
                    }
                },
                None => {
                    return None;
                }
            },
            Err(e) => {
                error!("Error receiving on channel. Sender dropped.");
                DataState::Failed(DataStateError::SenderDropped(e))
            }
        })
    }

    /// Returns `true` if the data state is [`Present`].
    ///
    /// [`Present`]: DataState::Present
    #[must_use]
    pub fn is_present(&self) -> bool {
        matches!(self, Self::Present(..))
    }

    /// Returns `true` if the data state is [`None`].
    ///
    /// [`None`]: DataState::None
    #[must_use]
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

impl<T, E: ErrorBounds> AsRef<DataState<T, E>> for DataState<T, E> {
    fn as_ref(&self) -> &DataState<T, E> {
        self
    }
}

impl<T, E: ErrorBounds> AsMut<DataState<T, E>> for DataState<T, E> {
    fn as_mut(&mut self) -> &mut DataState<T, E> {
        self
    }
}

impl<E: ErrorBounds> From<E> for DataStateError<E> {
    fn from(value: E) -> Self {
        Self::FromE(value)
    }
}

impl From<&str> for DataStateError<anyhow::Error> {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

impl From<String> for DataStateError<anyhow::Error> {
    fn from(value: String) -> Self {
        anyhow!(value).into()
    }
}

impl CanMakeProgress {
    /// Returns `true` if the can make progress is [`AbleToMakeProgress`].
    ///
    /// [`AbleToMakeProgress`]: CanMakeProgress::AbleToMakeProgress
    #[must_use]
    pub fn is_able_to_make_progress(&self) -> bool {
        matches!(self, Self::AbleToMakeProgress)
    }

    /// Returns `true` if the can make progress is [`UnableToMakeProgress`].
    ///
    /// [`UnableToMakeProgress`]: CanMakeProgress::UnableToMakeProgress
    #[must_use]
    pub fn is_unable_to_make_progress(&self) -> bool {
        matches!(self, Self::UnableToMakeProgress)
    }
}
