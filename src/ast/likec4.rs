use crate::{ast::Scalar, Located, Span};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LikeC4Document {
    pub span: Span,
    pub statements: Vec<Located<LikeC4Statement>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum LikeC4Statement {
    Section(LikeC4Section),
    KindDefinition(LikeC4KindDefinition),
    Element(LikeC4Element),
    Relationship(LikeC4Relationship),
    View(LikeC4View),
    Extend(LikeC4Extend),
    Tag(LikeC4Tag),
    Property(LikeC4Property),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LikeC4SectionKind {
    Specification,
    Model,
    Views,
    Global,
    Deployment,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LikeC4Section {
    pub section: LikeC4SectionKind,
    pub body: Vec<Located<LikeC4Statement>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LikeC4KindDefinition {
    pub category: String,
    pub name: String,
    pub body: Vec<Located<LikeC4Statement>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LikeC4Element {
    pub name: String,
    pub element_type: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub reference: Option<String>,
    pub body: Vec<Located<LikeC4Statement>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LikeC4Relationship {
    pub source: Option<String>,
    pub target: String,
    pub relationship_type: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub body: Vec<Located<LikeC4Statement>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LikeC4View {
    pub view_type: String,
    pub name: Option<String>,
    pub scope: Option<String>,
    pub body: Vec<Located<LikeC4Statement>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LikeC4Extend {
    pub target: String,
    pub body: Vec<Located<LikeC4Statement>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LikeC4Tag {
    pub names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LikeC4Property {
    pub name: String,
    pub values: Vec<Scalar>,
    pub body: Vec<Located<LikeC4Statement>>,
}
