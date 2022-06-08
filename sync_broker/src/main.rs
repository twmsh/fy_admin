use build_time::build_time_local;
use clap::{arg, Command};
use sync_broker::app_cfg::AppCfg;
use sync_broker::error::AppResult;

const APP_NAME: &str = "sync_broker";
const APP_VER_NUM: &str = "0.1.0";

#[tokio::main]
async fn main() {
    let build_time = build_time_local!("%Y%m%d_%H%M%S");
    let app_ver_text = format!("{} ({})", APP_VER_NUM, build_time);

    // 命令行解析
    let cli_matches = Command::new(APP_NAME)
        .author("tomtong")
        .version(app_ver_text.as_str())
        .about("a server provides sync service for box agent")
        .arg(arg!(-c --config <config>)).get_matches();

    // 读取config参数
    let config_file = cli_matches.value_of("config").expect("can't find config argument");

    // 加载config文件内容
    println!("read config: {}",config_file);
    let app_config = match AppCfg::load(config_file) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error, load {}, err: {:?}", config_file, e);
            return;
        }
    };
    println!("config: {:?}",app_config);

    // 初始化日志
    //




    print!("end");
}
