pub struct UiState {
    pub titles: Vec<&'static str>,
    pub index: usize,
}

impl UiState {
    pub fn step(&mut self, index: usize) {
        self.index = index;
    }

    pub fn next_tab(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous_tab(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}


impl Default for UiState {
    fn default() -> UiState {
        UiState {
            titles: vec!["Branch", "GlobalThreads", "BranchThreads", "FileThreads"],
            index: 0,
        }
    }
}
