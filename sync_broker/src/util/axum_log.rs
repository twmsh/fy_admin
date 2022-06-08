use std::net::SocketAddr;

use axum::body::HttpBody;
use axum::extract::ConnectInfo;
use axum::http::{HeaderValue, Request, StatusCode};
use axum::http::header::{USER_AGENT, REFERER, HeaderName};
use axum::middleware::Next;
use axum::response::IntoResponse;
use chrono::Local;

use hyper::HeaderMap;

fn get_header_value(headers:  &HeaderMap<HeaderValue>, name: HeaderName) -> String {
    match headers.get(name) {
        None => { "-".to_string() }
        Some(v) => {
            match v.to_str() {
                Ok(vv) => {
                    vv.to_string()
                }
                Err(_) => { "-".to_string() }
            }
        }
    }
}

pub async fn access_log<B>(req: Request<B>, next: Next<B>, f: fn(String))
                           -> Result<impl IntoResponse, (StatusCode, String)> {
    // 167.248.133.52 - - [31/Oct/2020:03:43:28 +0800] "GET /api/v1/label/__name__/values HTTP/1.1" 404 1098

    let now = Local::now().format("%d/%b/%Y:%T %z").to_string();
    let ip = match req.extensions().get::<ConnectInfo<SocketAddr>>() {
        None => {
            "-".to_string()
        }
        Some(conn) => {
            conn.0.ip().to_string()
        }
    };
    let method = req.method().clone();
    let uri = req.uri().clone();
    let version = req.version();
    let ua =get_header_value(req.headers(),USER_AGENT);
    let refer =get_header_value(req.headers(),REFERER);

    let res = next.run(req).await;

    let status = res.status().as_u16();
    let size_hint = match res.size_hint().upper() {
        None => { "-".to_string() }
        Some(size) => { size.to_string() }
    };
    let log_line = format!(r#"{} - - [{}] "{} {} {:?}" {} {} "{}" "{}""#, ip, now, method, uri, version, status, size_hint, refer, ua);
    f(log_line);
    Ok(res)
}


