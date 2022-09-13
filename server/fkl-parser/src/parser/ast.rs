use serde::Deserialize;
use serde::Serialize;

// strategy DDD

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextMap {
  pub name: String,
  pub contexts: BoundedContext,
  pub relations: Vec<BoundedContextRelation>
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BoundedContext {
  pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BoundedContextRelation {
  // Symmetric relation
  SharedKernel,
  Partnership,
  // Upstream Downstream
  CustomerSupplier,
  GenericUpstreamDownstream,
  // SeparateWay,
  Conformist,
  AntiCorruptionLayer,
  OpenHostService,
  PublishedLanguage,
}

// tactic DDD

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Aggregate {
  pub name: String,
  pub description: String,
  pub is_root: bool,
  pub context: String,
  pub entities: Vec<Entity>
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DomainEvent {
  pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct  Entity {
  pub name: String,
  pub fields: Vec<Field>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct  Field {
  pub name: String,
  pub type_: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Attribute {
  pub key: String,
  pub value: String
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Property {
  pub required: bool,
  pub nullable: bool,
  pub unique: bool,
}
