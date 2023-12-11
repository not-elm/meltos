use auto_delegate::Delegate;
use axum::extract::FromRef;

use meltos_backend::user::UserSessionIo;

use crate::room::Rooms;

#[derive(Clone, Default)]
pub struct AppState<Session> {
    rooms: Rooms,
    session: SessionState<Session>,
}


impl<Session> AppState<Session>
    where Session: UserSessionIo + Clone
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


impl<Session> FromRef<AppState<Session>> for Rooms {
    fn from_ref(input: &AppState<Session>) -> Self {
        input.rooms.clone()
    }
}


impl<Session> FromRef<AppState<Session>> for SessionState<Session>
    where Session: UserSessionIo + Clone
{
    fn from_ref(input: &AppState<Session>) -> Self {
        input.session.clone()
    }
}



