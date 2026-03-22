use serde::{Deserialize, Serialize};

/// Type of Spring bean.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BeanType {
    Component,
    Service,
    Repository,
    Controller,
    RestController,
    Configuration,
    BeanMethod,
}

/// A Spring bean discovered by the indexer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpringBean {
    pub name: String,
    pub qualified_name: String,
    pub bean_type: BeanType,
    pub file_uri: String,
    pub line: u32,
    pub character: u32,
    pub dependencies: Vec<String>,
    pub scope: String,
}

/// HTTP method for REST endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Options,
    Head,
}

/// A REST endpoint discovered by the indexer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestEndpoint {
    pub method: HttpMethod,
    pub path: String,
    pub handler_class: String,
    pub handler_method: String,
    pub file_uri: String,
    pub line: u32,
    pub character: u32,
    pub parameters: Vec<EndpointParam>,
}

/// A parameter of a REST endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointParam {
    pub name: String,
    pub param_type: String,
    pub source: ParamSource,
}

/// Source of an endpoint parameter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ParamSource {
    Path,
    Query,
    Body,
    Header,
}

/// A node in the bean dependency graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphNode {
    pub id: String,
    pub bean_type: BeanType,
    pub qualified_name: String,
}

/// An edge in the bean dependency graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
}

/// Bean dependency graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeanGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

/// A Gradle task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradleTask {
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub group: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_bean() {
        let bean = SpringBean {
            name: "userService".into(),
            qualified_name: "com.example.UserService".into(),
            bean_type: BeanType::Service,
            file_uri: "file:///src/UserService.kt".into(),
            line: 10,
            character: 0,
            dependencies: vec!["userRepository".into()],
            scope: "singleton".into(),
        };
        let json = serde_json::to_string(&bean).unwrap();
        assert!(json.contains("userService"));
        assert!(json.contains("SERVICE"));
    }

    #[test]
    fn serialize_endpoint() {
        let endpoint = RestEndpoint {
            method: HttpMethod::Get,
            path: "/api/users".into(),
            handler_class: "UserController".into(),
            handler_method: "getUsers".into(),
            file_uri: "file:///src/UserController.kt".into(),
            line: 15,
            character: 4,
            parameters: vec![],
        };
        let json = serde_json::to_string(&endpoint).unwrap();
        assert!(json.contains("/api/users"));
        assert!(json.contains("GET"));
    }

    #[test]
    fn deserialize_bean_graph() {
        let json = r#"{
            "nodes": [{"id": "userService", "beanType": "SERVICE", "qualifiedName": "com.example.UserService"}],
            "edges": [{"from": "userService", "to": "userRepository"}]
        }"#;
        let graph: BeanGraph = serde_json::from_str(json).unwrap();
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.edges.len(), 1);
    }
}
