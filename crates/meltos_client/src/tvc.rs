use async_trait::async_trait;
use wasm_bindgen::prelude::wasm_bindgen;

use meltos::room::RoomId;
use meltos::user::UserId;
use meltos_tvc::branch::BranchName;
use meltos_tvc::file_system::{FilePath, FileSystem};
use meltos_tvc::io::atomic::head::HeadIo;
use meltos_tvc::io::atomic::object::ObjIo;
use meltos_tvc::io::atomic::staging::StagingIo;
use meltos_tvc::io::bundle::Bundle;
use meltos_tvc::io::commit_hashes::CommitHashIo;
use meltos_tvc::io::commit_obj::CommitObjIo;
use meltos_tvc::io::trace_tree::TraceTreeIo;
use meltos_tvc::object::commit::CommitHash;
use meltos_tvc::object::ObjHash;
use meltos_tvc::object::tree::TreeObj;
use meltos_tvc::operation::merge::MergedStatus;
use meltos_tvc::operation::Operations;
use meltos_tvc::operation::push::Pushable;

use crate::config::SessionConfigs;
use crate::error;
use crate::http::HttpClient;

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug)]
pub struct ObjMeta {
    pub file_path: String,
    pub hash: String,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug)]
pub struct CommitMeta {
    pub hash: String,
    pub message: String,
    pub objs: Vec<ObjMeta>,
}

pub const BASE: &str = "http://127.0.0.1:3000";

#[derive(Clone)]
pub struct TvcClient<Fs: FileSystem + Clone> {
    operations: Operations<Fs>,
    staging: StagingIo<Fs>,
    head: HeadIo<Fs>,
    trace: TraceTreeIo<Fs>,
    commit_obj: CommitObjIo<Fs>,
    commit_hashes: CommitHashIo<Fs>,
    obj: ObjIo<Fs>,
    fs: Fs,
    branch_name: BranchName,
}

impl<Fs: FileSystem + Clone> TvcClient<Fs> {
    pub fn new(branch_name: String, fs: Fs) -> Self {
        Self {
            operations: Operations::new(fs.clone()),
            staging: StagingIo::new(fs.clone()),
            head: HeadIo::new(fs.clone()),
            trace: TraceTreeIo::new(fs.clone()),
            commit_obj: CommitObjIo::new(fs.clone()),
            commit_hashes: CommitHashIo::new(fs.clone()),
            obj: ObjIo::new(fs.clone()),
            fs,
            branch_name: BranchName(branch_name),
        }
    }

    /// このメソッドはクライアントツール側でテストを実行する際に使用する想定です。
    pub fn init_repository(&self) -> error::Result<CommitHash> {
        let commit_hash = self.operations.init.execute(&self.branch_name)?;
        Ok(commit_hash)
    }

    pub async fn open_room(&self, lifetime_sec: Option<u64>) -> error::Result<SessionConfigs> {
        self.operations.init.execute(&self.branch_name)?;
        let mut sender = OpenSender {
            user_id: Some(BranchName::owner().0),
            lifetime_sec,
        };

        let session_configs = self.operations.push.execute(self.branch_name.clone(), &mut sender).await?;
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
            .execute(&self.branch_name)?;

        Ok(http.configs().clone())
    }


    #[inline(always)]
    pub async fn leave(&self, session_configs: SessionConfigs) -> error::Result {
        HttpClient::new(BASE, session_configs).leave().await
    }

    #[inline]
    pub async fn fetch(&self, session_config: SessionConfigs) -> error::Result {
        let http = HttpClient::new(BASE, session_config);
        let bundle = http.fetch().await?;
        self.operations.save.execute(bundle)?;
        Ok(())
    }


    #[inline(always)]
    pub fn stage(&self, path: String) -> error::Result {
        self.operations.stage.execute(&self.branch_name, &path)?;
        Ok(())
    }

    #[inline(always)]
    pub fn un_stage(&self, file_path: &str) -> error::Result {
        self.operations.un_stage.execute(file_path)?;
        Ok(())
    }

    #[inline(always)]
    pub fn un_stage_all(&self) -> error::Result {
        self.operations.un_stage.execute_all()?;
        Ok(())
    }

    #[inline(always)]
    pub fn commit(&self, commit_text: String) -> error::Result<CommitHash> {
        Ok(self.operations.commit.execute(&self.branch_name, commit_text)?)
    }

    pub async fn push(&mut self, session_configs: SessionConfigs) -> error::Result {
        let mut sender = PushSender {
            session_configs,
        };
        self.operations.push.execute(self.branch_name.clone(), &mut sender).await?;
        Ok(())
    }

    pub fn merge(&self, source_commit_hash: CommitHash) -> error::Result<MergedStatus> {
        let dist = self.branch_name.clone();
        let status = self.operations.merge.execute(source_commit_hash, dist)?;
        Ok(status)
    }


    pub fn read_file_from_hash(&self, obj_hash: &ObjHash) -> error::Result<Option<String>> {
        let Some(file_obj) = self.obj.try_read_to_file(obj_hash)?
            else {
                return Ok(None);
            };

        Ok(Some(String::from_utf8(file_obj.0).unwrap()))
    }


    pub fn all_commit_metas(&self) -> error::Result<Vec<CommitMeta>> {
        let Some(head) = self.head.read(&self.branch_name)?
            else {
                return Ok(Vec::with_capacity(0));
            };
        let hashes = self.commit_hashes.read_all(head, &None)?;
        let mut metas = Vec::with_capacity(hashes.len());
        for commit_hash in hashes {
            metas.push(self.read_commit_meta(&commit_hash)?);
        }
        Ok(metas)
    }


    pub fn read_commit_meta(&self, commit_hash: &CommitHash) -> error::Result<CommitMeta> {
        let obj = self.commit_obj.read(commit_hash)?;
        let tree = self.commit_obj.read_commit_tree(commit_hash)?;
        Ok(CommitMeta {
            hash: commit_hash.to_string(),
            message: obj.text.0,
            objs: tree
                .0
                .into_iter()
                .map(|(file_path, hash)| ObjMeta {
                    file_path: file_path.0,
                    hash: hash.0,
                })
                .collect(),
        })
    }

    pub fn staging_files(&self) -> error::Result<Vec<String>> {
        let tree = self.staging.read()?;
        Ok(tree
            .as_ref()
            .map(|tree| tree.keys().map(|path| path.to_string()).collect())
            .unwrap_or_default())
    }


    pub fn traces(&self) -> error::Result<Option<TreeObj>> {
        let Some(head) = self.head.read(&self.branch_name)?
            else {
                return Ok(None);
            };
        let trace_tree = self.trace.read(&head)?;
        Ok(Some(trace_tree))
    }


    pub fn find_obj_hash_from_traces(&self, file_path: &str) -> error::Result<Option<ObjHash>> {
        let Some(head) = self.head.read(&self.branch_name)?
            else {
                return Ok(None);
            };
        let trace_tree = self.trace.read(&head)?;
        Ok(trace_tree.get(&FilePath::from(file_path)).cloned())
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
            "http://127.0.0.1:3000",
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
