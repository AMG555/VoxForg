use std::collections::{HashMap, HashSet, VecDeque};
use voxforg_core::error::{Result, VoxForgError};
use voxforg_core::models::PipelineDefinition;

pub struct GraphValidator;

impl GraphValidator {
    pub fn topological_sort(pipeline: &PipelineDefinition) -> Result<Vec<String>> {
        if pipeline.nodes.len() > 500 {
            return Err(VoxForgError::PipelineValidation(
                "Pipeline exceeds maximum allowed node limit (500 nodes)".to_string(),
            ));
        }

        if pipeline.edges.len() > 2000 {
            return Err(VoxForgError::PipelineValidation(
                "Pipeline exceeds maximum allowed edge limit (2000 edges)".to_string(),
            ));
        }

        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();

        for node in &pipeline.nodes {
            in_degree.insert(node.id.clone(), 0);
            adjacency.insert(node.id.clone(), Vec::new());
        }

        for edge in &pipeline.edges {
            if !in_degree.contains_key(&edge.from_node) {
                return Err(VoxForgError::PipelineValidation(format!(
                    "Edge references nonexistent from_node: '{}'",
                    edge.from_node
                )));
            }
            if !in_degree.contains_key(&edge.to_node) {
                return Err(VoxForgError::PipelineValidation(format!(
                    "Edge references nonexistent to_node: '{}'",
                    edge.to_node
                )));
            }

            adjacency.get_mut(&edge.from_node).unwrap().push(edge.to_node.clone());
            *in_degree.get_mut(&edge.to_node).unwrap() += 1;
        }

        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut order = Vec::new();
        let mut visited = HashSet::new();

        while let Some(node_id) = queue.pop_front() {
            visited.insert(node_id.clone());
            order.push(node_id.clone());

            if let Some(neighbors) = adjacency.get(&node_id) {
                for neighbor in neighbors {
                    let deg = in_degree.get_mut(neighbor).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        if order.len() != pipeline.nodes.len() {
            return Err(VoxForgError::PipelineValidation(
                "Cycle detected in pipeline graph! Graphs must be strictly directed acyclic (DAG).".to_string(),
            ));
        }

        Ok(order)
    }
}
