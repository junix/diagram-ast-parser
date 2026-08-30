use crate::{ast::Scalar, Located, Span};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PikchrDocument {
    pub span: Span,
    pub statements: Vec<Located<PikchrStatement>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum PikchrStatement {
    Object(PikchrObject),
    Direction(PikchrDirection),
    Assignment(PikchrAssignment),
    Define(PikchrDefine),
    Print(Vec<Scalar>),
    Assert(String),
    Place(PikchrPlace),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PikchrObject {
    pub label: Option<String>,
    pub object_type: String,
    pub attributes: Vec<Scalar>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PikchrDirection {
    Right,
    Down,
    Left,
    Up,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PikchrAssignment {
    pub variable: String,
    pub operator: String,
    pub expression: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PikchrDefine {
    pub name: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PikchrPlace {
    pub label: String,
    pub expression: String,
}
