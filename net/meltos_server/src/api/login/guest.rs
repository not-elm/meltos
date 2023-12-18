use std::fmt::Debug;

use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{Response, StatusCode};
use axum_extra::extract::cookie::Expiration::DateTime;
use serde::{Deserialize, Serialize};
use serde_json::json;

use meltos::user::{SessionId, UserId};
use meltos_backend::user::SessionIo;

use crate::api::HttpResult;
use crate::state::SessionState;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Param {
    user_id: Option<UserId>,
}


/// Creates the guest user.
///
/// Guest user session is automatically deleted after 3 hours.
#[tracing::instrument]
pub async fn guest<Session>(
    State(session): State<SessionState<Session>>,
    Query(query): Query<Param>,
) -> HttpResult
    where
        Session: SessionIo + Debug,
{
    let session_id = SessionId::new();
    let user_id = query.user_id.unwrap_or(UserId::new());
    session.register(session_id.clone(), user_id.clone()).await?;
    Ok(response_ok(session_id, user_id))
}

fn response_ok(session_id: SessionId, user_id: UserId) -> Response<Body> {
    let response_message = json!({
        "user_id" : user_id,
        "session_id" : session_id
    })
        .to_string();
    Response::new(Body::from(response_message))
}

