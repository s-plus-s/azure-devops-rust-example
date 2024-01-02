use serde::{Serialize, Deserialize};
use std::fs;
use chrono::{DateTime, Utc, Duration};

extern crate azure_devops_rust_lib;
use azure_devops_rust_lib::models::config::Config;

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
    let mut config: Config = Config { organization: app_config.organization, project: app_config.project, repository_id: app_config.repository_id, pat: app_config.pat };

    // 取得開始日時を設定
    let now_utc: DateTime<Utc> = Utc::now() - Duration::days(30);

    // work_itemsのidを取得
    let ids = get_work_items_ids(&mut config, now_utc).await;

    // work_itemsを取得
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

    // revisionsを取得
    let revisions_json_text_list = get_revisions(&mut config, ids.clone()).await;
    let work_items_revisions_path = format!("{}/{}", &app_config.output_path, "work_items_revisions");
    fs::create_dir_all(&work_items_revisions_path).unwrap();
    for (id, revisions_json_text) in revisions_json_text_list {
        let file_name = format!("{}.json", id);
        let file_path = format!("{}/{}", &work_items_revisions_path, file_name);
        fs::write(file_path, revisions_json_text).unwrap();
    }

    // pull_requestsを取得
    let repository_id = config.repository_id.clone();
    let pull_requests_list =  get_pull_requests(&mut config, &repository_id).await;
    let pull_requests_path = format!("{}/{}", &app_config.output_path, "pull_requests");
    fs::create_dir_all(&pull_requests_path).unwrap();
    for (id, pull_request_json_text) in pull_requests_list {
        let file_name = format!("{}.json", id);
        let file_path = format!("{}/{}", &pull_requests_path, file_name);
        fs::write(file_path, pull_request_json_text).unwrap();
    }

    Ok(())

}

async fn get_pull_requests(config: &mut Config, repository_id: &str) -> Vec<(u32, String)> {

    let mut pull_requests_json_text_list = Vec::new();

    // pull_requestsを全件取得するために、100件ずつ取得する
    let mut skip = 0u32;
    let mut top = 100u32;
    let mut pull_requests_list = Vec::new();
    loop {
        let json_text = azure_devops_rust_lib::resources::git::get_pull_requests(&config, repository_id, Option::from(skip), Option::from(top), None, None, None, None, None).await.unwrap();

        let json: serde_json::Value = serde_json::from_str(&json_text).unwrap();
        let pullrequests = json["value"].as_array().unwrap();

        // pullrequestsでループ
        for pullrequest in pullrequests {
            let id = pullrequest["pullRequestId"].as_u64().unwrap() as u32;
            pull_requests_list.push((id, serde_json::to_string(&pullrequest).unwrap()));
        }

        if pullrequests.len() < 100 {
            break;
        }

        skip += 100;
        top += 100;
    }

    // pullrequestsでループ
    for (id, pullrequest) in pull_requests_list {
        pull_requests_json_text_list.push((id, serde_json::to_string(&pullrequest).unwrap()));
    }
    pull_requests_json_text_list
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

async fn get_work_items_ids(config: &mut Config, from_changed_date: DateTime<Utc>) -> Vec<u32> {

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


