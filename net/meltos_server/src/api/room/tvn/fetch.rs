use crate::api::{AsSuccessResponse, HttpResult};
use crate::middleware::room::SessionRoom;

#[tracing::instrument]
pub async fn fetch(
    SessionRoom(room): SessionRoom
) -> HttpResult{
    let bundle = room.create_bundle()?;
    Ok(bundle.as_success_response())
}


#[cfg(test)]
mod tests{
    use meltos_tvn::file_system::mock::MockFileSystem;
    use crate::api::test_util::{http_fetch, http_open_room, logged_in_app, mock_session_id};

    #[tokio::test]
    async fn fetch(){
        let mock = MockFileSystem::default();
        let (session_id, mut app)= logged_in_app().await;
        let room_id = http_open_room(&mut app, mock.clone(), session_id).await;
        let _bundle = http_fetch(&mut app, &room_id, &mock_session_id()).await;
    }
}