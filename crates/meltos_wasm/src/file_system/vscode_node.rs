use wasm_bindgen::prelude::wasm_bindgen;

use meltos_tvc::file_system::{FileSystem, Stat};

#[wasm_bindgen]
extern "C" {
    #[derive(Debug, Clone)]
    pub type VscodeNodeFs;

    #[wasm_bindgen(method, js_name = allFilesIn)]
    pub fn all_files_in_api(this: &VscodeNodeFs, path: &str) -> Vec<String>;

    #[wasm_bindgen(method, js_name = writeFileApi)]
    pub fn write_file_api(this: &VscodeNodeFs, path: &str, buf: Vec<u8>);

    #[wasm_bindgen(method, js_name = readFileApi)]
    pub fn read_file_api(this: &VscodeNodeFs, path: &str) -> Option<Vec<u8>>;

    #[wasm_bindgen(method, js_name = createDirApi)]
    pub fn create_dir_api(this: &VscodeNodeFs, path: &str) -> Option<Vec<u8>>;

    #[wasm_bindgen(method, js_name = readDirApi)]
    pub fn read_dir_api(this: &VscodeNodeFs, path: &str) -> Option<Vec<String>>;

    #[wasm_bindgen(method, js_name = deleteApi)]
    pub fn delete_api(this: &VscodeNodeFs, path: &str);
}

impl FileSystem for VscodeNodeFs {
    fn stat(&self, _: &str) -> std::io::Result<Option<Stat>> {
        todo!()
    }

    fn write_file(&self, path: &str, buf: &[u8]) -> std::io::Result<()> {
        self.write_file_api(path, buf.to_vec());
        Ok(())
    }

    fn create_dir(&self, path: &str) -> std::io::Result<()> {
        self.create_dir_api(path);
        Ok(())
    }

    fn read_file(&self, path: &str) -> std::io::Result<Option<Vec<u8>>> {
        let buf = self.read_file_api(path);
        Ok(buf)
    }

    fn read_dir(&self, path: &str) -> std::io::Result<Option<Vec<String>>> {
        let entries = self.read_dir_api(path);
        Ok(entries)
    }

    fn all_files_in(&self, path: &str) -> std::io::Result<Vec<String>> {
        let files = self.all_files_in_api(path);
        Ok(files)
    }

    fn delete(&self, path: &str) -> std::io::Result<()> {
        self.delete_api(path);
        Ok(())
    }
}
