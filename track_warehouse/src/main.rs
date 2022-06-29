use build_time::build_time_local;
use clap::{arg, Command};
use deadqueue::unlimited::Queue;
use std::sync::Arc;
use tokio::sync::watch;
use tracing::{error, info};
use track_warehouse::{app_cfg::AppCfg, app_ctx::AppCtx, service::signal_service::SignalService};

use fy_base::util::{logger, mysql_util, service::ServiceRepo};
use track_warehouse::dao::Dao;
use track_warehouse::queue_item::{CarQueue, FaceQueue};
use track_warehouse::service::minio::MinioService;
use track_warehouse::service::web::WebService;

const APP_NAME: &str = "track_warehouse";
const APP_VER_NUM: &str = "0.1.0";

#[tokio::main]
async fn main() {
    let build_time = build_time_local!("%Y%m%d_%H%M%S");
    let app_ver_text = format!("{} ({})", APP_VER_NUM, build_time);

    // 命令行解析
    let cli_matches = Command::new(APP_NAME)
        .version(app_ver_text.as_str())
        .about("track warehouse")
        .arg(
            arg!(-c --config <config>)
                .required(false)
                .default_value("cfg.json"),
        )
        .get_matches();

    // 读取config参数
    let config_file = cli_matches
        .value_of("config")
        .expect("can't find config argument");

    // 加载config文件内容
    println!("read config: {}", config_file);
    let app_config = match AppCfg::load(config_file) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error, load {}, err: {:?}", config_file, e);
            return;
        }
    };
    println!("config: {:?}", app_config);

    // 检查app_cfg的字段是否合法
    if let Err(e) = app_config.validate() {
        eprintln!("error, validate config, err: {:?}", e);
        return;
    }

    // 初始化日志
    let log_config = &app_config.log;
    let app_log_target = module_path!();
    match logger::init_app_logger_str(
        log_config.app.as_str(),
        app_log_target,
        log_config.level.as_str(),
        log_config.lib_level.as_str(),
    ) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("error, init log, err: {:?}", e);
            return;
        }
    };

    // 初始化数据库连接
    let db_pool = match mysql_util::init_mysql_pool(
        app_config.db.url.as_str(),
        app_config.db.tz.as_str(),
        app_config.db.max_conn as u32,
        app_config.db.min_conn as u32,
    ) {
        Ok(v) => v,
        Err(e) => {
            error!("error, connect db, err: {:?}", e);
            return;
        }
    };

    let tz = match mysql_util::parse_timezone(app_config.db.tz.as_str()) {
        Ok(v) => v,
        Err(e) => {
            error!("error, parse_timezone {}, err: {:?}", app_config.db.tz, e);
            return;
        }
    };

    let dao = Dao {
        pool: Arc::new(db_pool),
        tz,
    };

    // 初始化 context
    let (exit_tx, exit_rx) = watch::channel(0);
    let app_context = Arc::new(AppCtx::new(app_config, exit_rx, dao));

    // 创建服务集
    let mut service_repo = ServiceRepo::new(app_context.clone());

    // 创建队列
    let car_queue: Arc<CarQueue> = Arc::new(Queue::new());
    let face_queue: Arc<FaceQueue> = Arc::new(Queue::new());

    let car_mysql_queue: Arc<CarQueue> = Arc::new(Queue::new());
    let face_search_queue: Arc<FaceQueue> = Arc::new(Queue::new());


    // 初始退出信号服务
    let exit_service = SignalService::new(exit_tx);

    // 初始web服务
    let web_service = WebService::new(app_context.clone(), face_queue.clone(), car_queue.clone());

    // 初始 minio 服务
    let minio_service = MinioService::new(
        app_context.clone(),
        face_queue.clone(),
        car_queue.clone(),
        face_search_queue.clone(),
        car_mysql_queue.clone()

    );

    // 启动服务
    service_repo.start_service(exit_service);
    service_repo.start_service(web_service);
    service_repo.start_service(minio_service);

    // 等待退出
    service_repo.join().await;

    info!("app end.");
}
