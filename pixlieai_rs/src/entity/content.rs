use chrono::{DateTime, Utc};

pub struct Title(pub String);

pub struct Heading(pub String);

pub struct Paragraph(pub String);

pub struct TableHead(pub Vec<String>);

pub enum CommonDataTypes {
    SmallInteger(i8),
    Integer(i32),
    Float(f32),
    String(String),
    Boolean(bool),
    Date(DateTime<Utc>),
}

pub struct TableRow(Vec<CommonDataTypes>);

pub struct Table(Vec<TableRow>);
