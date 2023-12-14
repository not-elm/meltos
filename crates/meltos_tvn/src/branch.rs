use serde::{Deserialize, Serialize};

use meltos_util::impl_string_new_type;

use crate::commit::{CommitIo, CommitText};
use crate::error;
use crate::io::{OpenIo, TvnIo};
use crate::now::NowIo;
use crate::object::{ObjectIo, ObjectMeta};
use crate::stage::StageIo;
use crate::tree::Tree;
use crate::workspace::WorkspaceIo;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct BranchName(pub String);
impl_string_new_type!(BranchName);

impl BranchName {
    #[inline]
    pub fn main() -> Self {
        Self::from("main")
    }
}

#[derive(Debug, Clone)]
pub struct BranchIo<Open, Io>
where
    Open: OpenIo<Io>,
    Io: std::io::Write + std::io::Read,
{
    pub(crate) now: NowIo<Open, Io>,
    stage: StageIo<Open, Io>,
    object: ObjectIo<Open, Io>,
    workspace: WorkspaceIo<Open, Io>,
    commit: CommitIo<Open, Io>,
    branch_name: BranchName,
}

impl<Open, Io> BranchIo<Open, Io>
where
    Open: OpenIo<Io> + Clone,
    Io: std::io::Write + std::io::Read,
{
    #[inline]
    pub fn new_main(open: Open) -> BranchIo<Open, Io> {
        Self::new(BranchName::main(), open)
    }

    pub fn new(branch_name: BranchName, open: Open) -> BranchIo<Open, Io> {
        Self {
            object: ObjectIo::new(open.clone()),
            stage: StageIo::new(open.clone()),
            workspace: WorkspaceIo(TvnIo::new(open.clone())),
            now: NowIo::new(branch_name.clone(), open.clone()),
            commit: CommitIo::new(branch_name.clone(), open),
            branch_name,
        }
    }
}

impl<Open, Io> BranchIo<Open, Io>
where
    Open: OpenIo<Io>,
    Io: std::io::Write + std::io::Read,
{
    pub fn init(&self) -> error::Result {
        if self.now.exists()? {
            return Err(error::Error::BranchAlreadyInitialized(
                self.branch_name.clone(),
            ));
        }
        let mut now_tree = Tree::default();
        for meta in self.workspace.convert_to_objs(".")? {
            let meta = meta?;
            self.object.write(&meta.obj)?;
            now_tree.insert(meta.file_path, meta.obj.hash);
        }

        let now_obj = now_tree.as_obj()?;
        self.now.write_hash(&now_obj.hash)?;
        self.object.write(&now_obj)?;
        Ok(())
    }

    pub fn unpack_project(&self) -> error::Result {
        let Some(now_tree) = self.read_now_tree()? else {
            return Ok(());
        };
        for (path, hash) in now_tree.iter() {
            let obj = self.object.try_read_obj(hash)?;
            self.workspace.unpack(path, &obj.buf)?;
        }
        Ok(())
    }

    pub fn stage(&self, workspace_path: &str) -> error::Result {
        let mut stage_tree = self.stage.read_tree()?.unwrap_or_default();
        let now_tree = self.read_now_tree()?;
        for obj in self.workspace.convert_to_objs(workspace_path)? {
            self.stage_file(&mut stage_tree, &now_tree, obj?)?;
        }
        self.stage.write_tree(&stage_tree)?;
        Ok(())
    }

    pub fn commit(&self, commit_text: impl Into<CommitText>) -> error::Result {
        let Some(stage_tree) = self.stage.read_tree()? else {
            return Err(error::Error::NotfoundStages);
        };
        self.stage.reset()?;
        self.update_now_tree(&stage_tree)?;
        self.commit.commit(commit_text, stage_tree)?;
        Ok(())
    }

    fn update_now_tree(&self, stage_tree: &Tree) -> error::Result {
        let stage_obj = stage_tree.as_obj()?;
        self.now.write_hash(&stage_obj.hash)?;
        self.object.write(&stage_obj)?;
        Ok(())
    }

    fn read_now_tree(&self) -> error::Result<Option<Tree>> {
        let Some(now_obj_hash) = self.now.read_hash()? else {
            return Ok(None);
        };
        Ok(Some(self.object.read_to_tree(&now_obj_hash)?))
    }

    fn stage_file(&self, stage: &mut Tree, now: &Option<Tree>, meta: ObjectMeta) -> error::Result {
        if stage.changed_hash(&meta.file_path, meta.hash())
            || now
                .as_ref()
                .is_some_and(|now| now.changed_hash(&meta.file_path, meta.hash()))
        {
            self.object.write(&meta.obj)?;
            stage.insert(meta.file_path, meta.obj.hash);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::branch::BranchIo;
    use crate::io::mock::MockOpenIo;
    use crate::io::{FilePath, OpenIo};
    use crate::object::ObjectHash;

    #[test]
    fn init() {
        let mock = MockOpenIo::default();
        mock.write("./src/main.rs", b"bdadasjlgd").unwrap();
        mock.write("./test.rs", b"test").unwrap();
        let branch = BranchIo::new_main(mock.clone());
        branch.init().unwrap();

        assert!(&mock
            .read_to_end(&format!(
                ".meltos/objects/{}",
                ObjectHash::new(b"bdadasjlgd")
            ))
            .is_ok());
        assert!(&mock
            .read_to_end(&format!(".meltos/objects/{}", ObjectHash::new(b"test")))
            .is_ok());
        assert!(&mock.read_to_end(".meltos/branches/main/NOW").is_ok());
    }

    #[test]
    fn failed_init_if_has_been_initialized() {
        let mock = MockOpenIo::default();
        let branch = BranchIo::new_main(mock.clone());
        branch.init().unwrap();
        assert!(branch.init().is_err());
    }

    #[test]
    fn create_stage_file_after_staged() {
        let mock = MockOpenIo::default();
        mock.write("./src/main.rs", b"fn main(){println(\"hello\")}")
            .unwrap();
        mock.write("./src/test.rs", b"test").unwrap();
        let branch = BranchIo::new_main(mock);
        branch.stage("./src").unwrap();
        let stage = branch.stage.read_tree().unwrap().unwrap();
        assert_eq!(
            stage.get(&FilePath::from_path("./src/main.rs")),
            Some(&ObjectHash::new(b"fn main(){println(\"hello\")}"))
        );
        assert_eq!(
            stage.get(&FilePath::from_path("./src/test.rs")),
            Some(&ObjectHash::new(b"test"))
        );
    }

    #[test]
    fn create_objs_after_staged() {
        let mock = MockOpenIo::default();
        mock.write("./src/main.rs", b"fn main(){println(\"hello\")}")
            .unwrap();
        mock.write("./src/test.rs", b"test").unwrap();
        let branch = BranchIo::new_main(mock.clone());
        branch.stage("./src").unwrap();
        let hash1 = ObjectHash::new(b"fn main(){println(\"hello\")}");
        let hash2 = ObjectHash::new(b"test");
        assert!(mock
            .read_to_end(&FilePath(format!(".meltos/objects/{hash1}")))
            .is_ok());
        assert!(mock
            .read_to_end(&FilePath(format!(".meltos/objects/{hash2}")))
            .is_ok());
    }
}
