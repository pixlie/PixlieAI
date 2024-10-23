use chrono::{DateTime, Utc};

pub struct Title(pub String);

pub struct Heading(pub String);

pub struct Paragraph(pub String);

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

pub struct TableRow(pub Vec<TableCellType>);

// Headings are part of Table node
// Has part nodes to TableRow(s)
pub struct Table(pub Vec<String>);
