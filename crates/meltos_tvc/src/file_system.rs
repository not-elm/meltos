use std::io::ErrorKind;
use std::path::Path;
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use meltos_util::impl_string_new_type;

pub mod mock;
pub mod std_fs;

#[wasm_bindgen(getter_with_clone)]
#[derive(Eq, PartialEq, Debug, Copy, Clone, Hash)]
pub struct Stat {
    pub ty: StatType,

    /// ファイルの場合、ファイルサイズ
    /// ディレクトリの場合、エントリ数
    pub size: u64,

    /// ファイルが作成された時点におけるUTCの基準時刻からの経過時間（秒）
    pub create_time: u64,

    /// ファイルが更新された時点におけるUTCの基準時刻からの経過時間（秒）
    pub update_time: u64,
}

impl Stat {
    pub fn new(ty: StatType, size: u64) -> Self {
        let time = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            ty,
            size,
            create_time: time,
            update_time: time,
        }
    }

    #[inline]
    pub fn is_file(&self) -> bool {
        matches!(self.ty, StatType::File)
    }

    #[inline]
    pub fn is_dir(&self) -> bool {
        matches!(self.ty, StatType::Dir)
    }
}

#[wasm_bindgen]
#[derive(Debug, Eq, Copy, Clone, PartialEq, Hash)]
pub enum StatType {
    File,
    Dir,
}

pub trait FileSystem {
    /// エントリのStatを取得します。
    ///
    /// パスが存在しない場合、`None`が返されます。
    fn stat(&self, path: &str) -> std::io::Result<Option<Stat>>;

    /// 対象のパスにファイルを書き込みます。
    ///
    /// ファイルが存在しない場合は新規作成されます。
    /// 親ディレクトリが存在しない場合、親となるディレクトリを全て作成します。
    fn write_file(&self, path: &str, buf: &[u8]) -> std::io::Result<()>;

    /// ディレクトリを作成します。
    ///
    /// 親ディレクトリが存在しない場合、再帰的に作成します。
    fn create_dir(&self, path: &str) -> std::io::Result<()>;

    /// ファイルバッファを読み込みます。
    ///
    /// 対象のパスにファイルが存在しない場合、`None`が返されます。
    fn read_file(&self, path: &str) -> std::io::Result<Option<Vec<u8>>>;

    /// ディレクトリ内のエントリパスをすべて取得します。
    fn read_dir(&self, path: &str) -> std::io::Result<Option<Vec<String>>>;

    /// 指定したパスがディレクトリの場合、子孫となるファイルパスを全て返します。
    /// ファイルの場合、そのファイルパスを返します。
    ///
    /// ファイルパスはファイルシステムのルートからの相対パスになります。
    fn all_files_in(&self, path: &str) -> std::io::Result<Vec<String>>;

    ///　エントリを強制的に削除します。
    ///
    /// ディレクトリの場合、子孫も削除されます。
    fn delete(&self, path: &str) -> std::io::Result<()>;

    /// ファイルバッファを読み込みます。
    ///
    /// ファイルが存在しない場合`Error`が返されます。
    fn try_read_file(&self, path: &str) -> std::io::Result<Vec<u8>> {
        self.read_file(path).and_then(|buf| {
            match buf {
                Some(buf) => Ok(buf),
                None => {
                    Err(std::io::Error::new(
                        ErrorKind::NotFound,
                        format!("not found file path = {path}"),
                    ))
                }
            }
        })
    }

    /// ファイルバッファを読み込みます。
    ///
    /// ファイルが存在しない場合`Error`が返されます。
    fn try_read_dir(&self, path: &str) -> std::io::Result<Vec<String>> {
        self.read_dir(path).and_then(|buf| {
            match buf {
                Some(files) => Ok(files),
                None => {
                    Err(std::io::Error::new(
                        ErrorKind::NotFound,
                        format!("not found dir path = {path}"),
                    ))
                }
            }
        })
    }

    /// TVCのデータ構造が既に存在するかを検査します。
    fn project_already_initialized(&self) -> std::io::Result<bool> {
        let files = self.all_files_in("./.meltos")?;
        Ok(!files.is_empty())
    }
}

impl FileSystem for Box<dyn FileSystem> {
    fn stat(&self, path: &str) -> std::io::Result<Option<Stat>> {
        self.as_ref().stat(path)
    }

    fn write_file(&self, path: &str, buf: &[u8]) -> std::io::Result<()> {
        self.as_ref().write_file(path, buf)
    }

    fn create_dir(&self, path: &str) -> std::io::Result<()> {
        self.as_ref().create_dir(path)
    }

    fn read_file(&self, path: &str) -> std::io::Result<Option<Vec<u8>>> {
        self.as_ref().read_file(path)
    }

    fn read_dir(&self, path: &str) -> std::io::Result<Option<Vec<String>>> {
        self.as_ref().read_dir(path)
    }

    fn all_files_in(&self, path: &str) -> std::io::Result<Vec<String>> {
        self.as_ref().all_files_in(path)
    }

    fn delete(&self, path: &str) -> std::io::Result<()> {
        self.as_ref().delete(path)
    }
}

#[wasm_bindgen(getter_with_clone)]
#[repr(transparent)]
#[derive(Eq, PartialEq, Debug, Clone, Hash, Serialize, Deserialize, Ord, PartialOrd)]
pub struct FilePath(pub String);
impl_string_new_type!(FilePath);

impl FilePath {
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        Self(path.as_ref().to_str().unwrap().to_string())
    }
}

impl AsRef<Path> for FilePath {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl AsRef<String> for FilePath {
    fn as_ref(&self) -> &String {
        &self.0
    }
}
