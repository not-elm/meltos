mod error;

pub mod file_system;

pub mod tvc;
mod js_vec;
mod vscode;

#[cfg(test)]
pub mod tests {
    use crate::file_system::node::NodeFileSystem;

    pub fn workspace_folder() -> String {
        "D://tmp".to_string()
    }

    pub fn node_fs() -> NodeFileSystem {
        NodeFileSystem::new(workspace_folder())
    }
}
