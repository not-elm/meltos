use axum::body::Body;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::Response;
use axum_extra::extract::cookie::Cookie;
use serde_json::json;

use meltos::user::SessionId;

pub mod owner;
pub mod user;

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
