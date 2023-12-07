#[derive(Debug, clap::Args)]
pub struct New {
    file_path: String,
    line_no: usize,
}


impl New {
    pub fn run(self) {}
}
