use crate::{Located, Span};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct D2Document {
    pub span: Span,
    pub statements: Vec<Located<D2Statement>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum D2Statement {
    Entry(D2Entry),
    EdgeChain(D2EdgeChain),
    Import(D2Import),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct D2Entry {
    pub key: String,
    pub value: Option<D2Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum D2Value {
    Scalar(String),
    Map {
        label: Option<String>,
        statements: Vec<Located<D2Statement>>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct D2EdgeChain {
    pub endpoints: Vec<String>,
    pub operators: Vec<D2EdgeOperator>,
    pub label: Option<String>,
    pub attributes: Vec<Located<D2Statement>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum D2EdgeOperator {
    Directed,
    ReverseDirected,
    Undirected,
    Bidirectional,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct D2Import {
    pub path: String,
    pub spread: bool,
}
