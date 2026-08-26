//! The data layer: typed nodes and edges with provenance.
//!
//! A [`Node`] is a typed entity; an [`Edge`] is a typed, directed assertion
//! between two nodes. Edge `properties` carry semantic data belonging to the
//! relationship itself, while `annotations` carry metadata *about the
//! assertion* (the AnnotatedAxiom pattern) — e.g. who asserted it, review
//! status, or extraction context.

use serde::{Deserialize, Serialize};

/// A typed entity in the graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    /// Unique identifier within the graph.
    pub id: String,
    /// The node's type; must name a `NodeTypeDef` in the graph's ontology.
    #[serde(rename = "type")]
    pub node_type: String,
    /// Human-readable name.
    pub name: String,
    /// Human-readable description.
    #[serde(default)]
    pub description: String,
    /// Optional identifier in an external system.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    /// Arbitrary structured metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
    /// Creation timestamp (opaque string; the crate does not impose a format).
    #[serde(default)]
    pub created_at: String,
    /// Last-update timestamp (opaque string).
    #[serde(default)]
    pub updated_at: String,
}

impl Node {
    /// Create a node with the given id, type, and name.
    ///
    /// Description, metadata, and timestamps default to empty; use the
    /// `with_*` builders to set them.
    pub fn new(id: impl Into<String>, node_type: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            node_type: node_type.into(),
            name: name.into(),
            description: String::new(),
            external_id: None,
            metadata: serde_json::Value::Null,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    /// Set the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set the external identifier.
    pub fn with_external_id(mut self, external_id: impl Into<String>) -> Self {
        self.external_id = Some(external_id.into());
        self
    }

    /// Set the metadata value.
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set the creation and update timestamps.
    pub fn with_timestamps(
        mut self,
        created_at: impl Into<String>,
        updated_at: impl Into<String>,
    ) -> Self {
        self.created_at = created_at.into();
        self.updated_at = updated_at.into();
        self
    }
}

/// Where an assertion came from.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    /// The originating source (document, conversation, tool, ...).
    #[serde(default)]
    pub source: String,
    /// Optional location within the source (page, offset, message id, ...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// The agent or user that created the assertion.
    #[serde(default)]
    pub created_by: String,
}

impl Provenance {
    /// Create provenance from a source and creator.
    pub fn new(source: impl Into<String>, created_by: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            location: None,
            created_by: created_by.into(),
        }
    }

    /// Set the location within the source.
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }
}

/// A typed, directed assertion between two nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    /// Unique identifier within the graph.
    pub id: String,
    /// Id of the source node.
    pub source: String,
    /// Id of the target node.
    pub target: String,
    /// The edge's type; must name an `EdgeTypeDef` in the graph's ontology.
    #[serde(rename = "type")]
    pub edge_type: String,
    /// Semantic edge data — properties of the relationship itself.
    #[serde(default)]
    pub properties: serde_json::Value,
    /// Where this assertion came from.
    #[serde(default)]
    pub provenance: Provenance,
    /// Confidence in the assertion, in `0.0..=1.0`.
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    /// Metadata about the assertion itself (the AnnotatedAxiom pattern).
    #[serde(default)]
    pub annotations: serde_json::Value,
    /// Creation timestamp (opaque string).
    #[serde(default)]
    pub created_at: String,
}

fn default_confidence() -> f64 {
    1.0
}

impl Edge {
    /// Create an edge with the given id, endpoints, and type.
    ///
    /// Confidence defaults to `1.0`; properties, provenance, annotations,
    /// and the timestamp default to empty. Use the `with_*` builders.
    pub fn new(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
        edge_type: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            source: source.into(),
            target: target.into(),
            edge_type: edge_type.into(),
            properties: serde_json::Value::Null,
            provenance: Provenance::default(),
            confidence: 1.0,
            annotations: serde_json::Value::Null,
            created_at: String::new(),
        }
    }

    /// Set the semantic properties.
    pub fn with_properties(mut self, properties: serde_json::Value) -> Self {
        self.properties = properties;
        self
    }

    /// Set the provenance.
    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = provenance;
        self
    }

    /// Set the confidence (validated on insertion into a graph).
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }

    /// Set the annotations.
    pub fn with_annotations(mut self, annotations: serde_json::Value) -> Self {
        self.annotations = annotations;
        self
    }

    /// Set the creation timestamp.
    pub fn with_created_at(mut self, created_at: impl Into<String>) -> Self {
        self.created_at = created_at.into();
        self
    }
}
