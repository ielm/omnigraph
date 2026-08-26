//! Error types for ontology validation and graph mutation.

use thiserror::Error;

/// Errors produced while validating an [`Ontology`](crate::Ontology).
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum OntologyError {
    /// A type references a parent that is not defined in the ontology.
    #[error("type `{child}` references unknown parent `{parent}`")]
    UnknownParent {
        /// The type whose `parent` field is dangling.
        child: String,
        /// The missing parent type name.
        parent: String,
    },

    /// An edge type references an inverse edge type that is not defined.
    #[error("edge type `{edge_type}` references unknown inverse `{inverse}`")]
    UnknownInverse {
        /// The edge type whose `inverse` field is dangling.
        edge_type: String,
        /// The missing inverse edge type name.
        inverse: String,
    },

    /// A parent chain loops back on itself.
    #[error("parent cycle detected involving type `{0}`")]
    ParentCycle(String),
}

/// Errors produced while mutating a [`Graph`](crate::Graph).
#[derive(Debug, Error, Clone, PartialEq)]
pub enum GraphError {
    /// The graph's ontology failed validation.
    #[error(transparent)]
    Ontology(#[from] OntologyError),

    /// A node's type is not defined in the ontology.
    #[error("node `{node}` has unknown node type `{node_type}`")]
    UnknownNodeType {
        /// The offending node id.
        node: String,
        /// The undefined node type name.
        node_type: String,
    },

    /// An edge's type is not defined in the ontology.
    #[error("edge `{edge}` has unknown edge type `{edge_type}`")]
    UnknownEdgeType {
        /// The offending edge id.
        edge: String,
        /// The undefined edge type name.
        edge_type: String,
    },

    /// An edge references a source or target node that is not in the graph.
    #[error("edge `{edge}` references missing endpoint node `{node}`")]
    MissingEndpoint {
        /// The offending edge id.
        edge: String,
        /// The missing node id.
        node: String,
    },

    /// An edge's source node violates the edge type's domain constraint.
    #[error(
        "edge `{edge}` of type `{edge_type}` has source of type `{source_type}`, \
         which is not permitted by the edge type's source constraints"
    )]
    DomainViolation {
        /// The offending edge id.
        edge: String,
        /// The edge type whose domain was violated.
        edge_type: String,
        /// The actual type of the source node.
        source_type: String,
    },

    /// An edge's target node violates the edge type's range constraint.
    #[error(
        "edge `{edge}` of type `{edge_type}` has target of type `{target_type}`, \
         which is not permitted by the edge type's target constraints"
    )]
    RangeViolation {
        /// The offending edge id.
        edge: String,
        /// The edge type whose range was violated.
        edge_type: String,
        /// The actual type of the target node.
        target_type: String,
    },

    /// A node or edge with the same id already exists in the graph.
    #[error("an element with id `{0}` already exists")]
    DuplicateId(String),

    /// An edge's confidence lies outside the closed interval `0.0..=1.0`.
    #[error("edge `{edge}` has confidence {confidence}, expected 0.0..=1.0")]
    InvalidConfidence {
        /// The offending edge id.
        edge: String,
        /// The out-of-range confidence value.
        confidence: f64,
    },

    /// A node cannot be removed because edges are still attached to it.
    #[error("node `{0}` still has edges attached and cannot be removed")]
    NodeHasEdges(String),

    /// The referenced node does not exist in the graph.
    #[error("no node with id `{0}` exists")]
    UnknownNode(String),
}
