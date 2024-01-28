use axum::async_trait;
use axum::body::Body;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::Response;
use axum_extra::extract::cookie::Cookie;
use serde_json::json;

use meltos::user::{SessionId, UserId};

use crate::middleware::room::PathParam;
use crate::state::AppState;

#[derive(Eq, PartialEq, Clone, Hash, Debug)]
pub struct SessionUser(pub UserId);

#[async_trait]
impl FromRequestParts<AppState> for SessionUser {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let session_id = extract_session_id_from_cookie(parts)?;
        let room_id = PathParam::new(parts, state).await?.room_id;
        let mut rooms = state.rooms.lock().await;
        let room = rooms.room(&room_id)?;
        let user_id = room.session.fetch(session_id).await?;
        Ok(Self(user_id))
    }
}

fn response_unauthorized() -> Response {
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .body(Body::from(
            json!({
                "error" : "unauthorized"
            })
            .to_string(),
        ))
        .unwrap()
}

fn extract_session_id_from_cookie(parts: &mut Parts) -> Result<SessionId, Response> {
    let cookies = parts
        .headers
        .get("set-cookie")
        .ok_or(response_unauthorized())?
        .to_str()
        .map_err(|_| response_unauthorized())?
        .to_string();
    let cookies = Cookie::split_parse(cookies);
    let cookie = cookies
        .filter_map(|cookie| cookie.ok())
        .find(|cookie| cookie.name() == "session_id")
        .ok_or(response_unauthorized())?;

    Ok(SessionId(cookie.value().to_string()))
}
