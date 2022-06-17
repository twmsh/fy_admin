use build_time::build_time_local;
use clap::{arg, Command};
use std::sync::Arc;
use tokio::sync::watch;
use tracing::{error, info};
use deadqueue::unlimited::Queue;
use sync_client::{app_cfg::AppCfg, app_ctx::AppCtx, service::signal_service::SignalService};

use fy_base::util::{logger, se5, service::ServiceRepo};
use sync_client::model::queue_item::{RabbitmqItem, TaskItem};
use sync_client::service::timer_service::TimerService;
use sync_client::service::wroker::worker_service::WorkerService;

const APP_NAME: &str = "sync_client";
const APP_VER_NUM: &str = "0.1.0";

#[tokio::main]
async fn main() {
    let build_time = build_time_local!("%Y%m%d_%H%M%S");
    let app_ver_text = format!("{} ({})", APP_VER_NUM, build_time);

    // 命令行解析
    let cli_matches = Command::new(APP_NAME)
        .version(app_ver_text.as_str())
        .about("sync client for box agent")
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

    //  获取设备SN，1）先从cfg.json文件读取，2）没有的话，从设备读取。
    let hw_id = match app_config.hw_id {
        None => {
            // 从设备读取
            match se5::get_device_sn() {
                Ok(v) => v,
                Err(e) => {
                    error!("error, read device sn, err:{:?}", e);
                    return;
                }
            }
        }
        Some(ref v) => v.clone(),
    };

    // 初始化 context
    let (exit_tx, exit_rx) = watch::channel(0);
    let app_context = Arc::new(AppCtx::new(app_config, exit_rx, hw_id));

    // 创建服务集
    let mut service_repo = ServiceRepo::new(app_context.clone());

    // 创建队列
    let task_queue: Arc<Queue<TaskItem>> = Arc::new(Queue::new());
    let rabbitmq_queue: Arc<Queue<RabbitmqItem>> = Arc::new(Queue::new());

    // 初始退出信号服务
    let exit_service = SignalService::new(exit_tx);

    // 初始化timer服务
    let timer_service = TimerService::new(
        app_context.clone(),
        task_queue.clone());

    // 初始化worker服务
    let worker_service = WorkerService::new(
        app_context.clone(),
        task_queue.clone(),
        rabbitmq_queue.clone());

    // 启动服务
    service_repo.start_service(exit_service);
    service_repo.start_service(timer_service);
    service_repo.start_service(worker_service);

    // 等待退出
    service_repo.join().await;

    info!("app end.");
}
