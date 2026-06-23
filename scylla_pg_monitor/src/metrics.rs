use std::{collections::BTreeMap, io, sync::Arc};

use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpListener,
    sync::RwLock,
};

#[derive(Clone, Default)]
pub struct MetricsState {
    task_counts: Arc<RwLock<BTreeMap<String, i64>>>,
}

impl MetricsState {
    pub async fn update_task_counts(&self, counts: Vec<(String, i64)>) {
        let mut task_counts = self.task_counts.write().await;
        task_counts.clear();
        task_counts.extend(counts);
    }

    pub async fn render(&self) -> String {
        let task_counts = self.task_counts.read().await;
        let mut output = String::from("# HELP scylla_task_count Number of tasks grouped by status.\n# TYPE scylla_task_count gauge\n");

        for (status, count) in task_counts.iter() {
            output.push_str(&format!("scylla_task_count{{status=\"{}\"}} {}\n", escape_label_value(status), count));
        }

        output
    }
}

pub async fn serve_metrics(state: MetricsState, host: String, port: u16, path: String) -> io::Result<()> {
    let listener = TcpListener::bind((host.as_str(), port)).await?;
    log::info!("serving metrics on http://{host}:{port}{path}");

    loop {
        let (mut socket, _) = listener.accept().await?;
        let state_clone = state.clone();
        let path_clone = path.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(&mut socket, state_clone, &path_clone).await {
                log::error!("error serving metrics request {e}");
            }
        });
    }
}

async fn handle_connection<S>(socket: &mut S, state: MetricsState, metrics_path: &str) -> io::Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut buffer = [0_u8; 1024];
    let bytes_read = socket.read(&mut buffer).await?;
    if bytes_read == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let first_line = request.lines().next().unwrap_or_default();

    let (status_line, content_type, body) = match parse_request_path(first_line) {
        Some(path) if path == metrics_path => ("HTTP/1.1 200 OK", "text/plain; version=0.0.4; charset=utf-8", state.render().await),
        Some(_) => ("HTTP/1.1 404 Not Found", "text/plain; charset=utf-8", "not found\n".to_string()),
        None => ("HTTP/1.1 400 Bad Request", "text/plain; charset=utf-8", "bad request\n".to_string()),
    };

    let response = format!(
        "{status_line}\r\ncontent-type: {content_type}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
        body.len()
    );
    socket.write_all(response.as_bytes()).await?;
    socket.shutdown().await
}

fn parse_request_path(request_line: &str) -> Option<&str> {
    let mut parts = request_line.split_whitespace();
    let method = parts.next()?;
    let path = parts.next()?;
    let _http_version = parts.next()?;

    if method == "GET" {
        Some(path)
    } else {
        None
    }
}

fn escape_label_value(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn render_prometheus_task_count_metrics() {
        let state = MetricsState::default();
        state.update_task_counts(vec![("ready".to_string(), 3), ("running".to_string(), 1)]).await;

        let output = state.render().await;
        assert!(output.contains("scylla_task_count{status=\"ready\"} 3"));
        assert!(output.contains("scylla_task_count{status=\"running\"} 1"));
    }

    #[test]
    fn parse_metrics_request_path() {
        assert_eq!(parse_request_path("GET /metrics HTTP/1.1"), Some("/metrics"));
        assert_eq!(parse_request_path("POST /metrics HTTP/1.1"), None);
        assert_eq!(parse_request_path("not-a-request"), None);
    }
}
