//! Integration tests for the ontology-constrained in-memory graph.

use omnigraph::{
    Edge, EdgeTypeDef, Graph, GraphError, Node, NodeTypeDef, Ontology, OntologyError, Provenance,
};

/// Entity <- Person, Entity <- Org; works_at: Person -> Org;
/// related_to: unconstrained, with inverse related_to.
fn test_ontology() -> Ontology {
    Ontology::new("test")
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
            "related_to",
            EdgeTypeDef::new("related to", "Generic symmetric relation")
                .with_inverse("related_to"),
        )
}

fn populated_graph() -> Graph {
    let mut g = Graph::new(test_ontology()).unwrap();
    g.insert_node(Node::new("ada", "person", "Ada Lovelace")).unwrap();
    g.insert_node(Node::new("acme", "org", "Acme Corp")).unwrap();
    g
}

#[test]
fn valid_inserts_succeed() {
    let mut g = populated_graph();
    let edge = Edge::new("e1", "ada", "acme", "works_at")
        .with_confidence(0.95)
        .with_provenance(Provenance::new("hr-db", "ivan").with_location("row 42"));
    g.insert_edge(edge).unwrap();
    assert_eq!(g.node_count(), 2);
    assert_eq!(g.edge_count(), 1);
    assert!(!g.is_empty());
    assert_eq!(g.node("ada").unwrap().name, "Ada Lovelace");
    assert_eq!(g.edge("e1").unwrap().edge_type, "works_at");
}

#[test]
fn unknown_types_rejected() {
    let mut g = populated_graph();
    let err = g.insert_node(Node::new("x", "spaceship", "X")).unwrap_err();
    assert!(matches!(err, GraphError::UnknownNodeType { .. }));
    let err = g.insert_edge(Edge::new("e1", "ada", "acme", "pilots")).unwrap_err();
    assert!(matches!(err, GraphError::UnknownEdgeType { .. }));
}

#[test]
fn duplicate_ids_rejected() {
    let mut g = populated_graph();
    let err = g.insert_node(Node::new("ada", "person", "Other Ada")).unwrap_err();
    assert_eq!(err, GraphError::DuplicateId("ada".into()));
    g.insert_edge(Edge::new("e1", "ada", "acme", "works_at")).unwrap();
    let err = g.insert_edge(Edge::new("e1", "ada", "acme", "works_at")).unwrap_err();
    assert_eq!(err, GraphError::DuplicateId("e1".into()));
}

#[test]
fn domain_violation_rejected() {
    let mut g = populated_graph();
    // Org -works_at-> Person violates the domain (source must be person).
    let err = g.insert_edge(Edge::new("e1", "acme", "ada", "works_at")).unwrap_err();
    assert!(matches!(
        err,
        GraphError::DomainViolation { ref source_type, .. } if source_type == "org"
    ));
}

#[test]
fn range_violation_rejected() {
    let mut g = populated_graph();
    g.insert_node(Node::new("bob", "person", "Bob")).unwrap();
    // Person -works_at-> Person violates the range (target must be org).
    let err = g.insert_edge(Edge::new("e1", "ada", "bob", "works_at")).unwrap_err();
    assert!(matches!(err, GraphError::RangeViolation { .. }));
}

#[test]
fn subtype_satisfies_constraints() {
    // Constrain an edge to the root type; a subtype instance must satisfy it.
    let ontology = test_ontology().with_edge_type(
        "mentions",
        EdgeTypeDef::new("mentions", "Mention of any entity")
            .with_source("entity")
            .with_target("entity"),
    );
    let mut g = Graph::new(ontology).unwrap();
    g.insert_node(Node::new("ada", "person", "Ada")).unwrap();
    g.insert_node(Node::new("acme", "org", "Acme")).unwrap();
    // person and org are subtypes of entity, so both endpoints are accepted.
    g.insert_edge(Edge::new("e1", "ada", "acme", "mentions")).unwrap();
}

#[test]
fn missing_endpoint_rejected() {
    let mut g = populated_graph();
    let err = g.insert_edge(Edge::new("e1", "ghost", "acme", "works_at")).unwrap_err();
    assert!(matches!(err, GraphError::MissingEndpoint { ref node, .. } if node == "ghost"));
    let err = g.insert_edge(Edge::new("e1", "ada", "ghost", "works_at")).unwrap_err();
    assert!(matches!(err, GraphError::MissingEndpoint { ref node, .. } if node == "ghost"));
}

