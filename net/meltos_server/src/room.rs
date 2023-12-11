use std::collections::HashMap;

use axum::body::Body;
use axum::http::StatusCode;
use axum::response::Response;
use serde_json::json;

use meltos::discussion::io::global::mock::MockGlobalDiscussionIo;
use meltos::room::RoomId;
use meltos::user::UserId;
use meltos_util::macros::Deref;
use meltos_util::sync::arc_mutex::ArcMutex;

use crate::room::executor::discussion::DiscussionCommandExecutor;

mod executor;


#[derive(Default, Deref, Clone, Debug)]
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
    pub fn room_mut(&mut self, room_id: &RoomId) -> std::result::Result<&mut Room, Response> {
        self.0.get_mut(room_id).ok_or(
            Response::builder()
                .status(StatusCode::BAD_REQUEST)
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


#[derive(Debug)]
pub struct Room {
    pub owner: UserId,
    pub id: RoomId,
    global_discussion_io: MockGlobalDiscussionIo,
}


impl Room {
    pub fn open(owner: UserId) -> Self {
        Self {
            id: RoomId::default(),
            owner,
            global_discussion_io: MockGlobalDiscussionIo::default(),
        }
    }


    pub fn as_global_discussion_executor(
        &self,
        user_id: UserId,
    ) -> DiscussionCommandExecutor<'_, MockGlobalDiscussionIo> {
        DiscussionCommandExecutor::new(user_id, &self.global_discussion_io)
    }
}
