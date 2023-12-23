use std::ops::Range;

use similar::{ChangeTag, DiffOp};

use crate::io::diff::file::FileDiff;

pub struct Conflict {
    pub lhs_text: String,
    pub rhs_text: String,
}

//
// pub fn all_conflicts(lhs: &FileDiff, rhs: &FileDiff) -> Vec<Conflict> {
//     let rhs_ops = rhs.diff_ops();
//     let mut conflicts = Vec::with_capacity(rhs_ops.len());
//
//     for lhs_op in lhs.diff_ops() {
//         if let Some(rhs_op) = find_conflict(&lhs_op, &rhs_ops) {
//             conflicts.push(Conflict {
//                 old_range: conflicted_range(lhs_op.old_range(), rhs_op.old_range()),
//                 lhs_tag: lhs_op,
//                 rhs_tag: rhs_op,
//                 lhs_text: lhs.diff().new_slices()[lhs_op.new_range()].join("\n"),
//                 rhs_text: rhs.diff().new_slices()[rhs_op.new_range()].join("\n"),
//             });
//         }
//     }
//     conflicts
// }
//
// fn find_conflict(lhs_op: &DiffOp, rhs_ops: &[DiffOp]) -> Option<DiffOp> {
//     for rhs_op in rhs_ops.iter() {
//         let l = lhs_op.old_range();
//         let r = rhs_op.old_range();
//         if r.start <= l.end && l.start <= r.end {
//             return Some(*rhs_op);
//         }
//     }
//     None
// }
//
//
// fn conflicted_range(lhs: Range<usize>, rhs: Range<usize>) -> Range<usize> {
//     let start = lhs.start.max(rhs.start);
//     let end = lhs.end.min(rhs.end);
//     start..end
// }
//
//
// #[cfg(test)]
// mod tests {
//     use crate::io::diff::conflict::all_conflicts;
//     use crate::io::diff::file::FileDiff;
//
//     #[test]
//     fn conflict_text() {
//         let old = "HELLO\n";
//         let new = "HELLO\nINSERT";
//         let diff1 = FileDiff::from_strings(old, new);
//         let diff2 = FileDiff::from_strings(old, "HELLO\nUPDATE");
//         let conflicts = all_conflicts(&diff1, &diff2);
//         assert_eq!(conflicts.len(), 1);
//         assert_eq!(conflicts[0].old_range, 1..1);
//         assert_eq!(conflicts[0].lhs_text, "INSERT");
//         assert_eq!(conflicts[0].rhs_text, "UPDATE");
//     }
// }
