use async_trait::async_trait;

use meltos::room::RoomId;
use meltos::user::UserId;
use meltos_tvc::branch::BranchName;
use meltos_tvc::file_system::{FilePath, FileSystem};
use meltos_tvc::io::atomic::head::HeadIo;
use meltos_tvc::io::atomic::staging::StagingIo;
use meltos_tvc::io::bundle::Bundle;
use meltos_tvc::io::trace_tree::TraceTreeIo;
use meltos_tvc::object::tree::TreeObj;
use meltos_tvc::operation::merge::MergedStatus;
use meltos_tvc::operation::Operations;
use meltos_tvc::operation::push::Pushable;

use crate::config::SessionConfigs;
use crate::error;
use crate::http::HttpClient;

const BASE: &str = "http://127.0.0.1:3000";

pub struct TvcClient<Fs: FileSystem + Clone> {
    operations: Operations<Fs>,
    staging: StagingIo<Fs>,
    head: HeadIo<Fs>,
    trace: TraceTreeIo<Fs>,
    fs: Fs,
    branch_name: String,
}

impl<Fs: FileSystem + Clone> TvcClient<Fs> {
    pub fn new(branch_name: String, fs: Fs) -> Self {
        Self {
            operations: Operations::new(BranchName::from(branch_name.clone()), fs.clone()),
            staging: StagingIo::new(fs.clone()),
            head: HeadIo::new(fs.clone()),
            trace: TraceTreeIo::new(fs.clone()),
            fs,
            branch_name,
        }
    }

    pub async fn open_room(&self, lifetime_sec: Option<u64>) -> error::Result<SessionConfigs> {
        self.operations.init.execute()?;
        let mut sender = OpenSender {
            user_id: Some(BranchName::owner().0),
            lifetime_sec,
        };

        let session_configs = self.operations.push.execute(&mut sender).await?;
        Ok(session_configs)
    }

    pub async fn join_room(
        &self,
        room_id: String,
        user_id: String,
    ) -> error::Result<SessionConfigs> {
        let (http, bundle) =
            HttpClient::join(BASE, RoomId(room_id), Some(UserId(user_id.clone()))).await?;

        self.operations.save.execute(bundle)?;
        self.operations.checkout.execute(&BranchName(user_id))?;
        self.operations
            .unzip
            .execute(&BranchName(self.branch_name.clone()))?;

        Ok(http.configs().clone())
    }

    #[inline]
    pub async fn fetch(&self, session_config: SessionConfigs) -> error::Result {
        let http = HttpClient::new(BASE, session_config);
        let bundle = http.fetch().await?;
        self.operations.save.execute(bundle)?;
        Ok(())
    }

    pub fn stage(&self, path: String) -> error::Result {
        self.operations.stage.execute(&path)?;
        Ok(())
    }

    pub fn commit(&self, commit_text: String) -> error::Result {
        self.operations.commit.execute(commit_text)?;
        Ok(())
    }

    pub async fn push(&mut self, session_configs: SessionConfigs) -> error::Result {
        let mut sender = PushSender {
            session_configs,
        };
        self.operations.push.execute(&mut sender).await?;
        Ok(())
    }

    pub fn merge(&self, source: String) -> error::Result<MergedStatus> {
        let source = BranchName(source);
        let dist = BranchName(self.branch_name.clone());
        let status = self.operations.merge.execute(source, dist)?;
        Ok(status)
    }


    pub fn staging_files(&self) -> error::Result<Vec<String>> {
        let tree = self.staging.read()?;
        Ok(tree
            .as_ref()
            .map(|tree| tree.keys().map(|path| path.to_string()).collect())
            .unwrap_or_default())
    }


    pub fn traces(&self) -> error::Result<Option<TreeObj>> {
        let Some(head) = self.head.read(&BranchName(self.branch_name.clone()))?
            else {
                return Ok(None);
            };
        let trace_tree = self.trace.read(&head)?;
        Ok(Some(trace_tree))
    }


    pub fn exists_in_traces(&self, file_path: &str) -> error::Result<bool> {
        let Some(head) = self.head.read(&BranchName(self.branch_name.clone()))?
            else {
                return Ok(false);
            };
        let trace_tree = self.trace.read(&head)?;
        Ok(trace_tree.contains_key(&FilePath::from(file_path)))
    }


    pub fn close(&self) -> error::Result {
        self.fs.delete(".")?;
        Ok(())
    }
}

struct OpenSender {
    user_id: Option<String>,
    lifetime_sec: Option<u64>,
}

#[async_trait(? Send)]
impl Pushable<SessionConfigs> for OpenSender {
    type Error = crate::error::Error;

    async fn push(&mut self, bundle: Bundle) -> Result<SessionConfigs, Self::Error> {
        let http = HttpClient::open(
            "http://localhost:3000",
            Some(bundle),
            self.user_id.clone().map(UserId::from),
            self.lifetime_sec,
        )
            .await?;
        Ok(http.configs().clone())
    }
}

struct PushSender {
    session_configs: SessionConfigs,
}

#[async_trait(? Send)]
impl Pushable<()> for PushSender {
    type Error = String;

    async fn push(&mut self, bundle: Bundle) -> Result<(), Self::Error> {
        let mut http = HttpClient::new("http://localhost:3000", self.session_configs.clone());
        http.push(bundle).await?;

        Ok(())
    }
}
