use std::io::ErrorKind;

use meltos_util::impl_string_new_type;

use crate::commit::{CommitIo, CommitText};
use crate::io::{OpenIo, TvnIo};
use crate::now::NowIo;
use crate::object::{ObjectIo, ObjectMeta};
use crate::stage::StageIo;
use crate::tree::Tree;
use crate::workspace::WorkspaceIo;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
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
    stage: StageIo<Open, Io>,
    object: ObjectIo<Open, Io>,
    workspace: WorkspaceIo<Open, Io>,
    now: NowIo<Open, Io>,
    commit: CommitIo<Open, Io>,
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
            commit: CommitIo::new(branch_name, open),
        }
    }
}


impl<Open, Io> BranchIo<Open, Io>
    where
        Open: OpenIo<Io>,
        Io: std::io::Write + std::io::Read,
{
    pub fn init(&self) -> std::io::Result<()> {
        let mut now_tree = Tree::default();
        for meta in self.workspace.convert_to_objs(".")? {
            let meta = meta?;
            self.object.write(&meta.obj)?;
            now_tree.insert(meta.file_path, meta.obj.hash);
        }
        self.stage.write_tree(&now_tree)
    }


    pub fn unpack_project(&self) -> std::io::Result<()> {
        let Some(now_tree) = self.now.read_tree()? else {
            return Ok(());
        };
        for (path, hash) in now_tree.iter() {
            let obj = self.object.try_read(hash)?;
            self.workspace.unpack(path, &obj.buf)?;
        }
        Ok(())
    }

    pub fn stage(&self, workspace_path: &str) -> std::io::Result<()> {
        let mut stage_tree = self.stage.read_tree()?.unwrap_or_default();
        let now_tree = self.now.read_tree().ok().and_then(|now| now);
        for obj in self.workspace.convert_to_objs(workspace_path)? {
            self.stage_file(&mut stage_tree, &now_tree, obj?)?;
        }
        self.stage.write_tree(&stage_tree)?;
        Ok(())
    }

    pub fn commit(&self, commit_text: impl Into<CommitText>) -> std::io::Result<()> {
        let Some(stage_tree) = self.stage.read_tree()? else {
            return Err(std::io::Error::new(ErrorKind::NotFound, "no staged files"));
        };
        self.stage.reset()?;
        self.now.write_tree(&stage_tree)?;
        self.commit.commit(commit_text, stage_tree)?;
        Ok(())
    }


    fn stage_file(
        &self,
        stage: &mut Tree,
        now: &Option<Tree>,
        meta: ObjectMeta,
    ) -> std::io::Result<()> {
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
    use crate::io::{FilePath, OpenIo};
    use crate::io::mock::MockOpenIo;
    use crate::object::ObjectHash;

    #[test]
    fn init() {
        let mock = MockOpenIo::default();
        mock.write("./src/main.rs", b"bdadasjlgd").unwrap();
        mock.write("./test.rs", b"test").unwrap();
        let branch = BranchIo::new_main(mock.clone());
        branch.init().unwrap();

        assert!(&mock.read_to_end(&format!(".meltos/objects/{}", ObjectHash::new(b"bdadasjlgd"))).is_ok());
        assert!(&mock.read_to_end(&format!(".meltos/objects/{}", ObjectHash::new(b"test"))).is_ok());
        assert!(&mock.read_to_end(".meltos/branches/main/NOW").is_ok());
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
