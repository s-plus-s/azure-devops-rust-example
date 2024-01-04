use azure_devops_rust_lib::models::config::Config;
use chrono::{DateTime, Utc};
use std::fs;
use crate::AppConfig;

pub async fn export_work_items(app_config: &AppConfig, mut config: &mut Config, ids: &Vec<u32>) {
    let work_item_json_text_list = get_work_items(&mut config, ids.clone()).await;
    let work_items_path = format!("{}/{}", &app_config.output_path, "work_items");
    fs::create_dir_all(&work_items_path).unwrap();
    for work_item_json_text in work_item_json_text_list {
        let json: serde_json::Value = serde_json::from_str(&work_item_json_text).unwrap();
        let id = json["id"].as_u64().unwrap();
        let file_name = format!("{}.json", id);
        let file_path = format!("{}/{}", &work_items_path, file_name);
        fs::write(file_path, work_item_json_text).unwrap();
    }
}

pub async fn export_work_items_revisions(app_config: &AppConfig, mut config: &mut Config, ids: Vec<u32>) {
    let revisions_json_text_list = get_revisions(&mut config, ids.clone()).await;
    let work_items_revisions_path = format!("{}/{}", &app_config.output_path, "work_items_revisions");
    fs::create_dir_all(&work_items_revisions_path).unwrap();
    for (id, revisions_json_text) in revisions_json_text_list {
        let file_name = format!("{}.json", id);
        let file_path = format!("{}/{}", &work_items_revisions_path, file_name);
        fs::write(file_path, revisions_json_text).unwrap();
    }
}

async fn get_revisions(config: &mut Config, ids: Vec<u32>) -> Vec<(u32, String)> {

    let mut revisions_json_text_list: Vec<(u32, String)> = Vec::new();

    for id in ids {
        let json_text = azure_devops_rust_lib::resources::wit::get_workitem_revisions(&config, id).await.unwrap();
        revisions_json_text_list.push((id, json_text));
    }
    revisions_json_text_list
}

async fn get_work_items(config: &mut Config, ids: Vec<u32>) -> Vec<String> {
    // idsを100個ずつのVec<u32>に分割してループ処理する
    let mut ids_vec = Vec::new();

    let mut i = 0;
    let mut vec_index = 0;
    ids_vec.push(Vec::new());
    for id in ids {
        ids_vec[vec_index].push(id);
        i += 1;

        if i == 100 {
            i = 0;
            ids_vec.push(Vec::new());
            vec_index += 1;
        }
    }

    let mut work_item_json_text_list = Vec::new();

    // ids_vecでループ処理
    for ids in ids_vec {
        let json_text = azure_devops_rust_lib::resources::wit::get_work_items(&config, ids).await.unwrap();

        let json: serde_json::Value = serde_json::from_str(&json_text).unwrap();
        let work_items = json["value"].as_array().unwrap();

        // work_itemsでループ
        for work_item in work_items {


            println!("{}", work_item["fields"]["System.Title"].as_str().unwrap());

            // Value型からString型に変換する
            let json_text = serde_json::to_string(&work_item).unwrap();
            work_item_json_text_list.push(json_text);
        }
    }
    work_item_json_text_list
}

pub async fn get_work_items_ids(config: &mut Config, from_changed_date: DateTime<Utc>) -> Vec<u32> {

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

pub async fn export_fields(app_config: &AppConfig, config: &Config) {
    let fields_json_text = azure_devops_rust_lib::resources::wit::get_fields(&config).await.unwrap();
    let fields_path = format!("{}/{}", &app_config.output_path, "fields");
    let fields_file_path = format!("{}/{}", &fields_path, "fields.json");
    fs::create_dir_all(&fields_path).unwrap();
    fs::write(fields_file_path, fields_json_text).unwrap();
}
