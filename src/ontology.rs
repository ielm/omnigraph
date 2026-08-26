//! The schema vocabulary: node types, edge types, and property definitions.
//!
//! An [`Ontology`] declares what may exist in a [`Graph`](crate::Graph):
//! which node types are known, which edge types connect them, and what
//! properties each carries. Node and edge types form independent
//! single-inheritance hierarchies via their `parent` fields.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::error::OntologyError;

/// Metadata identifying an ontology.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OntologyMeta {
    /// Monotonically increasing schema version.
    pub version: u32,
    /// Human-readable label for the ontology.
    pub label: String,
    /// Optional namespace (e.g. a URI prefix) for the ontology's terms.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}

/// Definition of a single property on a node or edge type.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PropertyDef {
    /// The property's value type (e.g. `"string"`, `"number"`, `"boolean"`).
    #[serde(rename = "type")]
    pub prop_type: String,
    /// If non-empty, the closed set of permitted values.
    #[serde(rename = "enum", default, skip_serializing_if = "Vec::is_empty")]
    pub enum_values: Vec<String>,
    /// Optional human-readable description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether the property must be present.
    #[serde(default)]
    pub required: bool,
    /// Optional default value applied when the property is absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
}

/// Definition of a node type in the ontology.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NodeTypeDef {
    /// Optional parent node type for subtype inheritance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// Human-readable label.
    pub label: String,
    /// Human-readable description.
    pub description: String,
    /// Properties declared on this node type.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, PropertyDef>,
}

impl NodeTypeDef {
    /// Create a node type definition with the given label and description.
    pub fn new(label: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            description: description.into(),
            ..Self::default()
        }
    }

    /// Set the parent node type.
    pub fn with_parent(mut self, parent: impl Into<String>) -> Self {
        self.parent = Some(parent.into());
        self
    }

    /// Add a property definition.
    pub fn with_property(mut self, name: impl Into<String>, def: PropertyDef) -> Self {
        self.properties.insert(name.into(), def);
        self
    }
}

/// Definition of an edge type in the ontology.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EdgeTypeDef {
    /// Optional parent edge type for subtype inheritance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// Human-readable label.
    pub label: String,
    /// Human-readable description.
    pub description: String,
    /// Optional inverse edge type (e.g. `works_at` / `employs`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inverse: Option<String>,
    /// Properties declared on this edge type.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, PropertyDef>,
    /// Permitted source node types (domain). Empty means unconstrained.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_types: Vec<String>,
    /// Permitted target node types (range). Empty means unconstrained.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_types: Vec<String>,
}

impl EdgeTypeDef {
    /// Create an edge type definition with the given label and description.
    pub fn new(label: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            description: description.into(),
            ..Self::default()
        }
    }

    /// Set the parent edge type.
    pub fn with_parent(mut self, parent: impl Into<String>) -> Self {
        self.parent = Some(parent.into());
        self
    }

    /// Set the inverse edge type.
    pub fn with_inverse(mut self, inverse: impl Into<String>) -> Self {
        self.inverse = Some(inverse.into());
        self
    }

    /// Add a permitted source node type (domain constraint).
    pub fn with_source(mut self, ty: impl Into<String>) -> Self {
        self.source_types.push(ty.into());
        self
    }

    /// Add a permitted target node type (range constraint).
    pub fn with_target(mut self, ty: impl Into<String>) -> Self {
        self.target_types.push(ty.into());
        self
    }

    /// Add a property definition.
    pub fn with_property(mut self, name: impl Into<String>, def: PropertyDef) -> Self {
        self.properties.insert(name.into(), def);
        self
    }
}

/// A complete schema: node type and edge type vocabularies plus metadata.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Ontology {
    /// Ontology metadata (version, label, namespace).
    pub meta: OntologyMeta,
    /// Node type definitions, keyed by type name.
    #[serde(default)]
    pub node_types: BTreeMap<String, NodeTypeDef>,
    /// Edge type definitions, keyed by type name.
    #[serde(default)]
    pub edge_types: BTreeMap<String, EdgeTypeDef>,
}

impl Ontology {
    /// Create an empty ontology with the given label at version 1.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            meta: OntologyMeta {
                version: 1,
                label: label.into(),
                namespace: None,
            },
            ..Self::default()
        }
    }

    /// Define (or replace) a node type.
    pub fn with_node_type(mut self, name: impl Into<String>, def: NodeTypeDef) -> Self {
        self.node_types.insert(name.into(), def);
        self
    }

    /// Define (or replace) an edge type.
    pub fn with_edge_type(mut self, name: impl Into<String>, def: EdgeTypeDef) -> Self {
        self.edge_types.insert(name.into(), def);
        self
    }

    /// True if `ty` is `ancestor` or a (transitive) subtype of it.
    ///
    /// Node type and edge type hierarchies are consulted independently;
    /// a name is looked up first among node types, then among edge types.
    /// Unknown types are only subtypes of themselves.
    pub fn is_subtype_of(&self, ty: &str, ancestor: &str) -> bool {
        if ty == ancestor {
            return true;
        }
        let mut seen = BTreeSet::new();
        let mut current = ty;
        while seen.insert(current.to_string()) {
            let parent = self
                .node_types
                .get(current)
                .and_then(|d| d.parent.as_deref())
                .or_else(|| self.edge_types.get(current).and_then(|d| d.parent.as_deref()));
            match parent {
                Some(p) if p == ancestor => return true,
                Some(p) => current = p,
                None => return false,
            }
        }
        false // cycle encountered without reaching `ancestor`
    }

    /// Check the ontology for structural soundness.
    ///
    /// Verifies that every `parent` reference names an existing type of the
    /// same kind, that every edge `inverse` names an existing edge type, and
    /// that no parent chain forms a cycle.
    pub fn validate(&self) -> Result<(), OntologyError> {
        for (name, def) in &self.node_types {
            if let Some(parent) = &def.parent {
                if !self.node_types.contains_key(parent) {
                    return Err(OntologyError::UnknownParent {
                        child: name.clone(),
                        parent: parent.clone(),
                    });
                }
            }
        }
        for (name, def) in &self.edge_types {
            if let Some(parent) = &def.parent {
                if !self.edge_types.contains_key(parent) {
                    return Err(OntologyError::UnknownParent {
                        child: name.clone(),
                        parent: parent.clone(),
                    });
                }
            }
            if let Some(inverse) = &def.inverse {
                if !self.edge_types.contains_key(inverse) {
                    return Err(OntologyError::UnknownInverse {
                        edge_type: name.clone(),
                        inverse: inverse.clone(),
                    });
                }
            }
        }
        Self::check_cycles(self.node_types.iter().map(|(n, d)| (n, d.parent.as_deref())))?;
        Self::check_cycles(self.edge_types.iter().map(|(n, d)| (n, d.parent.as_deref())))?;
        Ok(())
    }

    /// Detect cycles in a parent relation given as `(name, parent)` pairs.
    fn check_cycles<'a>(
        pairs: impl Iterator<Item = (&'a String, Option<&'a str>)>,
    ) -> Result<(), OntologyError> {
        let parents: BTreeMap<&str, &str> = pairs
            .filter_map(|(name, parent)| parent.map(|p| (name.as_str(), p)))
            .collect();
        for start in parents.keys() {
            let mut seen = BTreeSet::new();
            let mut current = *start;
            while let Some(parent) = parents.get(current) {
                if !seen.insert(current) {
                    return Err(OntologyError::ParentCycle(current.to_string()));
                }
                current = parent;
            }
        }
        Ok(())
    }
}
