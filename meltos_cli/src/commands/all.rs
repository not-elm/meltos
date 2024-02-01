use crate::commands::{load_branch_name, load_configs, CommandExecutable};
use async_trait::async_trait;
use clap::Args;
use meltos_client::tvc::TvcClient;
use meltos_tvc::file_system::std_fs::StdFileSystem;

#[derive(Debug, Clone, Args)]
pub struct AllArgs {
    #[clap(short, long)]
    commit_text: Option<String>,
}

#[async_trait(?Send)]
impl CommandExecutable for AllArgs {
    async fn execute(self) -> meltos_client::error::Result {
        let tvc = TvcClient::new(StdFileSystem);
        let branch_name = load_branch_name()?;
        tvc.stage(&branch_name, ".".to_string()).await?;
        tvc.commit(
            &branch_name,
            self.commit_text.unwrap_or("COMMIT".to_string()),
        )
        .await?;
        tvc.push(load_configs()?).await?;
        Ok(())
    }
}
