use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use meltos_core::room::RoomId;
use meltos_core::user::UserId;
use meltos_tvc::branch::BranchName;
use meltos_tvc::file_system::{FilePath, FileSystem};
use meltos_tvc::io::atomic::head::HeadIo;
use meltos_tvc::io::atomic::local_commits::LocalCommitsIo;
use meltos_tvc::io::atomic::object::ObjIo;
use meltos_tvc::io::atomic::staging::StagingIo;
use meltos_tvc::io::bundle::Bundle;
use meltos_tvc::io::commit_hashes::CommitHashIo;
use meltos_tvc::io::commit_obj::CommitObjIo;
use meltos_tvc::io::trace_tree::TraceTreeIo;
use meltos_tvc::io::workspace::WorkspaceIo;
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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjMeta {
    pub file_path: String,
    pub hash: String,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommitMeta {
    pub hash: String,
    pub message: String,
    pub objs: Vec<ObjMeta>,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BranchCommitMeta {
    pub name: String,
    pub commits: Vec<CommitMeta>,
}

pub const BASE: &str = "http://room.meltos.net";

#[derive(Clone)]
pub struct TvcClient<Fs: FileSystem + Clone> {
    operations: Operations<Fs>,
    staging: StagingIo<Fs>,
    head: HeadIo<Fs>,
    trace: TraceTreeIo<Fs>,
    local_commits: LocalCommitsIo<Fs>,
    commit_obj: CommitObjIo<Fs>,
    commit_hashes: CommitHashIo<Fs>,
    workspace: WorkspaceIo<Fs>,
    obj: ObjIo<Fs>,
    fs: Fs,
}

impl<Fs: FileSystem + Clone> TvcClient<Fs> {
    pub fn new(fs: Fs) -> Self {
        Self {
            operations: Operations::new(fs.clone()),
            staging: StagingIo::new(fs.clone()),
            head: HeadIo::new(fs.clone()),
            trace: TraceTreeIo::new(fs.clone()),
            local_commits: LocalCommitsIo::new(fs.clone()),
            commit_obj: CommitObjIo::new(fs.clone()),
            commit_hashes: CommitHashIo::new(fs.clone()),
            workspace: WorkspaceIo::new(fs.clone()),
            obj: ObjIo::new(fs.clone()),
            fs,
        }
    }

    /// このメソッドはクライアントツール側でテストを実行する際に使用する想定です。
    pub async fn init_repository(&self, branch_name: &BranchName) -> error::Result<CommitHash> {
        let commit_hash = self.operations.init.execute(branch_name).await?;
        Ok(commit_hash)
    }

    pub async fn branch_names(&self) -> error::Result<Vec<BranchName>> {
        Ok(self
            .head
            .read_all()
            .await?
            .into_iter()
            .map(|(branch_name, _)| branch_name)
            .collect())
    }


    #[inline(always)]
    pub async fn unzip(&self, branch_name: &BranchName) -> error::Result {
        self.operations.unzip.execute(branch_name).await?;
        Ok(())
    }

    pub async fn open_room(&mut self, lifetime_sec: Option<u64>, user_limits: Option<u64>) -> error::Result<SessionConfigs> {
        let branch = BranchName::owner();

        self.operations.init.execute(&branch).await?;
        let mut sender = OpenSender {
            lifetime_sec,
            user_limits,
        };

        let session_configs = self
            .operations
            .push
            .execute(branch, &mut sender)
            .await?;
        Ok(session_configs)
    }

    pub async fn join_room(
        &mut self,
        room_id: String,
        user_id: Option<UserId>,
    ) -> error::Result<SessionConfigs> {
        let (http, bundle) = HttpClient::join(BASE, RoomId(room_id), user_id).await?;
        let branch = BranchName(http.configs().user_id.0.clone());

        self.operations.save.execute(bundle).await?;
        self.operations.checkout.execute(&branch).await?;
        self.operations.unzip.execute(&branch).await?;

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
        self.operations.save.execute(bundle).await?;
        Ok(())
    }

    #[inline(always)]
    pub async fn stage(&self, branch_name: &BranchName, path: String) -> error::Result {
        self.operations.stage.execute(branch_name, &path).await?;
        Ok(())
    }

    #[inline(always)]
    pub async fn un_stage(&self, file_path: &str) -> error::Result {
        self.operations.un_stage.execute(file_path).await?;
        Ok(())
    }

    #[inline(always)]
    pub async fn un_stage_all(&self) -> error::Result {
        self.operations.un_stage.execute_all().await?;
        Ok(())
    }

    #[inline(always)]
    pub async fn commit(&self, branch_name: &BranchName, commit_text: String) -> error::Result<CommitHash> {
        Ok(self
            .operations
            .commit
            .execute(branch_name, commit_text)
            .await?)
    }

    pub async fn push(&self, session_configs: SessionConfigs) -> error::Result {
        let branch_name = session_configs.user_id.clone().into();
        let mut sender = PushSender {
            session_configs,
        };
        self.operations
            .push
            .execute(branch_name, &mut sender)
            .await?;
        Ok(())
    }

    pub async fn merge(&self, dist: BranchName, source_commit_hash: CommitHash) -> error::Result<MergedStatus> {
        let status = self.operations.merge.execute(source_commit_hash, dist).await?;
        Ok(status)
    }

    pub async fn read_file_from_hash(&self, obj_hash: &ObjHash) -> error::Result<Option<String>> {
        let Some(file_obj) = self.obj.try_read_to_file(obj_hash).await? else {
            return Ok(None);
        };

        Ok(Some(String::from_utf8(file_obj.0).unwrap()))
    }

    pub async fn all_branch_commit_metas(&self) -> error::Result<Vec<BranchCommitMeta>> {
        let heads = self.head.read_all().await?;
        let mut branches = Vec::with_capacity(heads.len());
        for (branch, _) in heads {
            branches.push(BranchCommitMeta {
                commits: self.all_commit_metas(&branch).await?,
                name: branch.0,
            });
        }
        Ok(branches)
    }


    async fn all_commit_metas(&self, branch_name: &BranchName) -> error::Result<Vec<CommitMeta>> {
        let Some(head) = self.head.read(branch_name).await? else {
            return Ok(Vec::with_capacity(0));
        };
        let hashes = self.commit_hashes.read_all(head, &None).await?;
        let mut metas = Vec::with_capacity(hashes.len());
        for commit_hash in hashes {
            metas.push(self.read_commit_meta(&commit_hash).await?);
        }
        Ok(metas)
    }

    pub async fn read_commit_meta(&self, commit_hash: &CommitHash) -> error::Result<CommitMeta> {
        let obj = self.commit_obj.read(commit_hash).await?;
        let tree = self.commit_obj.read_commit_tree(commit_hash).await?;
        Ok(CommitMeta {
            hash: commit_hash.to_string(),
            message: obj.text.0,
            objs: tree
                .0
                .into_iter()
                .map(|(file_path, hash)| {
                    ObjMeta {
                        file_path: file_path.0,
                        hash: hash.0,
                    }
                })
                .collect(),
        })
    }

    #[inline(always)]
    pub async fn can_push(&self, branch_name: &BranchName) -> error::Result<bool> {
        Ok(self.local_commits.read(branch_name).await?.is_some_and(|commits| !commits.is_empty()))
    }


    pub async fn staging_files(&self) -> error::Result<Vec<String>> {
        let tree = self.staging.read().await?;
        Ok(tree
            .as_ref()
            .map(|tree| tree.keys().map(|path| path.to_string()).collect())
            .unwrap_or_default())
    }

    pub async fn traces(&self, branch_name: &BranchName) -> error::Result<Option<TreeObj>> {
        let Some(head) = self.head.read(branch_name).await? else {
            return Ok(None);
        };
        let trace_tree = self.trace.read(&head).await?;
        Ok(Some(trace_tree))
    }

    pub async fn find_obj_hash_from_traces(&self, branch_name: &BranchName, file_path: &str) -> error::Result<Option<ObjHash>> {
        let Some(head) = self.head.read(branch_name).await? else {
            return Ok(None);
        };
        let trace_tree = self.trace.read(&head).await?;
        Ok(trace_tree.get(&FilePath::from(format!("workspace/{file_path}"))).cloned())
    }


    #[inline(always)]
    pub async fn is_change(&self, branch_name: &BranchName, file_path: &FilePath) -> error::Result<bool> {
        Ok(self.workspace.is_change(branch_name, file_path).await?)
    }

    #[inline(always)]
    pub async fn save_bundle(&self, bundle: Bundle) -> error::Result {
        self.operations.save.execute(bundle).await?;
        Ok(())
    }

    pub async fn close(&self) -> error::Result {
        self.fs.delete(".").await?;
        Ok(())
    }
}

struct OpenSender {
    lifetime_sec: Option<u64>,
    user_limits: Option<u64>,
}

#[async_trait(? Send)]
impl Pushable<SessionConfigs> for OpenSender {
    type Error = crate::error::Error;

    async fn push(&mut self, bundle: Bundle) -> Result<SessionConfigs, Self::Error> {
        let http = HttpClient::open(
            BASE,
            Some(bundle),
            self.lifetime_sec,
            self.user_limits,
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
        let mut http = HttpClient::new(BASE, self.session_configs.clone());
        http.push(bundle).await?;

        Ok(())
    }
}
