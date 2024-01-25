mod error;
#[cfg(target_arch = "wasm32")]
pub mod file_system;
#[cfg(target_arch = "wasm32")]
pub mod tvc;
mod js_vec;

// #[cfg(test)]
// pub mod tests {
//     use crate::file_system::node::NodeFileSystem;
//
//     pub fn workspace_folder() -> String {
//         "D://tmp".to_string()
//     }
//
//     pub fn node_fs() -> NodeFileSystem {
//         NodeFileSystem::new(workspace_folder())
//     }
// }
