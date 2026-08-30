use crate::{ast::Scalar, Located, Span};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructurizrDocument {
    pub span: Span,
    pub statements: Vec<Located<StructurizrStatement>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum StructurizrStatement {
    Workspace(StructurizrWorkspace),
    Element(StructurizrElement),
    Relationship(StructurizrRelationship),
    Directive(StructurizrDirective),
    Block(StructurizrBlock),
    Property(StructurizrProperty),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructurizrWorkspace {
    pub name: Option<String>,
    pub description: Option<String>,
    pub extends: Option<String>,
    pub body: Vec<Located<StructurizrStatement>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructurizrElement {
    pub id: Option<String>,
    pub element_type: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub technology: Option<String>,
    pub body: Vec<Located<StructurizrStatement>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructurizrRelationship {
    pub id: Option<String>,
    pub source: String,
    pub target: String,
    pub description: Option<String>,
    pub technology: Option<String>,
    pub tags: Option<String>,
    pub body: Vec<Located<StructurizrStatement>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructurizrDirective {
    pub name: String,
    pub arguments: Vec<Scalar>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructurizrBlock {
    pub keyword: String,
    pub arguments: Vec<Scalar>,
    pub body: Vec<Located<StructurizrStatement>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructurizrProperty {
    pub name: String,
    pub values: Vec<Scalar>,
    pub body: Vec<Located<StructurizrStatement>>,
}
