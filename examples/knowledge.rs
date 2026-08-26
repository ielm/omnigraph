//! A tiny knowledge-graph demo: define an ontology, assert some facts,
//! and walk the neighborhood of a node.
//!
//! Run with: `cargo run --example knowledge`

use omnigraph::{Edge, EdgeTypeDef, Graph, GraphError, Node, NodeTypeDef, Ontology, Provenance};
use serde_json::json;

fn main() -> Result<(), GraphError> {
    let ontology = Ontology::new("workplace")
        .with_node_type("entity", NodeTypeDef::new("Entity", "Root entity type"))
        .with_node_type(
            "person",
            NodeTypeDef::new("Person", "A human").with_parent("entity"),
        )
        .with_node_type(
            "org",
            NodeTypeDef::new("Org", "An organization").with_parent("entity"),
        )
        .with_edge_type(
            "works_at",
            EdgeTypeDef::new("works at", "Employment relation")
                .with_source("person")
                .with_target("org"),
        )
        .with_edge_type(
            "knows",
            EdgeTypeDef::new("knows", "Acquaintance")
                .with_source("person")
                .with_target("person"),
        );

    let mut graph = Graph::new(ontology)?;

    graph.insert_node(Node::new("ada", "person", "Ada Lovelace"))?;
    graph.insert_node(Node::new("charles", "person", "Charles Babbage"))?;
    graph.insert_node(Node::new("engine-co", "org", "Analytical Engine Co."))?;

    graph.insert_edge(
        Edge::new("e1", "ada", "engine-co", "works_at")
            .with_confidence(0.95)
            .with_properties(json!({ "role": "programmer" }))
            .with_provenance(Provenance::new("founding-notes", "ivan")),
    )?;
    graph.insert_edge(
        Edge::new("e2", "ada", "charles", "knows")
            .with_annotations(json!({ "asserted_by": "biographer" })),
    )?;

    println!(
        "graph: {} nodes, {} edges",
        graph.node_count(),
        graph.edge_count()
    );

    let ada = graph.node("ada").expect("ada was inserted");
    println!("neighbors of {}:", ada.name);
    for (edge, node) in graph.neighbors("ada") {
        println!(
            "  -[{} @ {:.2}]-> {}",
            edge.edge_type, edge.confidence, node.name
        );
    }

    println!("entities (via subtype query):");
    for node in graph.nodes_of_type("entity") {
        println!("  {} ({})", node.name, node.node_type);
    }

    Ok(())
}
