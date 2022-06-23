use lapin::ConnectionProperties;

#[cfg(target_os = "windows")]
pub fn init_conn_props() -> ConnectionProperties {
    ConnectionProperties::default().with_executor(tokio_executor_trait::Tokio::current())
}

#[cfg(not(target_os = "windows"))]
pub fn init_conn_props() -> ConnectionProperties {
    ConnectionProperties::default()
        .with_executor(tokio_executor_trait::Tokio::current())
        .with_reactor(tokio_reactor_trait::Tokio);
}
