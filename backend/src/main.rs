use axum::{extract::State, response::IntoResponse};
use common::{CreateMessage, Message};
use sqlx::Row;

#[derive(Clone)]
struct AppState {
    pool: sqlx::sqlite::SqlitePool,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();

    // 静态文件打包路径
    let static_target = std::env::var("STATIC_TARGET").unwrap();

    // 初始化数据库
    let pool = {
        let options = sqlx::sqlite::SqliteConnectOptions::new().filename(":memory:");

        match sqlx::sqlite::SqlitePool::connect_with(options).await {
            Ok(pool) => {
                let _ = sqlx::query(
                    "CREATE TABLE IF NOT EXISTS messages(id INTEGER PRIMARY KEY, message TEXT)",
                )
                .execute(&pool)
                .await;
                pool
            }
            Err(err) => {
                println!("Error connecting to database: {:?}", err);
                std::process::exit(1);
            }
        }
    };

    let app = axum::Router::new()
        .route("/message", axum::routing::get(get_messages))
        .route("/message", axum::routing::put(create_message))
        .fallback_service(tower_http::services::ServeDir::new(static_target))
        .with_state(AppState { pool });

    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 8080))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_messages(state: State<AppState>) -> Result<impl IntoResponse, HttpError> {
    let messages = sqlx::query("SELECT id, message FROM messages ORDER BY id desc LIMIT 10")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| HttpError::internal_server_error(e.to_string()))?;

    let messages: Vec<Message> = messages
        .into_iter()
        .map(|row| Message {
            id: row.get("id"),
            content: row.get("message"),
        })
        .collect();

    Ok(axum::Json(messages))
}

async fn create_message(
    state: State<AppState>,
    payload: axum::Json<CreateMessage>,
) -> Result<impl IntoResponse, HttpError> {
    let _ = sqlx::query("INSERT INTO messages(message) VALUES($1)")
        .bind(&payload.content)
        .execute(&state.pool)
        .await
        .map_err(|e| HttpError::internal_server_error(e.to_string()))?;

    Ok(axum::http::StatusCode::CREATED)
}

struct HttpError {
    code: axum::http::StatusCode,
    message: String,
}

#[derive(serde::Serialize)]
struct HttpErrorMessage {
    message: String,
}

#[allow(unused)]
impl HttpError {
    fn new(code: axum::http::StatusCode, message: String) -> Self {
        Self { code, message }
    }

    fn bad_request(message: String) -> Self {
        Self::new(axum::http::StatusCode::BAD_REQUEST, message)
    }

    fn internal_server_error(message: String) -> Self {
        Self::new(axum::http::StatusCode::INTERNAL_SERVER_ERROR, message)
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> axum::response::Response {
        let body = axum::Json(HttpErrorMessage {
            message: self.message,
        });
        (self.code, body).into_response()
    }
}
