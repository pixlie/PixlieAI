// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct Title(pub String);

#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct Heading(pub String);

#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct Paragraph(pub String);

#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct BulletPoints(pub Vec<String>);

#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct OrderedPoints(pub Vec<String>);

#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export)]
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

#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct TableRow(pub Vec<TableCellType>);

// Headings are part of Table node
// Has part nodes to TableRow(s)
#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct Table(pub Vec<String>);
