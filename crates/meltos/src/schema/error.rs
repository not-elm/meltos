use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::discussion::id::DiscussionId;
use crate::discussion::message::MessageId;

/// meltos serverに対するリクエストが失敗した場合のエラーレスポンスボディを表します。
///
/// この構造体は全てのエラーの共通のフィールドを表しており、エラーの種別によってはさらに追加のフィールドが設定されている場合があります。
#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ErrorResponseBodyBase {
    /// エラーの大分類
    pub category: String,

    /// エラーの種別
    pub error_name: String,

    /// エラーのメッセージ
    pub message: String,
}

/// リクエスト時に送信したBundleサイズがサーバの上限値を超えた場合のレスポンスボディを表します。
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub struct ExceedBundleSizeBody {
    #[serde(flatten)]
    pub base: ErrorResponseBodyBase,

    /// サーバ側で設定されたバンドルサイズの上限値
    pub limit_bundle_size: usize,

    /// リクエスト時に送信されたバンドルのサイズ
    pub actual_bundle_size: usize,
}

/// RoomのTvcリポジトリのサイズが上限値を超えた場合
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub struct ExceedRepositorySizeBody {
    #[serde(flatten)]
    pub base: ErrorResponseBodyBase,

    /// サーバ側で設定されたバンドルサイズの上限値
    pub limit_tvc_repository_size: usize,

    /// リクエスト時に送信されたバンドルのサイズ
    pub actual_size: usize,
}

/// ルームの定員に達したことを表します。
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub struct ReachedCapacityBody {
    #[serde(flatten)]
    pub base: ErrorResponseBodyBase,

    /// ルームの定員
    pub capacity: u64,
}

/// リクエスト時に送信した[`DiscussionId`](crate::discussion::id::DiscussionId)に対応するディスカッションが見つからない場合のレスポンスボディを表します。。
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub struct DiscussionNotExistsBody {
    #[serde(flatten)]
    pub base: ErrorResponseBodyBase,

    /// リクエスト時に送信されたディスカッションId
    pub discussion_id: DiscussionId,
}

/// リクエスト時に送信した[`MessageId`](crate::discussion::message::MessageId)に対応するメッセージが見つからない場合のレスポンスボディを表します。。
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub struct MessageNotExistsBody {
    #[serde(flatten)]
    pub base: ErrorResponseBodyBase,

    /// リクエスト時に送信された返信先のメッセージId
    pub message_id: MessageId,
}

/// サーバ内のTvc操作が失敗した場合に発生します。
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub struct FailedTvcBody(ErrorResponseBodyBase);
