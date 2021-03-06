use axum::routing::get;
use axum::{middleware, Router, Server};
use hyper::server::conn::AddrIncoming;
use hyper::server::Builder;
use std::net::{AddrParseError, SocketAddr};
use std::sync::Arc;

use crate::{
    app_ctx::AppCtx,
    service::web::sync::{get_camera_update, get_db_update, get_person_update},
};
use fy_base::util::{axum_log::access_log, axum_log::time_use, service::Service};
use tokio::sync::watch::Receiver;
use tokio::task::JoinHandle;
use tower::limit::GlobalConcurrencyLimitLayer;
use tower::ServiceBuilder;
use tower_http::add_extension::AddExtensionLayer;

use tracing::{error, info};

pub mod model;
pub mod sync;

pub struct WebState {
    pub ctx: Arc<AppCtx>,
}

pub struct WebService {
    pub ctx: Arc<AppCtx>,
}

impl WebService {
    pub fn new(ctx: Arc<AppCtx>) -> Self {
        Self { ctx }
    }

    pub async fn init_router(&self) -> Router {
        let max_request_conn = self.ctx.cfg.http.max_conn as usize;

        let web_state = Arc::new(WebState {
            ctx: self.ctx.clone(),
        });

        Router::new()
            .route("/db_sync", get(get_db_update))
            .route("/person_sync", get(get_person_update))
            .route("/camera_sync", get(get_camera_update))
            .layer(
                ServiceBuilder::new()
                    // 限制请求的并发数量
                    .layer(GlobalConcurrencyLimitLayer::new(max_request_conn))
                    // 设置 web state
                    .layer(AddExtensionLayer::new(web_state))
                    // 接口调用时间
                    .layer(middleware::from_fn(time_use))
                    // access log日志
                    .layer(middleware::from_fn(|req, next| async {
                        let f = |line: String| {
                            info!(target:"access_log","{}",line);
                        };
                        access_log(req, next, f).await
                    })), // .layer(TraceLayer::new_for_http()),
            )
    }

    pub fn init_socket_addr(&self) -> Result<SocketAddr, AddrParseError> {
        let addr = self.ctx.cfg.http.addr.as_str();
        addr.parse::<SocketAddr>()
    }

    async fn run(self, server: Builder<AddrIncoming>, exit_rx: Receiver<i64>) {
        let mut exit_rx = exit_rx;
        let mut exit_rx_cl = exit_rx.clone();

        // 创建 router
        let app = self.init_router().await;

        let graceful = server
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .with_graceful_shutdown(async move {
                let _ = exit_rx_cl.changed().await;
                info!("with_graceful_shutdown.");
            });

        tokio::select! {
            _ = graceful => {
                info!("with_graceful_shutdown.");
            }
            _ = exit_rx.changed() => {
                info!("WebService, recv signal, will exit");
            }
        }
        info!("WebService exit.");
    }
}

impl Service for WebService {
    fn run(self, exit_rx: Receiver<i64>) -> JoinHandle<()> {
        // 绑定http端口
        let addr = self
            .init_socket_addr()
            .unwrap_or_else(|_| panic!("cant parse http addr: {}", self.ctx.cfg.http.addr));
        info!("http bind: {:?}", addr);

        let server = Server::try_bind(&addr).unwrap_or_else(|e| {
            error!("error, http can't bind: {:?}, err: {:?}", addr, e);
            panic!("error, http can't bind: {:?}, err: {:?}", addr, e)
        });

        tokio::spawn(self.run(server, exit_rx))
    }
}
