pub mod api;
pub mod config;
pub mod database;
pub mod helpers;
pub mod services;
pub mod tasks;

use std::{
    net::{IpAddr, SocketAddr},
    path::Path,
    sync::Arc,
};

use axum::{
    http::{header, HeaderValue, Method},
    Router,
};
use tokio::{net::TcpListener, signal};
use tokio_util::sync::CancellationToken;
use tower::ServiceBuilder;
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
};

use config::AppConfig;
use database::Database;
use services::{audit::AuditService, auth::AuthService, users::UsersService};
use tasks::maintenance;

const CONFIG_FILE: &str = "./config.json";

pub struct AppState {
    pub config: AppConfig,
    pub database: Database,
    pub shutdown_token: CancellationToken,
    pub audit_service: AuditService,
    pub auth_service: AuthService,
    pub users_service: UsersService,
}

#[tokio::main]
pub async fn main() {
    let config = AppConfig::load(CONFIG_FILE).await;

    let host_address = SocketAddr::from((
        config
            .host
            .parse::<IpAddr>()
            .expect("Failed to parse host IP address"),
        config.port,
    ));
    let tcp_listener = TcpListener::bind(host_address).await.unwrap();

    let shutdown_token = CancellationToken::new();

    let database = Database::new(&config);

    let app_state = Arc::new(AppState {
        audit_service: AuditService::new(&database),
        auth_service: AuthService::new(&database, &config),
        users_service: UsersService::new(&database, &config),
        shutdown_token: shutdown_token.clone(),
        database,
        config: config.clone(),
    });

    let static_file_index = Path::new(&config.static_file_root).join(config.static_file_index);

    let cors_origins: Vec<_> = config
        .cors_allowed_origins
        .iter()
        .map(|x| x.parse().unwrap())
        .collect();

    let app = Router::new()
        .nest(
            "/",
            Router::new()
                .route_service("/", ServeFile::new(&static_file_index))
                .route_service(
                    "/*path",
                    ServeDir::new(&config.static_file_root)
                        .fallback(ServeFile::new(&static_file_index)),
                )
                .layer(SetResponseHeaderLayer::if_not_present(
                    header::CACHE_CONTROL,
                    HeaderValue::from_str(&format!("max-age={}", config.http_caching_max_age))
                        .unwrap(),
                )),
        )
        .nest(
            "/api",
            api::route().layer(SetResponseHeaderLayer::if_not_present(
                header::CACHE_CONTROL,
                HeaderValue::from_static(
                    "no-store, no-cache, max-age=0, must-revalidate, proxy-revalidate",
                ),
            )),
        )
        .layer(
            ServiceBuilder::new()
                .layer(
                    CorsLayer::new()
                        .allow_origin(cors_origins)
                        .allow_credentials(true)
                        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
                        .allow_methods([
                            Method::GET,
                            Method::POST,
                            Method::PUT,
                            Method::PATCH,
                            Method::DELETE,
                        ]),
                )
                .layer(CatchPanicLayer::new())
                .layer(CompressionLayer::new()),
        )
        .with_state(app_state.clone());

    app_state.audit_service.log(None, "startup");

    let maintenance_task = tokio::spawn(maintenance::maintenance_tasks(app_state.clone()));

    axum::serve(tcp_listener, app)
        .with_graceful_shutdown(shutdown_signal(shutdown_token))
        .await
        .expect("Error occurred in web server task");

    maintenance_task
        .await
        .expect("Error occurred in maintenance task");

    app_state.audit_service.log(None, "shutdown");
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
