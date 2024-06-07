use std::{
    net::{IpAddr, SocketAddr},
    path::Path,
    sync::Arc,
};

use axum::{
    extract::{Request, State},
    http::{header, HeaderValue, Method},
    response::IntoResponse,
    routing::get,
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
    audit::AuditService,
    auth::service::AuthService,
    config::{file::AppConfig, service::ConfigService},
    database::Database,
    state::service::StateService,
    users::service::UsersService,
};

pub struct AppServices {
    pub config: AppConfig,
    pub database: Database,
    pub shutdown_token: CancellationToken,
    pub config_service: ConfigService,
    pub audit_service: AuditService,
    pub auth_service: AuthService,
    pub users_service: UsersService,
    pub state_service: StateService,
}

pub struct App {
    pub services: Arc<AppServices>,
    pub router: Router,
    pub listener: TcpListener,
    pub shutdown_token: CancellationToken,
}

impl App {
    pub async fn build(config: &AppConfig) -> Self {
        let shutdown_token = CancellationToken::new();

        let database = Database::new(config);

        let state = Arc::new(AppServices {
            config: config.clone(),
            shutdown_token: shutdown_token.clone(),
            config_service: ConfigService::new(&database),
            audit_service: AuditService::new(&database),
            auth_service: AuthService::new(&database, config),
            users_service: UsersService::new(&database, config),
            state_service: StateService::new(),
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

        let client_router = if config.client_proxy_url.is_some() {
            async fn client_proxy_handler(
                State(state): State<Arc<AppServices>>,
                req: Request,
            ) -> impl IntoResponse {
                let request_path = req.uri().path();
                let request_path_query = req
                    .uri()
                    .path_and_query()
                    .map(|path_and_query| path_and_query.as_str())
                    .unwrap_or(request_path);

                let client_uri_root = state
                    .config
                    .client_proxy_url
                    .as_ref()
                    .unwrap()
                    .trim_end_matches('/');

                let uri = format!("{}{}", client_uri_root, request_path_query);

                let response = reqwest::get(uri).await.unwrap();

                (
                    [(
                        header::CONTENT_TYPE,
                        String::from(
                            response
                                .headers()
                                .get(header::CONTENT_TYPE.as_str())
                                .map(|content_type| content_type.to_str().unwrap())
                                .unwrap_or("text/plain"),
                        ),
                    )],
                    response.bytes().await.unwrap(),
                )
            }

            Router::new()
                .route("/", get(client_proxy_handler))
                .route("/*path", get(client_proxy_handler))
        } else {
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
                ))
        };

        let router = Router::new()
            .nest("/", client_router)
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
            services: state,
            listener,
            router,
            shutdown_token,
        }
    }
}
