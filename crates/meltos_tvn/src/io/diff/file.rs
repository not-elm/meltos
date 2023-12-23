use similar::{DiffOp, TextDiff};

use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::object::ObjIo;
use crate::object::ObjHash;

#[derive(Debug, Clone)]
pub struct FileDiff {
    old: String,
    new: String,
}

impl FileDiff {
    #[inline]
    pub fn from_strings(old: impl Into<String>, new: impl Into<String>) -> Self {
        Self {
            old: old.into(),
            new: new.into(),
        }
    }

    pub fn from_obj_hashes<Fs>(
        fs: Fs,
        lhs: &ObjHash,
        rhs: &ObjHash,
    ) -> error::Result<Option<Self>>
        where
            Fs: FileSystem
    {
        if lhs == rhs {
            Ok(None)
        } else {
            let obj = ObjIo::new(fs);
            let lhs_file = obj.read_to_file(lhs)?;
            let rhs_file = obj.read_to_file(rhs)?;
            Ok(Some(Self {
                old: String::from_utf8(lhs_file.0)?,
                new: String::from_utf8(rhs_file.0)?,
            }))
        }
    }

    pub fn diff(&self) -> TextDiff<str> {
        TextDiff::from_lines(&self.old, &self.new)
    }

    pub fn diff_ops(&self) -> Vec<DiffOp> {
        self.diff()
            .ops()
            .iter()
            .filter(|op| {
                !matches!(
                    op,
                    &&DiffOp::Equal {
                        new_index: _,
                        old_index: _,
                        len: _,
                    }
                )
            })
            .copied()
            .collect()
    }
}
