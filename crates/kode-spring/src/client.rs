use std::io;

use kode_lsp::LspClient;

use crate::types::{BeanGraph, GradleTask, RestEndpoint, SpringBean};

/// Fetch all Spring beans from the LSP server.
pub async fn fetch_beans(client: &mut LspClient) -> io::Result<Vec<SpringBean>> {
    let result = client.send_request("spring/beans", None).await?;
    if result.is_null() {
        return Ok(vec![]);
    }
    serde_json::from_value(result).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Fetch all REST endpoints from the LSP server.
pub async fn fetch_endpoints(client: &mut LspClient) -> io::Result<Vec<RestEndpoint>> {
    let result = client.send_request("spring/endpoints", None).await?;
    if result.is_null() {
        return Ok(vec![]);
    }
    serde_json::from_value(result).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Fetch the bean dependency graph from the LSP server.
pub async fn fetch_bean_graph(client: &mut LspClient) -> io::Result<BeanGraph> {
    let result = client.send_request("spring/beanGraph", None).await?;
    serde_json::from_value(result).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Fetch available Gradle tasks from the LSP server.
pub async fn fetch_gradle_tasks(client: &mut LspClient) -> io::Result<Vec<GradleTask>> {
    let result = client.send_request("spring/gradleTasks", None).await?;
    if result.is_null() {
        return Ok(vec![]);
    }
    serde_json::from_value(result).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Run a Gradle task via the LSP server.
pub async fn run_gradle_task(client: &mut LspClient, task_name: &str) -> io::Result<()> {
    let params = serde_json::json!({"task": task_name});
    client
        .send_request("spring/runTask", Some(params))
        .await?;
    Ok(())
}
