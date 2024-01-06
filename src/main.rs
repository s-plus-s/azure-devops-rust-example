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
    wit::export_fields(&app_config.output_path, &config).await;

    // /wit/workitemtypecategories
    // カテゴリー一覧を取得する
    wit::export_work_item_categories(&app_config.output_path, &config).await;

    // /wit/workitemtypes/{type}/fields
    // ワーク項目タイプ一覧を取得する
    wit::export_work_item_types(&app_config.output_path, &config).await;    
    
    // /wit/workitemtypes/{type}/states
    // WorkItemの状態一覧を取得する
    wit::export_work_item_states(&app_config.output_path, &config).await;

    // wit/classificationnodes
    // WorkItemの選択肢の一覧を取得する
    wit::export_classification_nodes(&app_config.output_path, &config).await;


    // 取得開始日時を設定
    let now_utc: DateTime<Utc> = Utc::now() - Duration::days(30);

    // work_itemsのidを取得
    let ids = wit::get_work_items_ids(&config, now_utc).await;

    // work_itemsを取得
    wit::export_work_items(&app_config.output_path,  &config, &ids).await;
    // revisionsを取得
    wit::export_work_items_revisions(&app_config.output_path, &config, &ids).await;
    // pull_requestsを取得
    git::export_pull_requests(&app_config, &config).await;

    Ok(())

}


