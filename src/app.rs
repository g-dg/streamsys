use std::{
    net::{IpAddr, SocketAddr},
    path::Path,
    sync::Arc,
};

use axum::{
    http::{header, HeaderValue, Method},
    Router,
};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tower::ServiceBuilder;
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
};

use crate::{
    api,
    config::AppConfig,
    database::Database,
    services::{
        audit::AuditService, auth::AuthService, config::ConfigService, display_state::DisplayStateService, users::UsersService
    },
};

pub struct AppState {
    pub config: AppConfig,
    pub database: Database,
    pub shutdown_token: CancellationToken,
    pub config_service: ConfigService,
    pub audit_service: AuditService,
    pub auth_service: AuthService,
    pub users_service: UsersService,
    pub display_state_service: DisplayStateService,
}

pub struct App {
    pub state: Arc<AppState>,
    pub router: Router,
    pub listener: TcpListener,
    pub shutdown_token: CancellationToken,
}

impl App {
    pub async fn build(config: &AppConfig) -> Self {
        let shutdown_token = CancellationToken::new();

        let database = Database::new(config);

        let state = Arc::new(AppState {
            config: config.clone(),
            shutdown_token: shutdown_token.clone(),
            config_service: ConfigService::new(&database),
            audit_service: AuditService::new(&database),
            auth_service: AuthService::new(&database, config),
            users_service: UsersService::new(&database, config),
            display_state_service: DisplayStateService::new(),
            database,
        });

        let host_address = SocketAddr::from((
            state
                .config
                .host
                .parse::<IpAddr>()
                .expect("Failed to parse host IP address"),
            state.config.port,
        ));
        let listener = TcpListener::bind(host_address).await.unwrap();

        let static_file_index =
            Path::new(&state.config.static_file_root).join(state.config.static_file_index.clone());

        let cors_origins: Vec<_> = state
            .config
            .cors_allowed_origins
            .iter()
            .map(|x| x.parse().unwrap())
            .collect();

        let router = Router::new()
            .nest(
                "/",
                Router::new()
                    .route_service("/", ServeFile::new(&static_file_index))
                    .route_service(
                        "/*path",
                        ServeDir::new(&state.config.static_file_root)
                            .fallback(ServeFile::new(&static_file_index)),
                    )
                    .layer(SetResponseHeaderLayer::if_not_present(
                        header::CACHE_CONTROL,
                        HeaderValue::from_str(&format!(
                            "max-age={}",
                            state.config.http_caching_max_age
                        ))
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
            .with_state(state.clone());

        Self {
            state,
            listener,
            router,
            shutdown_token,
        }
    }
}
