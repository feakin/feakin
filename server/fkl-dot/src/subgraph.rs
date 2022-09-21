use std::fmt;

use crate::edge::Edge;
use crate::node::Node;

pub struct Subgraph {
  name: String,
  label: String,
  depth: usize,
  nodes: Vec<Node>,
  edges: Vec<Edge>,
  subgraph: Vec<Subgraph>,
}

impl Subgraph {
  pub fn new(name: &str, label: &str) -> Self {
    let mut new_name = name.to_string();
    if !name.starts_with("cluster_") {
      new_name = format!("cluster_{}", name);
    }

    Subgraph {
      name: new_name.to_string(),
      label: label.to_string(),
      depth: 0,
      nodes: Vec::new(),
      edges: Vec::new(),
      subgraph: Vec::new(),
    }
  }

  pub(crate) fn add_subgraph(&mut self, subgraph: Subgraph) {
    self.subgraph.push(subgraph);
  }

  pub(crate) fn add_node(&mut self, node: Node) {
    self.nodes.push(node);
  }
}

impl fmt::Display for Subgraph {
  fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
    out.write_str(&format!("subgraph {} {{", self.name))?;

    out.write_str(&format!("label=\"{}\";", self.label))?;

    for node in &self.nodes {
      out.write_str(&format!("{}", node))?
    }

    for edge in &self.edges {
      out.write_str(&format!("{}", edge))?
    }

    for subgraph in &self.subgraph {
      out.write_str(&format!("{}", subgraph))?
    }

    out.write_str("}")
  }
}
