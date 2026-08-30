use crate::Span;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaveDromDocument {
    pub span: Span,
    pub timing: Option<WaveTimingDiagram>,
    pub register: Option<WaveRegisterDiagram>,
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaveTimingDiagram {
    pub signal: Vec<WaveSignalItem>,
    pub edges: Vec<String>,
    pub head: Option<WaveHeaderFooter>,
    pub foot: Option<WaveHeaderFooter>,
    pub config: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum WaveSignalItem {
    Lane(WaveLane),
    Group(WaveGroup),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaveLane {
    pub name: Option<String>,
    pub wave: Option<String>,
    pub data: Vec<String>,
    pub node: Option<String>,
    pub phase: Option<f64>,
    pub period: Option<f64>,
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaveGroup {
    pub label: String,
    pub items: Vec<WaveSignalItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaveHeaderFooter {
    pub text: Option<String>,
    pub tick: Option<i64>,
    pub tock: Option<i64>,
    pub every: Option<i64>,
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaveRegisterDiagram {
    pub fields: Vec<WaveRegisterField>,
    pub config: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaveRegisterField {
    pub bits: Option<u64>,
    pub name: Option<String>,
    pub attr: Option<Value>,
    pub field_type: Option<Value>,
    pub extra: Map<String, Value>,
}
