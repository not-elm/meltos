use similar::{DiffOp, DiffableStr};

use crate::io::diff::conflict::Conflict;
use crate::io::diff::file::FileDiff;

pub fn merge(lhs: &FileDiff, rhs: &FileDiff) -> String {
    let merged = merge_(lhs, rhs);
    merged.join("")
}


pub fn merge_(lhs: &FileDiff, rhs: &FileDiff) -> Vec<String> {
    let lhs_diff = lhs.diff();
    let rhs_diff = rhs.diff();
    let mut lhs_op = lhs_diff.ops().into_iter();
    lhs_op.next();
    let mut texts = Vec::new();

    let mut lhs_changes = MergeFile {
        texts: lhs_diff.new_slices(),
        ops: lhs_diff.ops().into_iter().copied(),
    };
    let mut rhs_changes = MergeFile {
        texts: rhs_diff.new_slices(),
        ops: rhs_diff.ops().into_iter().copied(),
    };
    let merges = Merges::new(lhs_changes, rhs_changes);
    for status in merges {
        match status {
            MergeStatus::Success(status) => {
                match status {
                    Status::Equal {
                        text, ..
                    } => {
                        texts.push(text);
                    }
                    Status::Insert {
                        text, ..
                    } => {
                        texts.push(text);
                    }
                    Status::Delete {
                        ..
                    } => {}
                }
            }
            _ => {}
        }
    }


    texts
}


struct Merges<'l, 'r, L, R> {
    lhs: MergeFile<'l, L>,
    rhs: MergeFile<'r, R>,
    lhs_op: Option<Status>,
    rhs_op: Option<Status>,
}

impl<'l, 'r, L, R> Merges<'l, 'r, L, R>
where
    L: Iterator<Item = DiffOp>,
    R: Iterator<Item = DiffOp>,
{
    pub fn new(mut lhs: MergeFile<'l, L>, mut rhs: MergeFile<'r, R>) -> Merges<'l, 'r, L, R> {
        Self {
            rhs_op: rhs.next(),
            lhs_op: lhs.next(),
            rhs,
            lhs,
        }
    }
    fn side(&self, status: Status) -> MergeStatus {
        MergeStatus::Success(status)
    }
}

impl<'l, 'r, L, R> Iterator for Merges<'l, 'r, L, R>
where
    L: Iterator<Item = DiffOp>,
    R: Iterator<Item = DiffOp>,
{
    type Item = MergeStatus;

    fn next(&mut self) -> Option<Self::Item> {
        if self.lhs_op.is_none() && self.rhs_op.is_none() {
            return None;
        }
        if self.rhs_op.is_none() {
            let status = MergeStatus::Success(self.lhs_op.clone().unwrap());
            self.lhs_op = self.lhs.next();
            self.rhs_op = self.rhs.next();
            return Some(status);
        }
        if self.lhs_op.is_none() {
            let status = MergeStatus::Success(self.rhs_op.clone().unwrap());
            self.lhs_op = self.lhs.next();
            self.rhs_op = self.rhs.next();
            return Some(status);
        }
        let lhs_status = self.lhs_op.clone().unwrap();
        let rhs_status = self.rhs_op.clone().unwrap();
        if lhs_status.conflict(&rhs_status) {
            self.rhs_op = self.rhs.next();
            self.lhs_op = self.lhs.next();
            Some(MergeStatus::Conflicted)
        } else if lhs_status.ord(&rhs_status) {
            self.lhs_op = self.lhs.next();
            Some(MergeStatus::Success(lhs_status))
        } else {
            self.rhs_op = self.rhs.next();
            Some(MergeStatus::Success(rhs_status))
        }
    }
}


#[derive(Debug, Clone)]
enum MergeStatus {
    Success(Status),
    Conflicted,
}


#[derive(Debug, Clone)]
enum Status {
    Equal {
        start: usize,
        end: usize,
        text: String,
    },
    Insert {
        start: usize,
        end: usize,
        text: String,
    },
    Delete {
        start: usize,
        end: usize,
    },
}


