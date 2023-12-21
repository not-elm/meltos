use similar::{Change, ChangeTag, DiffableStr};

use crate::io::diff::conflict::Conflict;
use crate::io::diff::file::FileDiff;

pub fn merge(lhs: &FileDiff, rhs: &FileDiff) -> String {
    let rhs_diff = rhs.diff();

    let mut original = rhs_diff
        .old_slices()
        .iter()
        .map(|line| line.to_string())
        .collect();
    merge_(&mut original, lhs, rhs);
    original.join("")
}


pub fn merge_(orginal: &mut Vec<String>, lhs: &FileDiff, rhs: &FileDiff) -> Vec<Conflict> {
    let lhs_diff = lhs.diff();
    let rhs_diff = rhs.diff();
    let mut lhs_changes = MergeFile::new(lhs_diff.iter_all_changes().collect());
    let mut rhs_changes = MergeFile::new(rhs_diff.iter_all_changes().collect());
    let mut index = 0;
    let mut conflicts = Vec::new();
    loop {
        if lhs_changes.is_end() && rhs_changes.is_end() {
            break;
        }

        if lhs_changes.file_line == rhs_changes.file_line {
            rhs_changes.next(orginal);
        }
    }

    conflicts
}


struct MergeFile<'a> {
    file_line: usize,
    index: usize,
    changes: Vec<Change<&'a str>>,
}


impl<'a> MergeFile<'a> {
    pub fn new(changes: Vec<Change<&'a str>>) -> MergeFile<'a> {
        Self {
            file_line: 0,
            index: 0,
            changes,
        }
    }
    pub fn is_end(&self) -> bool {
        self.changes.len() == self.file_line
    }


    pub fn changed(&self) -> bool {
        !matches!(self.tag(), ChangeTag::Equal)
    }


    pub fn tag(&self) -> ChangeTag {
        let change = self.changes[self.index];
        change.tag()
    }

    pub fn next(&mut self, original: &mut Vec<String>) -> bool {
        if self.is_end() {
            return false;
        }

        let change = self.changes[self.index];
        match change.tag() {
            ChangeTag::Equal => {
                self.file_line += 1;
                return false;
            }
            ChangeTag::Delete => {
                original.remove(self.file_line);
            }
            ChangeTag::Insert => {
                if original.len() <= self.file_line {
                    original.push(change.to_string());
                    self.file_line += 1;
                } else {
                    original.insert(self.file_line, change.to_string());
                    self.file_line += 2;
                }
            }
        }
        self.index += 1;
        false
    }
}


impl MergeFile {}

#[cfg(test)]
mod tests {
    use crate::io::diff::file::FileDiff;
    use crate::io::diff::merge::merge;

    #[test]
    fn simple_inserted() {
        const ORIGINAL: &str = "LINE0\nLINE1\nLINE2";
        let diff1 = FileDiff::from_strings(
            ORIGINAL,
            "LINE0\nLINE1\nLINE2\nLINE3",
        );
        let diff2 = FileDiff::from_strings(
            ORIGINAL,
            "LINE0\nLINE3\nLINE1\nLINE2",
        );
        let merged = merge(&diff1, &diff2);
        assert_eq!(merged, "LINE0\nLINE3\nLINE1\nLINE2\nLINE3\n");
    }


    #[test]
    fn merge2() {
        const ORIGINAL: &str = "hello\nworld\ntext";
        let diff1 = FileDiff::from_strings(
            ORIGINAL,
            "hello\ntext",
        );
        let diff2 = FileDiff::from_strings(
            ORIGINAL,
            "hello\nworld\nyes\ntext",
        );
        let merged = merge(&diff1, &diff2);
        assert_eq!(merged, "hello\nyes\ntext");
    }


    #[test]
    fn merge3() {
        const ORIGINAL: &str = "hello1\nworld2\nrust3\ntext4";
        let diff1 = FileDiff::from_strings(
            ORIGINAL,
            "hello1\ntext4",
        );
        let diff2 = FileDiff::from_strings(
            ORIGINAL,
            "hello1\nworld2\nyes3\ntext4",
        );
        let merged = merge(&diff1, &diff2);
        assert_eq!(merged, "hello1\nyes3\ntext4");
    }


    #[test]
    fn merge4() {
        const ORIGINAL: &str = "hello1\nworld2\nrust3\ntext4";
        let diff1 = FileDiff::from_strings(
            ORIGINAL,
            "hello1\nstring5\ntext4",
        );
        let diff2 = FileDiff::from_strings(
            ORIGINAL,
            "hello1\nworld2\nyes3\ntext4",
        );
        let merged = merge(&diff1, &diff2);
        assert_eq!(merged, "hello1\nyes3\nstring5text4");
    }
}