use serde::{Deserialize, Serialize};
use std::fs;
use chrono::{DateTime, Duration, Utc};

extern crate azure_devops_rust_lib;
use azure_devops_rust_lib::models::config::Config;

mod git;
mod wit;

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    pub organization: String,
    pub project: String,
    pub repository_id: String,
    pub pat: String,
    pub output_path: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // config.tomlを読み込む
    let contents = fs::read_to_string("config.toml").unwrap();
    let app_config: AppConfig = toml::from_str(&contents).unwrap();
    let config: Config = Config { organization: app_config.organization.clone(), project: app_config.project.clone(), repository_id: app_config.repository_id.clone(), pat: app_config.pat.clone() };

    // フィールド一覧を取得する
    wit::export_fields(&app_config, &config).await;

    // カテゴリー一覧を取得する
    wit::export_work_item_categories(&app_config, &config).await;

    // ワーク項目タイプ一覧を取得する
    wit::export_work_item_types(&app_config, &config).await;

    // ワーク項目ステート一覧を取得する
    wit::export_work_item_states(&app_config, &config).await;

    // ワーク項目エリアパス一覧を取得する
    wit::export_work_item_areas(&app_config, &config).await;

    // ワーク項目イテレーションパス一覧を取得する
    wit::export_work_item_iterations(&app_config, &config).await;

    // ワークアイテムの種類一覧を取得する(ClassificationNodes)
    wit::export_classification_nodes(&app_config, &config).await;

    // 取得開始日時を設定
    let now_utc: DateTime<Utc> = Utc::now() - Duration::days(300);

    // work_itemsのidを取得
    let ids = wit::get_work_items_ids(&config, now_utc).await;

    // work_itemsを取得
    wit::export_work_items(&app_config, &config, &ids).await;
    // revisionsを取得
    wit::export_work_items_revisions(&app_config, &config, ids).await;
    // pull_requestsを取得
    git::export_pull_requests(&app_config, &config).await;

    Ok(())

}


