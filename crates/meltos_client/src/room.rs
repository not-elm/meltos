use async_trait::async_trait;

use wasm_bindgen::prelude::wasm_bindgen;

use meltos::room::RoomId;
use meltos::user::UserId;
use meltos_tvn::branch::BranchName;
use meltos_tvn::io::bundle::Bundle;
use meltos_tvn::operation::merge::MergedStatus;
use meltos_tvn::operation::Operations;
use meltos_tvn::operation::push::Pushable;

use crate::config::SessionConfigs;
use crate::error::JsResult;
use crate::http::HttpClient;
use crate::room::in_memory::StorageFs;

pub mod discussion;
pub mod file_system;
mod in_memory;


const BASE: &str = "http://127.0.0.1:3000";


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
    operations: Operations<StorageFs>,
    branch_name: String,
}


#[wasm_bindgen]
impl TvnClient {
    #[wasm_bindgen(constructor)]
    pub fn wasm_new(
        branch_name: String,
        fs: StorageFs,
    ) -> Self {
        Self {
            operations: Operations::new(BranchName::from(branch_name.clone()), fs),
            branch_name,
        }
    }


    pub async fn open_room(&self, lifetime_sec: Option<u64>) -> JsResult<SessionConfigs> {
        self.operations.init.execute()?;

        let mut sender = OpenSender {
            user_id: Some(BranchName::owner().0),
            lifetime_sec,
        };

        let session_configs = self.operations.push.execute(&mut sender).await?;
        Ok(session_configs)
    }


    pub async fn join_room(&self, room_id: String, user_id: String) -> JsResult<SessionConfigs> {
        let (http, bundle) = HttpClient::join(
            BASE,
            RoomId(room_id),
            Some(UserId(user_id.clone())),
        ).await?;

        self.operations.save.execute(bundle)?;
        self.operations.checkout.execute(&BranchName(user_id))?;
        self.operations.unzip.execute(&BranchName(self.branch_name.clone()))?;

        Ok(http.configs().clone())
    }

    #[inline]
    pub async fn fetch(&self, session_config: SessionConfigs) -> JsResult {
        let http = HttpClient::new(BASE, session_config);
        let bundle = http.fetch().await?;
        self.operations.save.execute(bundle)?;
        Ok(())
    }

    pub fn stage(&self, path: String) -> JsResult {
        self.operations.stage.execute(&path)?;
        Ok(())
    }

    pub fn commit(&self, commit_text: String) -> JsResult {
        self.operations.commit.execute(commit_text)?;
        Ok(())
    }

    pub async fn push(&mut self, session_configs: SessionConfigs) -> JsResult {
        let mut sender = PushSender {
            session_configs
        };
        self.operations.push.execute(&mut sender).await?;
        Ok(())
    }

    pub async fn merge(&mut self, source: String) -> JsResult<MergedStatus> {
        let source = BranchName(source);
        let dist = BranchName(self.branch_name.clone());
        let status = self.operations.merge.execute(source, dist)?;
        Ok(status)
    }
}


struct OpenSender {
    user_id: Option<String>,
    lifetime_sec: Option<u64>,
}


#[async_trait(? Send)]
impl Pushable<SessionConfigs> for OpenSender {
    type Error = crate::error::Error;

    async fn push(&mut self, bundle: Bundle) -> Result<SessionConfigs, Self::Error> {
        let http = HttpClient::open(
            "http://localhost:3000",
            Some(bundle),
            self.user_id.clone().map(UserId::from),
            self.lifetime_sec,
        ).await?;
        console_log!("CONNECTED");
        Ok(http.configs().clone())
    }
}


struct PushSender {
    session_configs: SessionConfigs,
}


#[async_trait(? Send)]
impl Pushable<()> for PushSender {
    type Error = crate::error::Error;

    async fn push(&mut self, bundle: Bundle) -> Result<(), Self::Error> {
        let mut http = HttpClient::new(
            "http://localhost:3000",
            self.session_configs.clone(),
        );
        http.push(bundle).await?;

        Ok(())
    }
}

