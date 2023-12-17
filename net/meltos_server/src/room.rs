use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::future::Future;
use std::sync::Arc;

use axum::body::Body;
use axum::http::StatusCode;
use axum::response::Response;
use serde::Serialize;
use serde_json::json;

use meltos::room::RoomId;
use meltos::user::UserId;
use meltos_backend::discussion::DiscussionIo;
use meltos_tvn::branch::BranchName;
use meltos_tvn::file_system::file::StdFileSystem;
use meltos_tvn::io::bundle::Bundle;
use meltos_tvn::operation::Operations;
use meltos_tvn::operation::push::PushParam;
use meltos_util::macros::Deref;
use meltos_util::sync::arc_mutex::ArcMutex;

use crate::api::AsSuccessResponse;
use crate::error;
use crate::room::executor::discussion::DiscussionCommandExecutor;

mod executor;

#[derive(Default, Clone, Debug, Deref)]
pub struct Rooms(ArcMutex<RoomMap>);

impl Rooms {
    pub async fn insert_room(&self, room: Room) {
        let mut rooms = self.0.lock().await;
        rooms.0.insert(room.id.clone(), room);
    }
}

#[derive(Default, Debug)]
pub struct RoomMap(HashMap<RoomId, Room>);

impl RoomMap {
    pub fn room(&mut self, room_id: &RoomId) -> std::result::Result<Room, Response> {
        Ok(self.room_mut(room_id)?.clone())
    }

    pub fn room_mut(&mut self, room_id: &RoomId) -> std::result::Result<&mut Room, Response> {
        self.0.get_mut(room_id).ok_or(
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(
                    json!({
                        "error": format!("room_id {room_id} is not exists")
                    })
                        .to_string(),
                ))
                .unwrap(),
        )
    }
}

#[derive(Clone)]
pub struct Room {
    pub owner: UserId,
    pub id: RoomId,
    pub tvn: Operations<StdFileSystem, File>,
    discussion: Arc<dyn DiscussionIo>,
}


impl Room {
    pub fn open<Discussion: DiscussionIo + Default + 'static>(owner: UserId) -> Self {
        Self {
            id: RoomId::default(),
            owner: owner.clone(),
            discussion: Arc::new(Discussion::default()),
            tvn: Operations::new(BranchName::from(owner.to_string()), StdFileSystem),
        }
    }

    pub fn save_commits(&self, push_param: PushParam) -> std::result::Result<(), Response> {
        self.tvn.save.execute(push_param).map_err(|e| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(
                    json!({
                        "error" : e.to_string()
                    })
                        .to_string(),
                ))
                .unwrap()
        })?;

        Ok(())
    }


    pub fn create_bundle(&self) -> std::result::Result<Bundle, Response> {
        match self.tvn.bundle.create() {
            Ok(bundle) => Ok(bundle),
            Err(error) => {
                let response = Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from(json!({
                        "error": error.to_string()
                    }).to_string()))
                    .unwrap();
                Err(response)
            }
        }
    }


    pub async fn global_discussion<'a, F, O, S>(
        &'a self,
        user_id: UserId,
        f: F,
    ) -> error::Result<Response>
        where
            F: FnOnce(DiscussionCommandExecutor<'a, dyn DiscussionIo>) -> O,
            O: Future<Output=error::Result<S>>,
            S: Serialize,
    {
        let command = f(self.as_global_discussion_executor(user_id)).await?;
        Ok(command.as_success_response())
    }

    fn as_global_discussion_executor(
        &self,
        user_id: UserId,
    ) -> DiscussionCommandExecutor<'_, dyn DiscussionIo> {
        DiscussionCommandExecutor::new(user_id, self.discussion.as_ref())
    }
}

impl Debug for Room {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Room")
            .field("id", &self.id)
            .field("owner", &self.owner)
            .finish()
    }
}
