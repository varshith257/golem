use reqwest;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;
use tauri::command;
use tauri::Manager;
use tracing::{error, info};


// #[command]
// fn greet(name: &str) -> String {
//     format!("Hello, {}!", name)
// }

#[command]
pub async fn upload_openapi_definition(file_path: String) -> Result<String, String> {
    let url = "http://localhost:9881/v1/api/definitions/import";
    let openapi_json = std::fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let client = reqwest::Client::new();
    let response = client
        .put(url)
        .body(openapi_json)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status().is_success() {
        Ok("OpenAPI definition uploaded successfully".to_string())
    } else {
        Err(format!(
            "Failed to upload OpenAPI definition: {:?}",
            response.text().await
        ))
    }
}

// #[command]
// pub async fn list_api_definitions(api_definition_id: Option<String>) -> Result<String, String> {
//     println!("enteirgn this=======>");
//     let url = match api_definition_id {
//         Some(id) => format!(
//             "http://localhost:9881/v1/api/definitions?api-definition-id={}",
//             id
//         ),
//         None => "http://localhost:9881/v1/api/definitions".to_string(),
//     };
//     let client = reqwest::Client::new();
//     let response = client.get(&url).send().await.map_err(|e| e.to_string())?;

//     if response.status().is_success() {
//         Ok(response
//             .text()
//             .await
//             .unwrap_or_else(|_| "Success but empty response".to_string()))
//     } else {
//         Err(format!(
//             "Failed to list API definitions: {:?}",
//             response.text().await
//         ))
//     }
// }


#[command]
pub async fn list_api_definitions() -> Result<Value, String> {
    println!("Entering this=======>");
    let url = "http://localhost:9881/v1/api/definitions".to_string();

    let client = reqwest::Client::new();
    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;

    if response.status().is_success() {
        let result: Value = response
            .json::<Value>()
            .await
            .map_err(|e| format!("Failed to deserialize JSON: {}", e))?;
        Ok(result)
    } else {
        Err(format!(
            "Failed to list API definitions: {:?}",
            response.text().await
        ))
    }
}
// pub async fn list_api_definitions() -> Result<Value, String> {
//     println!("enteirgn this=======>");
//     let url = "http://localhost:9881/v1/api/definitions".to_string();

//     let client = reqwest::Client::new();
//     let response = client.get(&url).send().await.map_err(|e| e.to_string())?;

//     if response.status().is_success(){
//         let result:Value = response.json();
//         // Ok(response
//         //     .text()
//             .await
//             .unwrap_or_else(|_| "Success but empty response".to_string()).Ok(result);
//     } else {
//         Err(format!(
//             "Failed to list API definitions: {:?}",
//             response.text().await
//         ))
//     }
// }

#[command]
pub async fn create_api_definition(
    project_name: String,
    api_name: String,
    version: String,
) -> Result<String, String> {
    let url = "http://localhost:9881/v1/api/definitions";

    let api_definition = json!({
            "id": api_name,
            "version": version,
    "routes": [
        {
            "method": "GET",
            "path": "/example",
            "security": "none",
            "binding": {
                "componentId": {
                    "componentId": "616ccd92-d666-4180-8349-8d125b269fac",
                    "version": 0
                },
                "workerName": "worker_name",
                "bindingType": "default"
            }
        }
    ],

            "draft": true,
        });

    let client = reqwest::Client::new();

    let response = client
        .post(url)
        .json(&api_definition)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if response.status().is_success() {
        Ok("API definition created successfully".to_string())
    } else {
        Err(format!(
            "Failed to create API definition: {:?}",
            response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string())
        ))
    }
}

#[command]
pub async fn update_api_definition(
    id: String,
    version: String,
    updated_definition: String,
) -> Result<String, String> {
    let url = format!(
        "http://localhost:9881/v1/api/definitions/{}/{}",
        id, version
    );
    let client = reqwest::Client::new();
    let response = client
        .put(&url)
        .body(updated_definition)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status().is_success() {
        Ok("API definition updated successfully".to_string())
    } else {
        Err(format!(
            "Failed to update API definition: {:?}",
            response.text().await
        ))
    }
}

#[command]
pub async fn delete_api_definition(id: String, version: String) -> Result<String, String> {
    let url = format!(
        "http://localhost:9881/v1/api/definitions/{}/{}",
        id, version
    );
    let client = reqwest::Client::new();
    let response = client
        .delete(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status().is_success() {
        Ok("API definition deleted successfully".to_string())
    } else {
        Err(format!(
            "Failed to delete API definition: {:?}",
            response.text().await
        ))
    }
}

// #[tokio::main]
// async fn main() {
//     // Test the create_api_definition function
//     let result = create_api_definition(
//         "TestProject".to_string(),
//         "TestAPI".to_string(),
//         "0.1.0".to_string(),
//     )
//     .await;

//     match result {
//         Ok(success) => println!("{}", success),
//         Err(error) => eprintln!("Error: {}", error),
//     }
// }
