/* Copyright (C) 2025 Natália Silva Machado 

This file is part of Data-Sniffing Caramelo. 

Data-Sniffing Caramelo is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation; version 3 of the License. 

Data-Sniffing Caramelo is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details. 

You should have received a copy of the GNU General Public License along with Data-Sniffing Caramelo. If not, see <https://www.gnu.org/licenses/>. */

mod utils;
mod password_strength;

use axum::{
    extract::{Json, Path, State},
    routing::{get, post},
    http::StatusCode,
    response::{Html, IntoResponse},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use uuid::Uuid;
use tokio::task;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct AppState {
    pub tasks: Arc<Mutex<HashMap<String, TaskStatus>>>,
}

#[derive(Clone)]
pub struct TaskStatus {
    pub ready: bool,
    pub results: Option<Vec<utils::CheckResult>>,
}

#[derive(Deserialize)]
pub struct CrawlerRequest {
    pub url: String,
}

#[derive(Serialize)]
pub struct StartResponse {
    pub success: bool,
    pub task_id: String,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        tasks: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/", get(serve_html))
        .route("/run-crawler", post(run_crawler))
        .route("/crawler-result/{task_id}", get(get_crawler_result))
        .nest_service(
            "/assets",
            ServeDir::new("assets")
        )
        .nest_service(
            "/static", 
            ServeDir::new("static")
        )
        .nest_service(
            "/scripts", 
            ServeDir::new("scripts")
        )
        .with_state(state);


    let mut port = std::env::var("PORT").unwrap_or("8080".to_string());
    let mut addr = format!("0.0.0.0:{}", port);

    if cfg!(debug_assertions) {
        port = std::env::var("PORT").unwrap_or("3000".to_string());
        addr = format!("127.0.0.1:{}", port);
    }

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");

    axum::serve(listener, app)
        .await
        .expect("server error");
}

async fn serve_html() -> impl IntoResponse {
    let html_content = include_str!("../index.html");
    Html(html_content)
}

async fn run_crawler(
    State(state): State<AppState>,
    Json(payload): Json<CrawlerRequest>,
) -> impl IntoResponse {
    let task_id = Uuid::new_v4().to_string();

    {
        let mut tasks = state.tasks.lock().unwrap();
        tasks.insert(
            task_id.clone(),
            TaskStatus {
                ready: false,
                results: None,
            },
        );
    }

    let task_id_clone = task_id.clone();
    let state_clone = state.clone();
    let url = payload.url.clone();

    task::spawn(async move {
        let results = utils::run_crawler(&url).await;

        let mut tasks = state_clone.tasks.lock().unwrap();
        tasks.insert(
            task_id_clone,
            TaskStatus {
                ready: true,
                results: Some(results),
            },
        );
    });

    (
        StatusCode::OK,
        Json(StartResponse {
            success: true,
            task_id,
        }),
    )
}

async fn get_crawler_result(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let tasks = state.tasks.lock().unwrap();

    if let Some(task) = tasks.get(&task_id) {
        if task.ready {
            return (
                StatusCode::OK,
                Json(serde_json::json!({
                    "ready": true,
                    "results": task.results
                })),
            );
        }

        return (
            StatusCode::OK,
            Json(serde_json::json!({
                "ready": false
            })),
        );
    }

    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error": "Task not found"
        })),
    )
}
