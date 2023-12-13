use meltos_tvc::branch::BranchIo;
use meltos_tvc::io::file::FileOpen;

fn main() {
    let io = BranchIo::new_main(FileOpen);
    io.stage("./branch").unwrap();
    io.commit().unwrap();
}
