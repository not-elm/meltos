use meltos::command::client::ClientCommand;
use meltos::command::request::RequestCmd;
use meltos::discussion::io::DiscussionIo;
use meltos::room::RoomId;
use meltos::user::UserId;

use crate::error;
use crate::room::executor::discussion::DiscussionCommandExecutor;

mod discussion;

pub(crate) struct ServerCommandExecutor<'a, Global> {
    _room_id: RoomId,
    from: UserId,
    global_discussion_io: &'a Global,
}


impl<'a, Global> ServerCommandExecutor<'a, Global>
where
    Global: DiscussionIo,
{
    pub fn new(
        room_id: RoomId,
        user_id: UserId,
        io: &'a Global,
    ) -> ServerCommandExecutor<'a, Global> {
        Self {
            _room_id: room_id,
            from: user_id,
            global_discussion_io: io,
        }
    }

    pub async fn execute(self, command: RequestCmd) -> error::Result<Option<ClientCommand>> {
        match command {
            RequestCmd::Discussion(cmd) => {
                DiscussionCommandExecutor::new(self.from, self.global_discussion_io)
                    .execute(cmd)
                    .await
            }
        }
    }
}
