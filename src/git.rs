use azure_devops_rust_lib::models::config::Config;
use std::fs;
use crate::AppConfig;

pub async fn export_pull_requests(app_config: &AppConfig, config: &Config) {
    let repository_id = config.repository_id.clone();
    let pull_requests_list = get_pull_requests(&config, &repository_id).await;
    let pull_requests_path = format!("{}/{}", &app_config.output_path, "pull_requests");
    fs::create_dir_all(&pull_requests_path).unwrap();
    for (id, pull_request_json_text) in pull_requests_list {
        let file_name = format!("{}.json", id);
        let file_path = format!("{}/{}", &pull_requests_path, file_name);
        fs::write(file_path, pull_request_json_text).unwrap();
    }
}

async fn get_pull_requests(config: &Config, repository_id: &str) -> Vec<(u32, String)> {

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
