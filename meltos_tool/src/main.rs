use meltos_tvn::branch::BranchIo;
use meltos_tvn::io::file::FileOpen;

fn main() {
    let io = BranchIo::new_main(FileOpen);
    io.stage("./branch").unwrap();
    io.commit().unwrap();
}
