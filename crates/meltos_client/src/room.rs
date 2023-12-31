use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use meltos::schema::discussion::global::{Create, Created};
use meltos_tvn::branch::BranchName;
use meltos_tvn::operation::Operations;
use meltos_tvn::operation::push::Pushable;

use crate::config::SessionConfigs;
use crate::error::JsResult;
use crate::http::HttpClient;
use crate::room::in_memory::MemFs;

pub mod discussion;
pub mod file_system;
mod in_memory;

#[wasm_bindgen(getter_with_clone)]
pub struct RoomClient {
    client: HttpClient,
    operations: Operations<MemFs>,
}

const BASE: &str = "http://127.0.0.1:3000";

#[wasm_bindgen]
impl RoomClient {
    #[wasm_bindgen(constructor)]
    pub fn new(configs: SessionConfigs) -> RoomClient {
        Self {
            operations: Operations::new(
                BranchName::from(configs.user_id.to_string()),
                MemFs::new(),
            ),
            client: HttpClient::new(BASE, configs),
        }
    }

    //
    // #[inline(always)]
    // pub fn merge(&self, source: BranchName) -> Result<MergedStatus, JsValue> {
    //     let result = self
    //         .operations
    //         .merge
    //         .execute(source, BranchName::from(self.configs().user_id.to_string()));
    //     to_js_result(result)
    // }
    //
    // #[inline(always)]
    // pub fn stage(&self, workspace_path: &str) -> Result<(), JsValue> {
    //     to_js_result(self.operations.stage.execute(workspace_path))
    // }
    //
    // #[inline(always)]
    // pub fn commit(&self, commit_text: String) -> Result<CommitHash, JsValue> {
    //     to_js_result(self.operations.commit.execute(commit_text))
    // }
    //
    // #[inline(always)]
    // pub async fn push(&mut self) -> Result<(), JsValue> {
    //     to_js_result(self.operations.push.execute(&mut self.client).await)
    // }

    #[inline(always)]
    pub async fn create_discussion(&self, title: String) -> Result<Created, JsValue> {
        let created = self.client.create_discussion(&Create::new(title)).await?;
        Ok(created)
    }

    #[inline(always)]
    pub fn configs(&self) -> SessionConfigs {
        self.client.configs().clone()
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[macro_export]
macro_rules! console_log {
    () => {
       $crate::room::log("\n")
    };
    ($($arg:tt)*) => {{
        $crate::room::log(&format!($($arg)*));
    }};
}



#[wasm_bindgen]
pub struct TvnClient {
    operations: Operations<MemFs>,
    http: HttpClient,
}


#[wasm_bindgen]
impl TvnClient {
    #[wasm_bindgen(constructor)]
    pub fn wasm_new(
        branch_name: String,
        fs: MemFs,
        base_uri: String,
        session_configs: SessionConfigs,
    ) -> Self {
        Self {
            operations: Operations::new(BranchName::from(branch_name), fs),
            http: HttpClient::new(base_uri, session_configs),
        }
    }

    pub fn init(&self) -> JsResult {
        self.operations.init.execute()?;
        Ok(())
    }


    #[inline]
    pub async fn fetch(&self) -> JsResult {
        let bundle = self.http.fetch().await?;
        self.operations.patch.execute(&bundle)?;
        Ok(())
    }


    pub fn stage(&self, path: String) -> JsResult{
        self.operations.stage.execute(&path)?;
        Ok(())
    }

    pub fn commit(&self, commit_text: String) -> JsResult{
        self.operations.commit.execute(commit_text)?;
        Ok(())
    }

    pub async fn push(&mut self) -> JsResult{
        let bundle = self.operations.push.create_push_bundle()?;
        self.http.push(bundle).await?;
        Ok(())
    }
}

