use crate::{Located, Span};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DbmlDocument {
    pub span: Span,
    pub items: Vec<Located<DbmlItem>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum DbmlItem {
    Project(DbmlProject),
    Table(DbmlTable),
    TablePartial(DbmlTablePartial),
    Enum(DbmlEnum),
    Ref(DbmlRef),
    TableGroup(DbmlTableGroup),
    Note(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DbmlProject {
    pub name: String,
    pub properties: Vec<Located<DbmlProperty>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DbmlProperty {
    pub name: String,
    pub value: DbmlValue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DbmlTable {
    pub schema: Option<String>,
    pub name: String,
    pub alias: Option<String>,
    pub settings: Vec<DbmlSetting>,
    pub items: Vec<Located<DbmlTableItem>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum DbmlTableItem {
    Column(DbmlColumn),
    Indexes(Vec<Located<DbmlIndex>>),
    Check(DbmlCheck),
    Checks(Vec<Located<DbmlCheck>>),
    Note(String),
    Partial(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DbmlColumn {
    pub name: String,
    pub data_type: String,
    pub settings: Vec<DbmlSetting>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DbmlIndex {
    pub expressions: Vec<String>,
    pub settings: Vec<DbmlSetting>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DbmlCheck {
    pub expression: String,
    pub settings: Vec<DbmlSetting>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DbmlTablePartial {
    pub name: String,
    pub items: Vec<Located<DbmlTableItem>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DbmlEnum {
    pub schema: Option<String>,
    pub name: String,
    pub values: Vec<Located<DbmlEnumValue>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DbmlEnumValue {
    pub name: String,
    pub settings: Vec<DbmlSetting>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DbmlRef {
    pub name: Option<String>,
    pub from: DbmlEndpoint,
    pub cardinality: DbmlCardinality,
    pub to: DbmlEndpoint,
    pub settings: Vec<DbmlSetting>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DbmlEndpoint {
    pub schema: Option<String>,
    pub table: String,
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DbmlCardinality {
    ManyToOne,
    OneToMany,
    OneToOne,
    ManyToMany,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DbmlTableGroup {
    pub name: String,
    pub tables: Vec<String>,
    pub properties: Vec<Located<DbmlProperty>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DbmlSetting {
    pub name: String,
    pub value: Option<DbmlValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum DbmlValue {
    String(String),
    Number(String),
    Boolean(bool),
    Expression(String),
    Identifier(String),
    Raw(String),
}
