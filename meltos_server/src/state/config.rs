use config::Config;
use serde::Deserialize;

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct AppConfigs {
    /// openやpushの際に渡されるbundleの最大サイズ
    /// objsのバッファの合計サイズを元に計算される
    pub limit_bundle_size: usize,

    /// roomの最大生存期間
    /// ユーザーがroomをopenする際にlifetime_secsを指定していない、またはこの上限値を超えた場合場合はこの値が反映される。
    pub room_limit_life_time_sec: u64,

    /// TVCのリポジトリの最大サイズ
    /// bundleがpushされる際にリポジトリサイズが許容値を超えないかを確認するために使用される。
    /// リポジトリサイズは.meltos/objs内のバッファサイズの合計値
    pub limit_tvc_repository_size: usize,

    /// ルームの定員の上限値
    /// ユーザーがroomをopenする際にuser_limitsを指定していない、またはこの上限値を超えた場合はこの値が反映される
    pub max_user_limits: u64,
}

impl Default for AppConfigs {
    fn default() -> Self {
        Config::builder()
            .add_source(config::File::with_name(&source()))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap()
    }
}

fn source() -> String {
    #[cfg(not(test))]
        let config_file_name = "Settings.toml";
    #[cfg(test)]
        let config_file_name = "SettingsTest.toml";

    if std::fs::read_dir("meltos_server").is_ok() {
        // meltos_project直下から実行されている場合
        format!("./meltos_server/{config_file_name}")
    } else {
        config_file_name.to_string()
    }
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


    #[test]
    fn it_read_max_user_limits() {
        let config = AppConfigs::default();
        assert_eq!(config.max_user_limits, 100);
    }
}
