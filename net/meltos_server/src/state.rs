use auto_delegate::Delegate;
use axum::body::Body;
use axum::extract::FromRef;
use axum::http::StatusCode;
use axum::response::Response;

use meltos::user::{UserId, UserToken};
use meltos_backend::user::SessionIo;

use crate::room::Rooms;

#[derive(Clone, Default)]
pub struct AppState<Session> {
    rooms: Rooms,
    session: SessionState<Session>,
}


impl<Session> AppState<Session>
where
    Session: SessionIo + Clone,
{
    pub fn new(session: Session) -> AppState<Session> {
        Self {
            rooms: Rooms::default(),
            session: SessionState(session),
        }
    }
}


#[derive(Delegate, Clone, Default, Debug)]
#[to(UserSessionIo)]
pub struct SessionState<Session>(Session);

impl<Session> SessionState<Session>
where
    Session: SessionIo,
{
    pub async fn try_fetch_user_id(
        &self,
        user_token: UserToken,
    ) -> std::result::Result<UserId, Response<Body>> {
        self.0.fetch_user_id(user_token).await.map_err(|e| {
            Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(e.to_string()))
                .unwrap()
        })
    }
}


impl<Session> FromRef<AppState<Session>> for Rooms {
    fn from_ref(input: &AppState<Session>) -> Self {
        input.rooms.clone()
    }
}


impl<Session> FromRef<AppState<Session>> for SessionState<Session>
where
    Session: SessionIo + Clone,
{
    fn from_ref(input: &AppState<Session>) -> Self {
        input.session.clone()
    }
}
