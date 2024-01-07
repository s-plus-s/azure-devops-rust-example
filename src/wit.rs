use azure_devops_rust_lib::models::config::Config;
use chrono::{DateTime, Utc};

pub async fn export_work_items(root_path: &String, config: &Config, ids: &Vec<u32>) {
    azure_devops_rust_lib::data_loader::wit::load_work_items(&root_path ,&config, &ids).await;
}

pub async fn export_work_items_revisions(root_path: &String, config: &Config, ids: &Vec<u32>) {
    azure_devops_rust_lib::data_loader::wit::load_work_items_revisions(&root_path ,&config, &ids).await;
}

pub async fn get_work_items_ids(config: &Config, from_changed_date: DateTime<Utc>) -> Vec<u32> {

    let date_time_str = from_changed_date.format("%Y-%m-%d %H:%M:%S").to_string();
    // Work Item Query Language (WIQL) クエリ
    let query = r#"{
        "query": "
        SELECT
            [System.Id], [System.Title], [System.WorkItemType]
        FROM
            workitems
        WHERE
            [System.TeamProject] = '@project' AND [System.ChangedDate] > '@now_utc'
        ORDER BY
            [System.Id]"
    }"#.replace("@project", &config.project).replace("@now_utc", &date_time_str);

    let json_text = azure_devops_rust_lib::resources::wit::get_work_item_ids(&config, &query).await.unwrap();

    let json: serde_json::Value = serde_json::from_str(&json_text).unwrap();
    let work_items = json["workItems"].as_array().unwrap();

    println!("{}", work_items.len());
    let mut ids = Vec::new();
    for work_item in work_items {
        let id: u32 = work_item["id"].as_u64().unwrap() as u32;
        ids.push(id);
    }
    println!("{}", ids.len());
    ids
}

pub async fn export_fields(root_path: &String, config: &Config) {
    // /wit/fields
    azure_devops_rust_lib::data_loader::wit::load_fields(&root_path ,&config).await;
}

pub async fn export_work_item_types(root_path: &String, config: &Config) {
    // wit/workitemtypes/{}/fields
    // ワーク項目タイプ一覧の取得
    azure_devops_rust_lib::data_loader::wit::load_work_item_types(&root_path ,&config).await;  
}

pub async fn export_work_item_categories(root_path: &String, config: &Config) {
    // /wit/workitemtypecategories
    // カテゴリー一覧の取得
    azure_devops_rust_lib::data_loader::wit::load_categories(&root_path ,&config).await;  
}

pub async fn export_work_item_states(root_path: &String, config: &Config) {
    // wit/workitemtypes/{type}/states
    // ワーク項目ステート一覧の取得
    azure_devops_rust_lib::data_loader::wit::load_work_item_states(&root_path ,&config).await; 
}

pub async fn export_classification_nodes(root_path: &String, config: &Config) {
    // wit/classificationnodes
    // ワークアイテムの種類一覧を取得する(ClassificationNodes)
    azure_devops_rust_lib::data_loader::wit::load_classification_nodes(&root_path ,&config).await; 
}
