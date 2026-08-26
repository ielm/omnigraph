# omnigraph

A typed property-graph substrate for agent knowledge representation.

`omnigraph` is the knowledge-representation layer of the Omicron project: an
agent memory graph for building arbitrary semantic relationships across typed
nodes, constrained by an explicit ontology.

**Status: pre-release v0.0.1 — the API will change.**

## Example

```rust
use omnigraph::{Edge, EdgeTypeDef, Graph, GraphError, Node, NodeTypeDef, Ontology};

fn main() -> Result<(), GraphError> {
    let ontology = Ontology::new("demo")
        .with_node_type("person", NodeTypeDef::new("Person", "A human"))
        .with_node_type("org", NodeTypeDef::new("Org", "An organization"))
        .with_edge_type(
            "works_at",
            EdgeTypeDef::new("works at", "Employment")
                .with_source("person")
                .with_target("org"),
        );

    let mut graph = Graph::new(ontology)?;
    graph.insert_node(Node::new("ada", "person", "Ada Lovelace"))?;
    graph.insert_node(Node::new("acme", "org", "Acme"))?;
    graph.insert_edge(Edge::new("e1", "ada", "acme", "works_at").with_confidence(0.9))?;

    for (edge, node) in graph.neighbors("ada") {
        println!("ada -[{}]-> {}", edge.edge_type, node.name);
    }
    Ok(())
}
```

## Design

- **Ontology-constrained.** An `Ontology` declares node types and edge types
  (each with single-inheritance subtype hierarchies), and every insertion is
  validated against it: unknown types are rejected, and edge domain/range
  constraints are enforced honoring subtyping.
- **Typed edges as assertions.** An `Edge` carries semantic `properties`
  (data about the relationship), `provenance` (where the assertion came
  from), a `confidence` score in `0.0..=1.0`, and `annotations` — metadata
  about the assertion itself, following the AnnotatedAxiom pattern.
- **Plain data.** Everything serializes with serde; no async, no heavy
  dependencies.

## Roadmap

- Persistence backends
- Embedding-aware nodes and similarity queries
- A query layer over the substrate

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at
your option.
