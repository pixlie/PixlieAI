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
pub struct LossyLocation {
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub address_line_1: String,
    pub address_line_2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: String,
}

#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub enum TypedData {
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
    Place(LossyLocation),
}

#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub enum CellData {
    TypedData(TypedData),
    NamedEntity(String, String),
}

#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct TableRow(pub Vec<CellData>);

// Headings are part of Table node
// Has part nodes to TableRow(s)
#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct Table(pub Vec<String>);
