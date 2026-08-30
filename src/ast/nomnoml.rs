use crate::{Located, Span};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NomnomlDocument {
    pub span: Span,
    pub directives: Vec<Located<NomnomlDirective>>,
    pub statements: Vec<Located<NomnomlStatement>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NomnomlDirective {
    pub name: String,
    pub value: String,
    pub custom_style: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum NomnomlStatement {
    Classifier(NomnomlClassifier),
    Relation(NomnomlRelation),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NomnomlClassifier {
    pub classifier_type: Option<String>,
    pub attributes: BTreeMap<String, String>,
    pub compartments: Vec<NomnomlCompartment>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NomnomlCompartment {
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NomnomlRelation {
    pub start: NomnomlClassifier,
    pub end: NomnomlClassifier,
    pub association: String,
    pub start_label: Option<String>,
    pub end_label: Option<String>,
    pub raw_middle: String,
}
