use build_time::build_time_local;
use clap::Command;

const APP_NAME : &str = "sync_broker";
const APP_VER_NUM: &str = "0.1.0";

#[tokio::main]
async fn main() {
    let build_time = build_time_local!("%Y%m%d_%H%M%S");
    let app_ver_text = format!("{} ({})",APP_VER_NUM,build_time);

    let cli_app = Command::new(APP_NAME)
        .author("tomtong")
        .version(app_ver_text)
        .about("a server provides sync service for box agent");

}
