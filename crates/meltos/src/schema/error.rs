use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;


/// meltos serverに対するリクエストが失敗した場合のエラーレスポンスボディを表します。
///
/// この構造体は全てのエラーの共通のフィールドを表しており、エラーの種別によってはさらに追加のフィールドが設定されている場合があります。
#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ErrorResponseBodyBase {
    /// エラーの種別
    pub error_type: String,

    /// エラーのメッセージ
    pub message: String,
}


/// リクエスト時に送信したBundleサイズがサーバの上限値を超えた場合のレスポンスボディを表します。
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub struct ExceedBundleSizeBody{
    #[serde(flatten)]
    pub base: ErrorResponseBodyBase,

    /// サーバ側で設定されたバンドルサイズの上限値
    pub limit_bundle_size: usize,

    /// リクエスト時に送信されたバンドルのサイズ
    pub actual_bundle_size: usize
}