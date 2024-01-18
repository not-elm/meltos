pub mod file_system;
mod error;

pub mod tvc;


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