
use lapin::{ConnectionProperties};


#[cfg(target_os = "windows")]
pub fn init_conn_props() -> ConnectionProperties {
    let mut conn_props = ConnectionProperties::default();
    conn_props.locale = "zh_CN".into();

    conn_props
        .with_executor(tokio_executor_trait::Tokio::current())
}

#[cfg(not(target_os = "windows"))]
pub fn init_conn_props() -> ConnectionProperties {
    let mut conn_props = ConnectionProperties::default();
    conn_props.locale = "zh_CN".into();

    conn_props
        .with_executor(tokio_executor_trait::Tokio::current())
        .with_reactor(tokio_executor_trait::Tokio::current())
}