use serde::{Deserialize, Serialize};

use crate::user::UserId;







/// ルームユーザーからオーナーに向けて送信される任意のリクエスト
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserRequest {
    /// リクエストを送信したユーザーのID
    pub from: UserId,

    /// リクエスト本文
    pub data: RequestMessage,
}



/// ルームユーザーからオーナー向けて送信されるリクエストデータ
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestMessage {
    /// リクエストの種別
    pub name: String,

    /// リクエストのデータ
    /// Json形式で表され、リクエストにより構造が異なる
    /// オーナーはまず`request_type`を元に`data`を適切な構造にデシリアライズしてリクエストを解析する必要がある。
    pub data: String,
}