use meltos_core::impl_string_new_type;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CommitMessage(String);
impl_string_new_type!(CommitMessage);