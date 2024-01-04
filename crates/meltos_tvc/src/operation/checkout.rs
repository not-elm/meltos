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
    pub fn execute(&self, target_branch: &BranchName) -> error::Result<CheckOutStatus> {
        let working = self.working.read()?.unwrap_or(BranchName::owner());
        if &working == target_branch {
            return Ok(CheckOutStatus::AlreadyCheckedOut);
        }

        if self.heads.read(target_branch)?.is_some() {
            self.working.write(target_branch)?;
            self.unzip.execute(target_branch)?;
            return Ok(CheckOutStatus::Checkout);
        }

        if let Some(branch_hash) = self.heads.read_remote(target_branch)? {
            self.heads.write(target_branch, &branch_hash)?;
            self.working.write(target_branch)?;
            self.unzip.execute(target_branch)?;
            return Ok(CheckOutStatus::Checkout);
        }

        self.new_branch.execute(working, target_branch.clone())?;
        self.working.write(target_branch)?;
        Ok(CheckOutStatus::NewBranch)
    }
}

#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::file_system::mock::MockFileSystem;
    use crate::file_system::FileSystem;
    use crate::io::atomic::work_branch::WorkingIo;
    use crate::operation::checkout::{CheckOutStatus, Checkout};
    use crate::operation::commit::Commit;
    use crate::operation::new_branch::NewBranch;
    use crate::operation::stage::Stage;
    use crate::tests::init_main_branch;

    #[test]
    fn not_checkout_if_already_working() {
        let mock = MockFileSystem::default();
        init_main_branch(mock.clone());
        let checked = Checkout::new(mock.clone())
            .execute(&BranchName::owner())
            .unwrap();
        assert_eq!(checked, CheckOutStatus::AlreadyCheckedOut);
        let working = WorkingIo::new(mock.clone()).try_read().unwrap();
        assert_eq!(BranchName::owner(), working);
    }

    #[test]
    fn checkout_if_exists_local() {
        let mock = MockFileSystem::default();
        init_main_branch(mock.clone());
        let second = BranchName::from("second");
        NewBranch::new(mock.clone())
            .execute(BranchName::owner(), second.clone())
            .unwrap();
        assert_eq!(
            Checkout::new(mock.clone()).execute(&second).unwrap(),
            CheckOutStatus::Checkout
        );
        let working = WorkingIo::new(mock.clone()).try_read().unwrap();
        assert_eq!(second, working);
    }

    #[test]
    fn create_new_branch_if_not_exists() {
        let mock = MockFileSystem::default();
        init_main_branch(mock.clone());
        let second = BranchName::from("second");
        assert_eq!(
            Checkout::new(mock.clone()).execute(&second).unwrap(),
            CheckOutStatus::NewBranch
        );
        let working = WorkingIo::new(mock.clone()).try_read().unwrap();
        assert_eq!(second, working);
    }

    #[test]
    fn remote_hello_txt() {
        let mock = MockFileSystem::default();
        init_main_branch(mock.clone());
        let checkout = Checkout::new(mock.clone());
        let b1 = BranchName::owner();
        let b2 = BranchName::from("user");
        checkout.execute(&b2).unwrap();
        checkout.execute(&b1).unwrap();

        mock.force_write("workspace/hello.txt", b"hello");
        Stage::new(b1.clone(), mock.clone())
            .execute("hello.txt")
            .unwrap();
        Commit::new(b1.clone(), mock.clone())
            .execute("commit text")
            .unwrap();

        checkout.execute(&b2).unwrap();

        let hello_txt = mock.read("workspace/hello.txt").unwrap();
        assert!(hello_txt.is_none());
    }

    // #[tokio::test]
    // async fn checkout_from_remote_branch() {
    //     let mock = MockFileSystem::default();
    //     init_main_branch(mock.clone());
    //     Patch::new(mock.clone(), mock_remote())
    //         .execute(None)
    //         .await
    //         .unwrap();
    //     let second = BranchName::from("second");
    //     assert_eq!(
    //         Checkout::new(mock.clone()).execute(&second).unwrap(),
    //         CheckOutStatus::Checkout
    //     );
    //
    //     let working = WorkingIo::new(mock.clone()).read().unwrap();
    //     assert_eq!(second, working);
    // }
    //
    //
    // fn mock_remote() -> MockRemoteClient {
    //     let mock = MockFileSystem::default();
    //     init_main_branch(mock.clone());
    //     NewBranch::new(mock.clone())
    //         .execute(BranchName::main(), BranchName::from("second"))
    //         .unwrap();
    //     MockRemoteClient::new(mock)
    // }
}