impl Status {
    #[inline]
    pub fn ord(&self, rhs: &Status) -> bool {
        let l = self.range();
        let r = rhs.range();
        if l.start == r.start {
            l.end <= r.end
        } else {
            l.start < r.start
        }
    }

    #[inline]
    pub fn conflict(&self, rhs: &Status) -> bool {
        if self.is_equal() || rhs.is_equal() {
            return false;
        }
        let l = self.range();
        let r = rhs.range();
        r.start <= l.end && l.start <= r.end
    }

    fn is_equal(&self) -> bool {
        matches!(self, Self::Equal { .. })
    }

    pub fn range(&self) -> std::ops::Range<&usize> {
        match self {
            Status::Equal {
                start,
                end,
                ..
            } => start..end,
            Status::Insert {
                start,
                end,
                text: _,
            } => start..end,
            Status::Delete {
                start,
                end,
            } => start..end,
        }
    }
}

struct MergeFile<'a, L> {
    texts: &'a [&'a str],
    ops: L,
}


impl<'a, L> Iterator for MergeFile<'a, L>
where
    L: Iterator<Item = DiffOp>,
{
    type Item = Status;

    fn next(&mut self) -> Option<Self::Item> {
        match self.ops.next()? {
            DiffOp::Delete {
                old_index,
                old_len,
                new_index: _,
            } => {
                Some(Status::Delete {
                    start: old_index,
                    end: old_index + old_len,
                })
            }
            DiffOp::Insert {
                old_index,
                new_len,
                new_index,
            } => {
                Some(Status::Insert {
                    start: old_index,
                    end: old_index,
                    text: self.texts[new_index..new_index + new_len].join(""),
                })
            }
            DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => {
                Some(Status::Insert {
                    start: old_index,
                    end: old_index + old_len,
                    text: self.texts[new_index..new_index + new_len].join(""),
                })
            }
            DiffOp::Equal {
                old_index,
                len,
                new_index,
            } => {
                Some(Status::Equal {
                    start: old_index,
                    end: old_index + len,
                    text: self.texts[new_index..new_index + len].join(""),
                })
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use similar::{DiffOp, TextDiff};

    use crate::io::diff::file::FileDiff;
    use crate::io::diff::merge::merge;


    #[test]
    fn simple_inserted() {
        const ORIGINAL: &str = "LINE0\nLINE1\nLINE2";
        let diff1 = FileDiff::from_strings(ORIGINAL, "LINE0\nLINE1\nLINE2\nLINE3");
        let diff2 = FileDiff::from_strings(ORIGINAL, "LINE0\nLINE3\nLINE1\nLINE2");
        let merged = merge(&diff1, &diff2);
        assert_eq!(merged, "LINE0\nLINE3\nLINE1\nLINE2\nLINE3\n");
    }


    #[test]
    fn merge2() {
        const ORIGINAL: &str = "hello\nworld\ntext";
        let diff1 = FileDiff::from_strings(ORIGINAL, "hello\ntext");
        let diff2 = FileDiff::from_strings(ORIGINAL, "hello\nworld\nyes\ntext");
        let merged = merge(&diff1, &diff2);
        assert_eq!(merged, "hello\nyes\ntext");
    }


    #[test]
    fn merge3() {
        const ORIGINAL: &str = "hello1\nworld2\nrust3\ntext4";
        let diff1 = FileDiff::from_strings(ORIGINAL, "hello1\ntext4");
        let diff2 = FileDiff::from_strings(ORIGINAL, "hello1\nworld2\nyes3\ntext4");
        let merged = merge(&diff1, &diff2);
        assert_eq!(merged, "hello1\nyes3\ntext4");
    }


    #[test]
    fn merge4() {
        const ORIGINAL: &str = "hello1\nworld2\nrust3\ntext4";
        let diff1 = FileDiff::from_strings(ORIGINAL, "hello1\nstring5\ntext4");
        let diff2 = FileDiff::from_strings(ORIGINAL, "hello1\nworld2\nyes3\ntext4");
        let merged = merge(&diff1, &diff2);
        assert_eq!(merged, "hello1\nyes3\nstring5text4");
    }
}
