use box_agent::app_cfg::AppCfg;
use box_agent::app_ctx::AppCtx;
use box_agent::queue_item::{CarQueue, FaceQueue, QI};
use box_agent::service::car::car_notify::CarNotifyService;
use box_agent::service::face::face_notify::FaceNotifyService;
use box_agent::service::face::face_search::FaceSearchService;
use box_agent::service::signal_service::SignalService;
use box_agent::service::web::WebService;
use box_agent::uplink::upload::UplinkService;
use build_time::build_time_local;
use clap::{arg, Command};
use deadqueue::unlimited::Queue;
use std::sync::Arc;
use tokio::sync::watch;
use tracing::info;
use fy_base::api::upload_api::QI;

use fy_base::util::{logger, service::ServiceRepo};

const APP_NAME: &str = "box_agent";
const APP_VER_NUM: &str = "0.1.0";

#[tokio::main]
async fn main() {
    let build_time = build_time_local!("%Y%m%d_%H%M%S");
    let app_ver_text = format!("{} ({})", APP_VER_NUM, build_time);

    // 命令行解析
    let cli_matches = Command::new(APP_NAME)
        .version(app_ver_text.as_str())
        .about("box agent")
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

    // 初始化 context
    let (exit_tx, exit_rx) = watch::channel(0);
    let app_context = Arc::new(AppCtx::new(app_config, exit_rx));

    // 创建服务集
    let mut service_repo = ServiceRepo::new(app_context.clone());

    // 创建队列
    let car_queue: Arc<CarQueue> = Arc::new(Queue::new());
    let face_queue: Arc<FaceQueue> = Arc::new(Queue::new());
    let face_search_queue: Arc<FaceQueue> = Arc::new(Queue::new());
    let uplink_queue: Arc<Queue<QI>> = Arc::new(Queue::new());

    // 初始退出信号服务
    let exit_service = SignalService::new(exit_tx);

    // 初始web服务
    let web_service = WebService::new(app_context.clone(), face_queue.clone(), car_queue.clone());

    // 初始化 人脸track接收 服务
    let face_notify_service =
        FaceNotifyService::new(app_context.clone(), face_queue, face_search_queue.clone());

    // 初始化 人脸track 比对 服务
    let face_search_service = FaceSearchService::new(
        0,
        app_context.clone(),
        face_search_queue,
        uplink_queue.clone(),
    );

    // 初始化 车辆track接收 服务
    let car_notify_service =
        CarNotifyService::new(app_context.clone(), car_queue.clone(), uplink_queue.clone());

    // 创建 uplink服务
    let uplink_service = UplinkService::new(app_context.clone(), uplink_queue);

    // 启动服务
    service_repo.start_service(exit_service);
    service_repo.start_service(web_service);
    service_repo.start_service(face_notify_service);
    service_repo.start_service(face_search_service);
    service_repo.start_service(car_notify_service);

    service_repo.start_service(uplink_service);

    // 等待退出
    service_repo.join().await;

    info!("app end.");
}
