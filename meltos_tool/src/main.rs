use std::fs::File;
use meltos_tvc::branch::BranchIo;
use meltos_tvc::io::file::FileOpen;


fn main() {
    let io = BranchIo::<FileOpen, File>::default();
    io.stage("./branch").unwrap();

}
