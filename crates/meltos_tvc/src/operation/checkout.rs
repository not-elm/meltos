use crate::branch::BranchName;
use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::head::HeadIo;
use crate::io::atomic::work_branch::WorkingIo;
use crate::operation::new_branch::NewBranch;
use crate::operation::unzip::UnZip;

#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash)]
pub enum CheckOutStatus {
    AlreadyCheckedOut,
    Checkout,
    NewBranch,
}

#[derive(Debug, Clone)]
pub struct Checkout<Fs>
where
    Fs: FileSystem,
{
    working: WorkingIo<Fs>,
    heads: HeadIo<Fs>,
    new_branch: NewBranch<Fs>,
    unzip: UnZip<Fs>,
}

impl<Fs> Checkout<Fs>
where
    Fs: FileSystem + Clone,
{
    pub fn new(fs: Fs) -> Checkout<Fs> {
        Self {
            working: WorkingIo::new(fs.clone()),
            heads: HeadIo::new(fs.clone()),
            new_branch: NewBranch::new(fs.clone()),
            unzip: UnZip::new(fs),
        }
    }
}

impl<Fs> Checkout<Fs>
where
    Fs: FileSystem,
{
    pub async fn execute(&self, target_branch: &BranchName) -> error::Result<CheckOutStatus> {
        let working = self.working.read().await?.unwrap_or(BranchName::owner());
        if &working == target_branch {
            return Ok(CheckOutStatus::AlreadyCheckedOut);
        }

        if self.heads.read(target_branch).await?.is_some() {
            self.working.write(target_branch).await?;
            self.unzip.execute(target_branch).await?;
            return Ok(CheckOutStatus::Checkout);
        }

        if let Some(branch_hash) = self.heads.read_remote(target_branch).await? {
            self.heads.write(target_branch, &branch_hash).await?;
            self.working.write(target_branch).await?;
            self.unzip.execute(target_branch).await?;
            return Ok(CheckOutStatus::Checkout);
        }

        self.new_branch
            .execute(working, target_branch.clone())
            .await?;
        self.working.write(target_branch).await?;
        Ok(CheckOutStatus::NewBranch)
    }
}

#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::file_system::memory::MemoryFileSystem;
    use crate::file_system::FileSystem;
    use crate::io::atomic::work_branch::WorkingIo;
    use crate::operation::checkout::{CheckOutStatus, Checkout};
    use crate::operation::commit::Commit;
    use crate::operation::new_branch::NewBranch;
    use crate::operation::stage::Stage;
    use crate::tests::init_owner_branch;

    #[tokio::test]
    async fn not_checkout_if_already_working() {
        let fs = MemoryFileSystem::default();
        init_owner_branch(fs.clone()).await;
        let checked = Checkout::new(fs.clone())
            .execute(&BranchName::owner())
            .await
            .unwrap();
        assert_eq!(checked, CheckOutStatus::AlreadyCheckedOut);
        let working = WorkingIo::new(fs.clone()).try_read().await.unwrap();
        assert_eq!(BranchName::owner(), working);
    }

    #[tokio::test]
    async fn checkout_if_exists_local() {
        let fs = MemoryFileSystem::default();
        init_owner_branch(fs.clone()).await;
        let second = BranchName::from("second");
        NewBranch::new(fs.clone())
            .execute(BranchName::owner(), second.clone())
            .await
            .unwrap();
        assert_eq!(
            Checkout::new(fs.clone()).execute(&second).await.unwrap(),
            CheckOutStatus::Checkout
        );
        let working = WorkingIo::new(fs.clone()).try_read().await.unwrap();
        assert_eq!(second, working);
    }

    #[tokio::test]
    async fn create_new_branch_if_not_exists() {
        let fs = MemoryFileSystem::default();
        init_owner_branch(fs.clone()).await;
        let second = BranchName::from("second");
        assert_eq!(
            Checkout::new(fs.clone()).execute(&second).await.unwrap(),
            CheckOutStatus::NewBranch
        );
        let working = WorkingIo::new(fs.clone()).try_read().await.unwrap();
        assert_eq!(second, working);
    }

    #[tokio::test]
    async fn remote_hello_txt() {
        let fs = MemoryFileSystem::default();
        init_owner_branch(fs.clone()).await;
        let checkout = Checkout::new(fs.clone());
        let b1 = BranchName::owner();
        let b2 = BranchName::from("session");
        checkout.execute(&b2).await.unwrap();
        checkout.execute(&b1).await.unwrap();

        fs.write_sync("workspace/hello.txt", b"hello");
        Stage::new(fs.clone())
            .execute(&b1, "hello.txt")
            .await
            .unwrap();

        Commit::new(fs.clone())
            .execute(&b1, "commit text")
            .await
            .unwrap();

        checkout.execute(&b2).await.unwrap();

        let hello_txt = fs.read_file("workspace/hello.txt").await.unwrap();
        assert!(hello_txt.is_none());
    }
}
