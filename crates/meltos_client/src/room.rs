use std::fmt::{Debug, Display};

use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use meltos::room::RoomId;
use meltos::user::UserId;
use meltos_tvn::branch::BranchName;
use meltos_tvn::file_system::file::StdFileSystem;
use meltos_tvn::object::commit::CommitHash;
use meltos_tvn::object::local_commits::LocalCommitsObj;
use meltos_tvn::operation::merge::MergedStatus;
use meltos_tvn::operation::Operations;

use crate::config::{SessionConfigs, SessionConfigsIo};
use crate::config::tmp_file::TmpSessionConfigsIo;
use crate::http::HttpClient;

pub mod discussion;
mod file_system;

#[wasm_bindgen(getter_with_clone)]
pub struct RoomClient {
    client: HttpClient,
    operations: Operations<StdFileSystem>,
}

const BASE: &str = "http://127.0.0.1:3000";

#[wasm_bindgen]
impl RoomClient {
    #[wasm_bindgen(constructor)]
    pub fn new(configs: SessionConfigs) -> RoomClient {
        Self {
            operations: Operations::new(BranchName::from(configs.user_id.to_string()), StdFileSystem),
            client: HttpClient::new(BASE, configs),
        }
    }


    #[inline]
    pub async fn fetch(&self) -> Result<(), JsValue> {
        let bundle = to_js_result(self.client.fetch().await)?;
        to_js_result(self.operations.patch.execute(&bundle))?;
        Ok(())
    }

    #[inline(always)]
    pub fn merge(&self, source: BranchName) -> Result<MergedStatus, JsValue> {
        let result = self
            .operations
            .merge
            .execute(source, BranchName::from(self.configs().user_id.to_string()));
        to_js_result(result)
    }


    #[inline(always)]
    pub fn stage(&self, workspace_path: &str) -> Result<(), JsValue> {
        to_js_result(self.operations.stage.execute(workspace_path))
    }

    #[inline(always)]
    pub fn commit(&self, commit_text: String) -> Result<CommitHash, JsValue> {
        to_js_result(self.operations.commit.execute(commit_text))
    }


    #[inline(always)]
    pub async fn push(&mut self) -> Result<(), JsValue> {
        to_js_result(self.operations.push.execute(&mut self.client).await)
    }

    #[inline(always)]
    pub fn configs(&self) -> SessionConfigs {
        self.client.configs().clone()
    }
}


#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub async fn open_room(user_id: Option<String>) -> Result<RoomClient, JsValue> {
    let operations = Operations::new_main(StdFileSystem);
    to_js_result(operations.init.execute())?;
    let bundle = to_js_result(operations.bundle.create())?;
    to_js_result(operations
        .local_commits
        .write(&LocalCommitsObj::default()))?;
    let client = to_js_result(HttpClient::open(BASE, bundle, user_id.map(UserId::from)).await)?;
    to_js_result(TmpSessionConfigsIo.save(client.configs().clone()).await)?;

    Ok(RoomClient {
        client,
        operations,
    })
}


#[wasm_bindgen]
pub async fn join(
    room_id: String,
    user_id: Option<String>,
) -> Result<RoomClient, JsValue> {
    let (client, bundle) = to_js_result(HttpClient::join(BASE, RoomId(room_id.clone()), user_id.map(UserId::from)).await)?;
    let configs = client.configs();
    to_js_result(TmpSessionConfigsIo.save(configs.clone()).await)?;

    let branch_name = BranchName::from(configs.user_id.to_string());
    let operations = Operations::new(branch_name.clone(), StdFileSystem);
    to_js_result(operations.save.execute(bundle))?;
    to_js_result(operations.checkout.execute(&branch_name))?;
    to_js_result(operations.unzip.execute(&branch_name))?;

    Ok(RoomClient {
        client,
        operations,
    })
}


#[inline]
fn to_js_result<Out: Debug, D: Display + Debug>(result: Result<Out, D>) -> Result<Out, JsValue> {
    log(format!("{result:?}").as_str());
    match result {
        Ok(out) => Ok(out),
        Err(e) => Err(JsValue::from_str(&e.to_string()))
    }
}