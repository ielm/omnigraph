//! A typed property-graph substrate for agent knowledge representation.
//!
//! `omnigraph` is the knowledge-representation substrate of the Omicron
//! project: an agent memory graph for building arbitrary semantic
//! relationships across typed nodes. An [`Ontology`] declares the vocabulary
//! — node types, edge types, subtype hierarchies, and domain/range
//! constraints — and a [`Graph`] stores nodes and edges validated against
//! that vocabulary. Edges carry [`Provenance`], a confidence score, semantic
//! `properties`, and assertion-level `annotations`.
//!
//! This is a pre-release (v0.0.1) establishing the crate; the API will
//! change.
//!
//! # Example
//!
//! ```
//! use omnigraph::{Edge, EdgeTypeDef, Graph, Node, NodeTypeDef, Ontology};
//!
//! let ontology = Ontology::new("demo")
//!     .with_node_type("person", NodeTypeDef::new("Person", "A human"))
//!     .with_node_type("org", NodeTypeDef::new("Org", "An organization"))
//!     .with_edge_type(
//!         "works_at",
//!         EdgeTypeDef::new("works at", "Employment")
//!             .with_source("person")
//!             .with_target("org"),
//!     );
//!
//! let mut graph = Graph::new(ontology)?;
//! graph.insert_node(Node::new("ada", "person", "Ada Lovelace"))?;
//! graph.insert_node(Node::new("acme", "org", "Acme"))?;
//! graph.insert_edge(Edge::new("e1", "ada", "acme", "works_at").with_confidence(0.9))?;
//!
//! let neighbors = graph.neighbors("ada");
//! assert_eq!(neighbors.len(), 1);
//! assert_eq!(neighbors[0].1.name, "Acme");
//! # Ok::<(), omnigraph::GraphError>(())
//! ```
#![warn(missing_docs)]

mod error;
mod graph;
mod ontology;
mod store;

pub use error::{GraphError, OntologyError};
pub use graph::{Edge, Node, Provenance};
pub use ontology::{EdgeTypeDef, NodeTypeDef, Ontology, OntologyMeta, PropertyDef};
pub use store::Graph;
