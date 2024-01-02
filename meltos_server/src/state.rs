use auto_delegate::Delegate;
use axum::body::Body;
use axum::extract::FromRef;
use axum::http::StatusCode;
use axum::response::Response;
use serde_json::json;

use meltos::user::{SessionId, UserId};
use meltos_backend::user::SessionIo;

use crate::room::Rooms;

#[derive(Clone, Default)]
pub struct AppState<Session> {
    pub(crate) rooms: Rooms,
    pub(crate) session: SessionState<Session>,
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
        user_token: SessionId,
    ) -> std::result::Result<UserId, Response<Body>> {
        self.0.fetch(user_token).await.map_err(|e| {
            Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(e.to_string()))
                .unwrap()
        })
    }

    pub async fn register(
        &self,
        user_id: Option<UserId>,
    ) -> std::result::Result<(UserId, SessionId), Response<Body>> {
        match self.0.register(user_id).await {
            Ok(id_pair) => Ok(id_pair),
            Err(e) => {
                Err(axum::http::Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from(
                        json!({
                            "error" : e.to_string()
                        })
                        .to_string(),
                    ))
                    .unwrap())
            }
        }
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
