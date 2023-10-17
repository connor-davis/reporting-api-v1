use axum::{response::IntoResponse, Json, extract::Query, body::StreamBody};
use reqwest::StatusCode;
use serde_json::json;
use tokio::fs::read_dir;
use tokio_util::io::ReaderStream;

pub async fn index(Query(params): Query<Vec<(String, String)>>) -> impl IntoResponse {
    let tenant = &params[0].1;
    let directory_files_result = read_dir("uploads").await;
    let mut files: Vec<String> = Vec::new();

    match directory_files_result {
        Ok(mut dir) => {
            while let Some(entry) = dir.next_entry().await.unwrap() {
                let file_name_str = &entry.file_name();
                let mut file_name = String::from(file_name_str.to_str().unwrap());

                if file_name.contains(tenant) {
                    file_name = file_name.replace(tenant, "");
                    file_name = file_name.replace(":", "");

                    files.push(file_name)
                }
            }
        },
        Err(_) => {}
    }

    Json(json!({
        "status": StatusCode::OK.as_u16(),
        "files": files
    }))
}

pub async fn get_file(Query(params): Query<Vec<(String, String)>>) -> impl IntoResponse {
    let file_name = &params[0].1;
    
    // `File` implements `AsyncRead`
    let file = match tokio::fs::File::open(format!("uploads/{}", file_name)).await {
        Ok(file) => file,
        Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
    };
    // convert the `AsyncRead` into a `Stream`
    let stream = ReaderStream::new(file);
    // convert the `Stream` into an `axum::body::HttpBody`
    let body = StreamBody::new(stream);

    Ok(body)
}
