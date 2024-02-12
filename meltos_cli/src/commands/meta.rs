use crate::commands::CommandExecutable;
use async_trait::async_trait;
use clap::Args;
use meltos_tvc::file_system::std_fs::StdFileSystem;
use meltos_tvc::io::atomic::object::ObjIo;
use meltos_tvc::io::commit_obj::CommitObjIo;
use meltos_tvc::object::{AsMeta, ObjHash};

#[derive(Args, Debug, Clone)]
pub struct MetaArgs {
    #[clap(short, long)]
    commit: String,
}

#[async_trait(?Send)]
impl CommandExecutable for MetaArgs {
    async fn execute(self) -> meltos_client::error::Result {
        let commit = CommitObjIo::new(StdFileSystem);

        let tree = commit.read_commit_tree(&ObjHash(self.commit)).await?;

        let obj = ObjIo::new(StdFileSystem);
        for hash in tree.values() {
            let buf = obj.try_read_obj(hash).await;
            println!(
                "{hash} : {:?}",
                buf.map(|b| String::from_utf8(b.as_meta().unwrap().buf).unwrap())
            );
        }
        Ok(())
    }
}
