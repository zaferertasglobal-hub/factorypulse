use axum::{
    routing::get,
    Router,
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool, FromRow};
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Clone)]
struct AppState {
    db: PgPool,
    tx: broadcast::Sender<MachineData>,
}

#[derive(Serialize, Deserialize, FromRow, Clone, Debug)]
struct MachineData {
    id: Uuid,
    machine_id: String,
    temperature: f32,
    vibration: f32,
    pressure: f32,
    status: String,
    timestamp: DateTime<Utc>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let database_url = "postgres://zafer:secret123@postgres:5432/factorypulse";
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .expect("Database connection failed");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS machine_data (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            machine_id TEXT NOT NULL,
            temperature REAL,
            vibration REAL,
            pressure REAL,
            status TEXT,
            timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create table");

    let (tx, _) = broadcast::channel::<MachineData>(1000);
    let state = Arc::new(AppState { db: pool, tx });

    let app = Router::new()
        .route("/api/health", get(|| async { "OK" }))
        .route("/api/data", get(get_latest_data))
        .route("/ws", get(ws_handler))
        .with_state(state.clone());

    println!("FACTORYPULSE BACKEND ÇALIŞIYOR → http://0.0.0.0:8000");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_latest_data(State(state): State<Arc<AppState>>) -> Json<Vec<MachineData>> {
    let data = sqlx::query_as::<_, MachineData>(
        "SELECT * FROM machine_data ORDER BY timestamp DESC LIMIT 50",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Json(data)
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| async move {
        let mut rx = state.tx.subscribe();
        let mut socket = socket;

        while let Ok(data) = rx.recv().await {
            let msg = serde_json::to_string(&data).unwrap();
            if socket.send(axum::extract::ws::Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    })
}