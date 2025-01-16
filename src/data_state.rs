//! Helpers for handling pending data.

use anyhow::anyhow;
use futures::channel::oneshot;
use std::fmt::{Debug, Display};
use thiserror::Error;
use tracing::{error, warn};

/// Provides a common way to specify the bounds errors are expected to meet
pub trait ErrorBounds: Display + Send + Sync + 'static + Debug {}
impl<T: Display + Send + Sync + 'static + Debug> ErrorBounds for T {}

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
impl<T, E: ErrorBounds> From<oneshot::Receiver<Result<T, E>>> for Awaiting<T, E> {
    fn from(value: oneshot::Receiver<Result<T, E>>) -> Self {
        Self(value)
    }
}

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
    /// Calls [Self::start_request] and adds a spinner if progress can be made
    #[must_use]
    pub fn egui_start_request<F, R>(&mut self, ui: &mut egui::Ui, fetch_fn: F) -> CanMakeProgress
    where
        F: FnOnce() -> R,
        R: Into<Awaiting<T, E>>,
    {
        let result = self.start_request(fetch_fn);
        if result.is_able_to_make_progress() {
            ui.spinner();
        }
        result
    }

    /// Starts a new request. Only intended to be on [Self::None] and if state
    /// is any other value it returns [CanMakeProgress::UnableToMakeProgress]
    #[must_use]
    pub fn start_request<F, R>(&mut self, fetch_fn: F) -> CanMakeProgress
    where
        F: FnOnce() -> R,
        R: Into<Awaiting<T, E>>,
    {
        if self.is_none() {
            *self = DataState::AwaitingResponse(fetch_fn().into());
            CanMakeProgress::AbleToMakeProgress
        } else {
            debug_assert!(
                false,
                "No known good reason this path should be hit other than logic error"
            );
            CanMakeProgress::UnableToMakeProgress
        }
    }

    /// Convenience method that will try to make progress if in
    /// [Self::AwaitingResponse] and does nothing otherwise. Returns a reference
    /// to self for chaining
    pub fn poll(&mut self) -> &mut Self {
        if let DataState::AwaitingResponse(rx) = self {
            if let Some(new_state) = Self::await_data(rx) {
                *self = new_state;
            }
        }
        self
    }

    #[cfg(feature = "egui")]
    /// Meant to be a simple method to just provide the data if it's ready or
    /// help with UI and polling to get it ready if it's not.
    ///
    /// WARNING: Does nothing if `self` is [Self::None]
    ///
    /// If a `error_btn_text` is provided then it overrides the default
    #[must_use]
    pub fn egui_poll_mut(
        &mut self,
        ui: &mut egui::Ui,
        error_btn_text: Option<&str>,
    ) -> Option<&mut T> {
        match self {
            DataState::None => {}
            DataState::AwaitingResponse(_) => {
                ui.spinner();
                self.poll();
            }
            DataState::Present(data) => {
                return Some(data);
            }
            DataState::Failed(e) => {
                ui.colored_label(ui.visuals().error_fg_color, e.to_string());
                if ui
                    .button(error_btn_text.unwrap_or("Clear Error Status"))
                    .clicked()
                {
                    *self = DataState::default();
                }
            }
        }
        None
    }

    #[cfg(feature = "egui")]
    /// Wraps [Self::egui_poll_mut] and returns an immutable reference
    #[must_use]
    pub fn egui_poll(&mut self, ui: &mut egui::Ui, error_btn_text: Option<&str>) -> Option<&T> {
        self.egui_poll_mut(ui, error_btn_text).map(|x| &*x)
    }

    /// Checks to see if the data is ready and if it is returns a new [`Self`]
    /// otherwise None.
    pub fn await_data(rx: &mut Awaiting<T, E>) -> Option<Self> {
        Some(match rx.0.try_recv() {
            Ok(recv_opt) => match recv_opt {
                Some(outcome_result) => match outcome_result {
                    Ok(data) => DataState::Present(data),
                    Err(err_msg) => {
                        warn!(?err_msg, "Error response received instead of the data");
                        DataState::Failed(DataStateError::ErrorResponse(err_msg))
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

    /// Returns a reference to the inner data if available otherwise None.
    ///
    /// NOTE: This function does not poll to get the data ready if the state is
    /// still awaiting
    pub fn present(&self) -> Option<&T> {
        if let Self::Present(data) = self {
            Some(data)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the inner data if available otherwise
    /// None
    ///
    /// NOTE: This function does not poll to get the data ready if the state is
    /// still awaiting
    pub fn present_mut(&mut self) -> Option<&mut T> {
        if let Self::Present(data) = self {
            Some(data)
        } else {
            None
        }
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
