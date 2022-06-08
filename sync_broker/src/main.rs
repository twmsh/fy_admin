use std::sync::Arc;
use build_time::build_time_local;
use clap::{arg, Command};
use tokio::sync::watch;
use tracing::{error, info};


use sync_broker::{app_cfg::AppCfg,
                  app_ctx::AppCtx,
                  dao::Dao,
                  util::{
                      logger, mysql_util,
                      service::ServiceRepo,
                  },
                  service::signal_service::SignalService,
};


const APP_NAME: &str = "sync_broker";
const APP_VER_NUM: &str = "0.1.0";

#[tokio::main]
async fn main() {
    let build_time = build_time_local!("%Y%m%d_%H%M%S");
    let app_ver_text = format!("{} ({})", APP_VER_NUM, build_time);

    // 命令行解析
    let cli_matches = Command::new(APP_NAME)
        .version(app_ver_text.as_str())
        .about("a server provides sync service for box agent")
        .arg(arg!(-c --config <config>).required(false).default_value("cfg.json"))
        .get_matches();

    // 读取config参数
    let config_file = cli_matches.value_of("config").expect("can't find config argument");

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

    // 初始化日志
    let log_config = &app_config.log;
    let web_log_target = "access_log";
    let app_log_target = module_path!();
    match logger::init_app_logger_with_web_str(
        log_config.app.as_str(),
        app_log_target,
        log_config.level.as_str(),
        log_config.lib_level.as_str(),
        log_config.web.as_str(),
        web_log_target,
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
            error!("error, connect db, err: {:?}",e);
            return;
        }
    };
    let dao = Dao {
        pool: Arc::new(db_pool)
    };

    // 初始化 context
    let (exit_tx, exit_rx) = watch::channel(0);
    let app_context = Arc::new(
        AppCtx::new(
            app_config,
            exit_rx,
            dao));

    // 创建服务集
    let mut service_repo = ServiceRepo::new(app_context.clone());

    // 初始退出信号服务
    let exit_signal_service = SignalService::new(exit_tx);

    // 初始web服务

    // 启动服务
    service_repo.start_service(exit_signal_service);


    // 等待退出
    service_repo.join().await;

    info!("app end.");
}
