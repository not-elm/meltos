use std::ops::Range;

use similar::{DiffOp, TextDiff};

use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::object::ObjIo;
use crate::object::ObjHash;

#[derive(Debug, Clone)]
pub struct ObjDiff {
    old: String,
    new: String,
}

impl ObjDiff {
    #[inline]
    pub fn from_strings(old: impl Into<String>, new: impl Into<String>) -> Self {
        Self {
            old: old.into(),
            new: new.into(),
        }
    }


    pub fn from_obj_hashes<Fs, Io>(fs: Fs, lhs: &ObjHash, rhs: &ObjHash) -> error::Result<Option<Self>>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
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

    pub fn occurred_all_conflicts<'l, 'r>(&'l self, other: &'r ObjDiff) -> Vec<Conflict<'l, 'r>> {
        let other_ops = other.diff_ops();
        let mut ops = Vec::with_capacity(other_ops.len());
        for lhs_op in self.diff_ops() {
            if let Some(rhs_op) = self.occurred_conflict(&lhs_op, &other_ops) {
                ops.push(Conflict {
                    old_range: conflicted_range(lhs_op.old_range(), rhs_op.old_range()),
                    lhs_op,
                    rhs_op,
                    lhs_diff: self.text_diff(),
                    rhs_diff: other.text_diff(),
                });
            }
        }
        ops
    }


    pub fn diff_ops(&self) -> Vec<DiffOp> {
        self.text_diff()
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


    pub fn text_diff(&self) -> TextDiff<str> {
        TextDiff::from_lines(&self.old, &self.new)
    }


    fn occurred_conflict(&self, lhs_op: &DiffOp, rhs_ops: &[DiffOp]) -> Option<DiffOp> {
        for rhs_op in rhs_ops.iter() {
            let l = lhs_op.old_range();
            let r = rhs_op.old_range();
            if r.start <= l.end && l.start <= r.end {
                return Some(*rhs_op);
            }
        }
        None
    }
}

fn conflicted_range(lhs: Range<usize>, rhs: Range<usize>) -> Range<usize> {
    let start = lhs.start.max(rhs.start);
    let end = lhs.end.min(rhs.end);
    start..end
}


pub struct Conflict<'l, 'r> {
    pub old_range: Range<usize>,
    pub lhs_op: DiffOp,
    pub rhs_op: DiffOp,
    pub lhs_diff: TextDiff<'l, 'l, 'l, str>,
    pub rhs_diff: TextDiff<'r, 'r, 'r, str>,
}

impl<'l, 'r> Conflict<'l, 'r> {
    pub fn conflict_lhs_text(&self) -> &[&'l str] {
        &self.lhs_diff.new_slices()[self.lhs_op.new_range()]
    }

    pub fn conflict_rhs_text(&self) -> &[&'r str] {
        &self.rhs_diff.new_slices()[self.rhs_op.new_range()]
    }
}


#[cfg(test)]
mod tests {
    use crate::io::diff::ObjDiff;

    #[test]
    fn conflict_text() {
        let old = "HELLO\n";
        let new = "HELLO\nINSERT";
        let diff1 = ObjDiff::from_strings(old, new);
        let diff2 = ObjDiff::from_strings(old, "HELLO\nUPDATE");
        let conflicts = diff1.occurred_all_conflicts(&diff2);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].old_range, 1..1);
        assert_eq!(conflicts[0].conflict_lhs_text(), &["INSERT"]);
        assert_eq!(conflicts[0].conflict_rhs_text(), &["UPDATE"]);
    }
}
