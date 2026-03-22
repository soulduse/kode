use std::collections::{HashMap, HashSet};

use crate::types::BeanGraph;

impl BeanGraph {
    /// Find beans that depend on the given bean.
    pub fn find_dependents(&self, bean_id: &str) -> Vec<&str> {
        self.edges
            .iter()
            .filter(|e| e.to == bean_id)
            .map(|e| e.from.as_str())
            .collect()
    }

    /// Find beans that the given bean depends on.
    pub fn find_dependencies(&self, bean_id: &str) -> Vec<&str> {
        self.edges
            .iter()
            .filter(|e| e.from == bean_id)
            .map(|e| e.to.as_str())
            .collect()
    }

    /// Detect circular dependencies using DFS.
    pub fn detect_cycles(&self) -> Vec<Vec<String>> {
        use std::collections::{HashMap, HashSet};

        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
        for edge in &self.edges {
            adj.entry(edge.from.as_str())
                .or_default()
                .push(edge.to.as_str());
        }

        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut on_stack = HashSet::new();
        let mut stack = Vec::new();

        for node in &self.nodes {
            if !visited.contains(node.id.as_str()) {
                dfs(
                    node.id.as_str(),
                    &adj,
                    &mut visited,
                    &mut on_stack,
                    &mut stack,
                    &mut cycles,
                );
            }
        }

        cycles
    }

    /// Render the graph as a text tree for display.
    pub fn render_tree(&self, root: &str) -> String {
        let mut lines = Vec::new();
        lines.push(root.to_string());
        let deps = self.find_dependencies(root);
        for (i, dep) in deps.iter().enumerate() {
            let prefix = if i == deps.len() - 1 { "└── " } else { "├── " };
            lines.push(format!("{}{}", prefix, dep));
            let sub_deps = self.find_dependencies(dep);
            for (j, sub) in sub_deps.iter().enumerate() {
                let branch = if i == deps.len() - 1 { "    " } else { "│   " };
                let sub_prefix = if j == sub_deps.len() - 1 {
                    "└── "
                } else {
                    "├── "
                };
                lines.push(format!("{}{}{}", branch, sub_prefix, sub));
            }
        }
        lines.join("\n")
    }
}

fn dfs<'a>(
    node: &'a str,
    adj: &HashMap<&str, Vec<&'a str>>,
    visited: &mut HashSet<&'a str>,
    on_stack: &mut HashSet<&'a str>,
    stack: &mut Vec<&'a str>,
    cycles: &mut Vec<Vec<String>>,
) {
    visited.insert(node);
    on_stack.insert(node);
    stack.push(node);

    if let Some(neighbors) = adj.get(node) {
        for &next in neighbors {
            if !visited.contains(next) {
                dfs(next, adj, visited, on_stack, stack, cycles);
            } else if on_stack.contains(next) {
                // Found a cycle
                let cycle_start = stack.iter().position(|&n| n == next).unwrap();
                let cycle: Vec<String> = stack[cycle_start..].iter().map(|s| s.to_string()).collect();
                cycles.push(cycle);
            }
        }
    }

    stack.pop();
    on_stack.remove(node);
}

#[cfg(test)]
mod tests {
    use crate::types::{BeanType, GraphEdge, GraphNode};

    use super::*;

    fn test_graph() -> BeanGraph {
        BeanGraph {
            nodes: vec![
                GraphNode { id: "a".into(), bean_type: BeanType::Service, qualified_name: "A".into() },
                GraphNode { id: "b".into(), bean_type: BeanType::Repository, qualified_name: "B".into() },
                GraphNode { id: "c".into(), bean_type: BeanType::Service, qualified_name: "C".into() },
            ],
            edges: vec![
                GraphEdge { from: "a".into(), to: "b".into() },
                GraphEdge { from: "a".into(), to: "c".into() },
                GraphEdge { from: "c".into(), to: "b".into() },
            ],
        }
    }

    #[test]
    fn find_dependents_and_dependencies() {
        let g = test_graph();
        assert_eq!(g.find_dependencies("a"), vec!["b", "c"]);
        assert_eq!(g.find_dependents("b"), vec!["a", "c"]);
    }

    #[test]
    fn no_cycles() {
        let g = test_graph();
        assert!(g.detect_cycles().is_empty());
    }

    #[test]
    fn detect_cycle() {
        let g = BeanGraph {
            nodes: vec![
                GraphNode { id: "x".into(), bean_type: BeanType::Service, qualified_name: "X".into() },
                GraphNode { id: "y".into(), bean_type: BeanType::Service, qualified_name: "Y".into() },
            ],
            edges: vec![
                GraphEdge { from: "x".into(), to: "y".into() },
                GraphEdge { from: "y".into(), to: "x".into() },
            ],
        };
        let cycles = g.detect_cycles();
        assert!(!cycles.is_empty());
    }

    #[test]
    fn render_tree_output() {
        let g = test_graph();
        let tree = g.render_tree("a");
        assert!(tree.contains("a"));
        assert!(tree.contains("b"));
        assert!(tree.contains("c"));
    }
}