#[test]
fn confidence_out_of_range_rejected() {
    let mut g = populated_graph();
    for bad in [-0.1, 1.5, f64::NAN] {
        let err = g
            .insert_edge(Edge::new("e1", "ada", "acme", "works_at").with_confidence(bad))
            .unwrap_err();
        assert!(matches!(err, GraphError::InvalidConfidence { .. }));
    }
}

#[test]
fn neighbors_and_incoming() {
    let mut g = populated_graph();
    g.insert_node(Node::new("bob", "person", "Bob")).unwrap();
    g.insert_edge(Edge::new("e1", "ada", "acme", "works_at")).unwrap();
    g.insert_edge(Edge::new("e2", "bob", "acme", "works_at")).unwrap();
    g.insert_edge(Edge::new("e3", "ada", "bob", "related_to")).unwrap();

    let out = g.neighbors("ada");
    assert_eq!(out.len(), 2);
    assert!(out.iter().any(|(e, n)| e.id == "e1" && n.id == "acme"));
    assert!(out.iter().any(|(e, n)| e.id == "e3" && n.id == "bob"));

    let inc = g.incoming("acme");
    assert_eq!(inc.len(), 2);
    assert!(inc.iter().any(|(e, n)| e.id == "e1" && n.id == "ada"));
    assert!(inc.iter().any(|(e, n)| e.id == "e2" && n.id == "bob"));

    assert!(g.neighbors("acme").is_empty());
    assert!(g.incoming("ada").is_empty());
}

#[test]
fn nodes_of_type_includes_subtypes() {
    let mut g = populated_graph();
    g.insert_node(Node::new("bob", "person", "Bob")).unwrap();
    assert_eq!(g.nodes_of_type("person").len(), 2);
    assert_eq!(g.nodes_of_type("org").len(), 1);
    assert_eq!(g.nodes_of_type("entity").len(), 3);
    assert!(g.nodes_of_type("spaceship").is_empty());
}

#[test]
fn remove_node_rejects_when_edges_attached() {
    let mut g = populated_graph();
    g.insert_edge(Edge::new("e1", "ada", "acme", "works_at")).unwrap();
    let err = g.remove_node("ada").unwrap_err();
    assert_eq!(err, GraphError::NodeHasEdges("ada".into()));
    let err = g.remove_node("acme").unwrap_err();
    assert_eq!(err, GraphError::NodeHasEdges("acme".into()));

    assert!(g.remove_edge("e1").is_some());
    let node = g.remove_node("ada").unwrap();
    assert_eq!(node.name, "Ada Lovelace");
    assert!(matches!(g.remove_node("ada").unwrap_err(), GraphError::UnknownNode(_)));
}

#[test]
fn ontology_validate_catches_parent_cycle() {
    let ontology = Ontology::new("cyclic")
        .with_node_type("a", NodeTypeDef::new("A", "").with_parent("b"))
        .with_node_type("b", NodeTypeDef::new("B", "").with_parent("a"));
    let err = ontology.validate().unwrap_err();
    assert!(matches!(err, OntologyError::ParentCycle(_)));
    // Graph::new must reject the invalid ontology too.
    assert!(Graph::new(ontology).is_err());
}

#[test]
fn ontology_validate_catches_unknown_references() {
    let ontology = Ontology::new("bad-inverse").with_edge_type(
        "likes",
        EdgeTypeDef::new("likes", "").with_inverse("liked_by"),
    );
    let err = ontology.validate().unwrap_err();
    assert_eq!(
        err,
        OntologyError::UnknownInverse {
            edge_type: "likes".into(),
            inverse: "liked_by".into(),
        }
    );

    let ontology = Ontology::new("bad-parent")
        .with_node_type("person", NodeTypeDef::new("Person", "").with_parent("entity"));
    assert!(matches!(
        ontology.validate().unwrap_err(),
        OntologyError::UnknownParent { .. }
    ));
}

#[test]
fn ontology_serde_round_trip() {
    let ontology = test_ontology();
    let json = serde_json::to_string_pretty(&ontology).unwrap();
    let back: Ontology = serde_json::from_str(&json).unwrap();
    assert_eq!(ontology, back);
    // PropertyDef renames survive the trip.
    assert!(json.contains("\"source_types\""));
}
