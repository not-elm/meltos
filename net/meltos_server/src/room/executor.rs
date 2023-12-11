use meltos::command::client;
use meltos::command::client::ClientCommand;
use meltos::command::request::RequestCommand;
use meltos::command::request::thread::global::GlobalThreadCommand;
use meltos::command::request::thread::ThreadCommand;
use meltos::session::RoomId;
use meltos::thread::io::ThreadIo;
use meltos::user::UserId;

use crate::error;

pub(crate) struct ServerOrderExecutor<'a, Global> {
    _room_id: RoomId,
    from: UserId,
    thread_io: &'a Global,
}

impl<'a, Global> ServerOrderExecutor<'a, Global>
    where Global: ThreadIo
{
    pub fn new(
        room_id: RoomId,
        user_id: UserId,
        io: &'a Global,
    ) -> ServerOrderExecutor<'a, Global> {
        Self {
            _room_id: room_id,
            from: user_id,
            thread_io: io,
        }
    }

    pub async fn execute(self, command: RequestCommand) -> error::Result<Option<ClientCommand>> {
        match command {
            RequestCommand::Thread(thread) => self.execute_thread_command(thread).await
        }
    }

    async fn execute_thread_command(self, thread_command: ThreadCommand) -> error::Result<Option<ClientCommand>> {
        match thread_command {
            ThreadCommand::Global(global) => self.exe_global_thread(global).await
        }
    }

    async fn exe_global_thread(self, global: GlobalThreadCommand) -> error::Result<Option<ClientCommand>> {
        match global {
            GlobalThreadCommand::NewThread => {
                self.thread_io.new_thread().await?;
                let order = client::thread::global::GlobalThreadOrder::NewThreadNotify {
                    creator: self.from
                };

                Ok(Some(ClientCommand::Thread(client::thread::ThreadOrder::Global(order))))
            }
        }
    }
}
