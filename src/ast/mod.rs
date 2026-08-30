use crate::Span;
use serde::{Deserialize, Serialize};

pub mod d2;
pub mod dbml;
pub mod likec4;
pub mod nomnoml;
pub mod pikchr;
pub mod structurizr;
pub mod wavedrom;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "format", content = "ast", rename_all = "snake_case")]
pub enum Document {
    Dbml(dbml::DbmlDocument),
    WaveDrom(wavedrom::WaveDromDocument),
    D2(d2::D2Document),
    Structurizr(structurizr::StructurizrDocument),
    LikeC4(likec4::LikeC4Document),
    Nomnoml(nomnoml::NomnomlDocument),
    Pikchr(pikchr::PikchrDocument),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScalarKind {
    Word,
    String,
    Symbol,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scalar {
    pub span: Span,
    pub kind: ScalarKind,
    pub value: String,
}
