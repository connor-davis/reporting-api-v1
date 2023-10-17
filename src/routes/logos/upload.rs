use axum::{
    extract::{Multipart, Query},
    response::IntoResponse,
    Json,
};
use reqwest::StatusCode;
use serde_json::json;
use tokio::{
    fs::{create_dir, try_exists, File},
    io::AsyncWriteExt,
};

pub async fn index(
    Query(params): Query<Vec<(String, String)>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let tenant = &params[0].1;

    let mut files_written = false;
    let mut directory_created_or_found = false;

    let directory_exists = try_exists("uploads").await;

    match directory_exists {
        Ok(directory) => {
            if directory {
                println!("directory found");
                directory_created_or_found = true;
            } else {
                println!("directory not found. creating");

                let create_dir_result = create_dir("uploads").await;

                match create_dir_result {
                    Ok(_) => {
                        println!("directory created");

                        directory_created_or_found = true;
                    }
                    Err(_) => {
                        println!("failed to create directory");
                    }
                }
            }
        }
        Err(_) => {
            println!("unknown error when checking directory exists")
        }
    }

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();
        let tenant_file_name = format!("{}:{}", tenant, name);

        let file_result = File::create(format!("uploads/{}", tenant_file_name)).await;

        match file_result {
            Ok(mut file) => {
                let write_result = file.write_all(&data).await;

                match write_result {
                    Ok(_) => {
                        files_written = true;
                    }
                    Err(_) => {
                        files_written = false;
                    }
                }
            }
            Err(_) => {
                files_written = false;
            }
        };
    }

    if files_written && directory_created_or_found {
        Json(json!({
            "status": StatusCode::OK.as_u16()
        }))
    } else {
        Json(json!({
            "status": StatusCode::INTERNAL_SERVER_ERROR.as_u16()
        }))
    }
}
