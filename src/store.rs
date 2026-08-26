//! The in-memory store: an ontology-validated graph with adjacency indexes.

use std::collections::HashMap;

use crate::error::GraphError;
use crate::graph::{Edge, Node};
use crate::ontology::Ontology;

/// An in-memory property graph whose contents are constrained by an
/// [`Ontology`].
///
/// Every node and edge inserted is checked against the ontology: types must
/// be defined, edge endpoints must exist, and edge domain/range constraints
/// are enforced honoring the subtype hierarchy.
#[derive(Debug, Clone)]
pub struct Graph {
    ontology: Ontology,
    nodes: HashMap<String, Node>,
    edges: HashMap<String, Edge>,
    /// node id -> ids of edges whose source is that node
    outgoing: HashMap<String, Vec<String>>,
    /// node id -> ids of edges whose target is that node
    incoming: HashMap<String, Vec<String>>,
}

impl Graph {
    /// Create an empty graph over the given ontology.
    ///
    /// The ontology is validated first; an invalid ontology is rejected.
    pub fn new(ontology: Ontology) -> Result<Self, GraphError> {
        ontology.validate()?;
        Ok(Self {
            ontology,
            nodes: HashMap::new(),
            edges: HashMap::new(),
            outgoing: HashMap::new(),
            incoming: HashMap::new(),
        })
    }

    /// The ontology this graph is constrained by.
    pub fn ontology(&self) -> &Ontology {
        &self.ontology
    }

    /// Insert a node.
    ///
    /// Fails if the node's type is not defined in the ontology or if a node
    /// with the same id already exists.
    pub fn insert_node(&mut self, node: Node) -> Result<(), GraphError> {
        if !self.ontology.node_types.contains_key(&node.node_type) {
            return Err(GraphError::UnknownNodeType {
                node: node.id.clone(),
                node_type: node.node_type.clone(),
            });
        }
        if self.nodes.contains_key(&node.id) {
            return Err(GraphError::DuplicateId(node.id));
        }
        self.nodes.insert(node.id.clone(), node);
        Ok(())
    }

    /// Insert an edge.
    ///
    /// Fails if the edge type is undefined, an edge with the same id exists,
    /// either endpoint is missing, the confidence is outside `0.0..=1.0`, or
    /// the endpoints violate the edge type's domain/range constraints
    /// (satisfied by any declared type or a subtype of it).
    pub fn insert_edge(&mut self, edge: Edge) -> Result<(), GraphError> {
        let def = self.ontology.edge_types.get(&edge.edge_type).ok_or_else(|| {
            GraphError::UnknownEdgeType {
                edge: edge.id.clone(),
                edge_type: edge.edge_type.clone(),
            }
        })?;
        if self.edges.contains_key(&edge.id) {
            return Err(GraphError::DuplicateId(edge.id));
        }
        if !(0.0..=1.0).contains(&edge.confidence) {
            return Err(GraphError::InvalidConfidence {
                edge: edge.id.clone(),
                confidence: edge.confidence,
            });
        }
        let source = self.nodes.get(&edge.source).ok_or_else(|| {
            GraphError::MissingEndpoint {
                edge: edge.id.clone(),
                node: edge.source.clone(),
            }
        })?;
        let target = self.nodes.get(&edge.target).ok_or_else(|| {
            GraphError::MissingEndpoint {
                edge: edge.id.clone(),
                node: edge.target.clone(),
            }
        })?;
        let satisfies = |actual: &str, allowed: &[String]| {
            allowed.is_empty()
                || allowed.iter().any(|a| self.ontology.is_subtype_of(actual, a))
        };
        if !satisfies(&source.node_type, &def.source_types) {
            return Err(GraphError::DomainViolation {
                edge: edge.id.clone(),
                edge_type: edge.edge_type.clone(),
                source_type: source.node_type.clone(),
            });
        }
        if !satisfies(&target.node_type, &def.target_types) {
            return Err(GraphError::RangeViolation {
                edge: edge.id.clone(),
                edge_type: edge.edge_type.clone(),
                target_type: target.node_type.clone(),
            });
        }
        self.outgoing
            .entry(edge.source.clone())
            .or_default()
            .push(edge.id.clone());
        self.incoming
            .entry(edge.target.clone())
            .or_default()
            .push(edge.id.clone());
        self.edges.insert(edge.id.clone(), edge);
        Ok(())
    }

    /// Look up a node by id.
    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// Look up an edge by id.
    pub fn edge(&self, id: &str) -> Option<&Edge> {
        self.edges.get(id)
    }

    /// Remove a node by id, returning it.
    ///
    /// Fails if the node does not exist or if any edges are still attached
    /// (removal does not cascade).
    pub fn remove_node(&mut self, id: &str) -> Result<Node, GraphError> {
        if !self.nodes.contains_key(id) {
            return Err(GraphError::UnknownNode(id.to_string()));
        }
        let attached = self.outgoing.get(id).map_or(false, |e| !e.is_empty())
            || self.incoming.get(id).map_or(false, |e| !e.is_empty());
        if attached {
            return Err(GraphError::NodeHasEdges(id.to_string()));
        }
        self.outgoing.remove(id);
        self.incoming.remove(id);
        Ok(self.nodes.remove(id).expect("presence checked above"))
    }

    /// Remove an edge by id, returning it if it existed.
    pub fn remove_edge(&mut self, id: &str) -> Option<Edge> {
        let edge = self.edges.remove(id)?;
        if let Some(out) = self.outgoing.get_mut(&edge.source) {
            out.retain(|e| e != id);
        }
        if let Some(inc) = self.incoming.get_mut(&edge.target) {
            inc.retain(|e| e != id);
        }
        Some(edge)
    }

    /// Outgoing neighbors of a node: each outgoing edge paired with its
    /// target node. Returns an empty list for unknown ids.
    pub fn neighbors(&self, id: &str) -> Vec<(&Edge, &Node)> {
        self.resolve(self.outgoing.get(id), |e| &e.target)
    }

    /// Incoming neighbors of a node: each incoming edge paired with its
    /// source node. Returns an empty list for unknown ids.
    pub fn incoming(&self, id: &str) -> Vec<(&Edge, &Node)> {
        self.resolve(self.incoming.get(id), |e| &e.source)
    }

    fn resolve<'a>(
        &'a self,
        edge_ids: Option<&'a Vec<String>>,
        endpoint: impl Fn(&Edge) -> &String,
    ) -> Vec<(&'a Edge, &'a Node)> {
        edge_ids
            .into_iter()
            .flatten()
            .filter_map(|eid| {
                let edge = self.edges.get(eid)?;
                let node = self.nodes.get(endpoint(edge))?;
                Some((edge, node))
            })
            .collect()
    }

    /// All nodes whose type is `ty` or a subtype of it.
    pub fn nodes_of_type(&self, ty: &str) -> Vec<&Node> {
        self.nodes
            .values()
            .filter(|n| self.ontology.is_subtype_of(&n.node_type, ty))
            .collect()
    }

    /// Number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// True if the graph contains no nodes and no edges.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }

    /// Iterate over all nodes (unordered).
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    /// Iterate over all edges (unordered).
    pub fn edges(&self) -> impl Iterator<Item = &Edge> {
        self.edges.values()
    }
}
