use lapin::{Channel, Connection, ConnectionProperties};
use tracing::error;

#[cfg(target_os = "windows")]
pub fn init_conn_props() -> ConnectionProperties {
    ConnectionProperties::default().with_executor(tokio_executor_trait::Tokio::current())
}

#[cfg(not(target_os = "windows"))]
pub fn init_conn_props() -> ConnectionProperties {
    ConnectionProperties::default()
        .with_executor(tokio_executor_trait::Tokio::current())
        .with_reactor(tokio_reactor_trait::Tokio)
}


pub async fn shutdown_rabbitmq(channel: Option<Channel>, conn: Option<Connection>) {
    if let Some(channel) = channel {
        match channel.close(0,"close channel").await {
            Ok(_v) => {

            }
            Err(e) => {
                error!("error, rabbitmq, close channel, err:{:?}",e);
            }
        };
    }

    if let Some(conn) = conn {
        match conn.close(0,"close connection").await {
            Ok(_v) => {

            }
            Err(e) => {
                error!("error, rabbitmq, close connection, err:{:?}",e);
            }
        };
    }

}