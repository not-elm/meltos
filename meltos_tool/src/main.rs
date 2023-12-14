use meltos_tvn::branch::BranchIo;
use meltos_tvn::file_system::file::StdFileSystem;

fn main() {
    let io = BranchIo::new_main(StdFileSystem);
    // file_system.stage("./branch").unwrap();
    // file_system.commit("hello!").unwrap();
}
