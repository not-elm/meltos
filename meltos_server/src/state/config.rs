use config::Config;
use serde::Deserialize;

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct AppConfigs {
    /// openやpushの際に渡されるbundleの最大サイズ
    /// objsのバッファの合計サイズを元に計算される
    pub limit_bundle_size: usize,

    /// roomの最大生存期間
    /// ユーザーがroomをopenする際にlife_time_secを指定していない場合はこの値が反映される
    /// ユーザーが指定していた場合、この設定値を超えていない場合はユーザーの生存期間が反映され、超えている場合は設定値が反映される。
    pub room_limit_life_time_sec: u64,

    /// TVCのリポジトリの最大サイズ
    /// bundleがpushされる際にリポジトリサイズが許容値を超えないかを確認するために使用される。
    /// リポジトリサイズは.meltos/objs内のバッファサイズの合計値
    pub limit_tvc_repository_size: usize,
}

impl Default for AppConfigs {
    fn default() -> Self {
        Config::builder()
            .add_source(config::File::with_name(source()))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap()
    }
}

fn source() -> &'static str {
    #[cfg(not(test))]
    return "./Settings.toml";
    #[cfg(test)]
    return "SettingsTest.toml";
}

#[cfg(test)]
mod tests {
    use crate::state::config::AppConfigs;

    #[test]
    fn it_read_limit_bundle_size() {
        let config = AppConfigs::default();
        assert_eq!(config.limit_bundle_size, 1024);
    }

    #[test]
    fn it_read_limit_life_time_sec() {
        let config = AppConfigs::default();
        assert_eq!(config.room_limit_life_time_sec, 86400);
    }

    #[test]
    fn it_read_limit_tvc_repository_size() {
        let config = AppConfigs::default();
        assert_eq!(config.limit_tvc_repository_size, 3072);
    }
}
