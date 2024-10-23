use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Title(pub String);

#[derive(Deserialize, Serialize)]
pub struct Heading(pub String);

#[derive(Deserialize, Serialize)]
pub struct Paragraph(pub String);

#[derive(Deserialize, Serialize)]
pub enum TableCellType {
    SmallInteger(i8),
    Integer(i32),
    Float(f32),
    String(String),
    Boolean(bool),
    Date(DateTime<Utc>),
    Time(DateTime<Utc>),
    DateTime(DateTime<Utc>),
    Email(String),
    Link(String),
    Currency(String),
    Place(String),
    Country(String),
}

#[derive(Deserialize, Serialize)]
pub struct TableRow(pub Vec<TableCellType>);

// Headings are part of Table node
// Has part nodes to TableRow(s)
#[derive(Deserialize, Serialize)]
pub struct Table(pub Vec<String>);
