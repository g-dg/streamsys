pub mod api;
pub mod app;
pub mod audit;
pub mod auth;
pub mod config;
pub mod content;
pub mod database;
pub mod helpers;
pub mod state;
pub mod tasks;
pub mod users;

use app::App;
use config::file::AppConfig;
use tokio::signal;
use tokio_util::sync::CancellationToken;

use tasks::maintenance;

const CONFIG_FILE: &str = "./config.json";

#[tokio::main]
pub async fn main() {
    let config = AppConfig::load(CONFIG_FILE).await;

    let app = App::build(&config).await;

    app.services.audit_service.log(None, "startup");

    let maintenance_task = tokio::spawn(maintenance::maintenance_tasks(app.services.clone()));

    axum::serve(app.listener, app.router)
        .with_graceful_shutdown(shutdown_signal(app.shutdown_token))
        .await
        .expect("Error occurred in web server task");

    maintenance_task
        .await
        .expect("Error occurred in maintenance task");

    app.services.audit_service.log(None, "shutdown");
}

async fn shutdown_signal(shutdown_token: CancellationToken) {
    let ctrl_c = async {
        signal::ctrl_c().await.unwrap();
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .unwrap()
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => shutdown_token.cancel(),
        _ = terminate => shutdown_token.cancel(),
        _ = shutdown_token.cancelled() => {},
    }
}
