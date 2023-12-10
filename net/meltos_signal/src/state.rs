use auto_delegate::Delegate;
use axum::extract::FromRef;

use crate::api::webrtc::SocketChannels;
use crate::session::SessionIo;

#[derive(Default, Clone)]
pub struct AppState<S>
    where
        S: SessionIo + Clone,
{
    session_io: SessionIoState<S>,
    channels: SocketChannels,
}


impl<S> FromRef<AppState<S>> for SessionIoState<S>
    where
        S: SessionIo + Clone,
{
    fn from_ref(input: &AppState<S>) -> Self {
        input.session_io.clone()
    }
}


impl<S> FromRef<AppState<S>> for SocketChannels
    where
        S: SessionIo + Clone,
{
    fn from_ref(input: &AppState<S>) -> Self {
        input.channels.clone()
    }
}


#[derive(Clone, Default, Delegate)]
#[to(SessionIo)]
pub struct SessionIoState<S: SessionIo + Clone>(S);

