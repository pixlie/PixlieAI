// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::{ToSchema};

#[derive(Clone, Default, Deserialize, Serialize, ToSchema, TS)]
pub struct Metadata {
    pub author: Option<String>,
    pub creator: Option<String>,
    pub description: Option<String>,
    pub favicon: Option<String>,
    pub image: Option<String>,
    pub language: Option<String>,
    pub locale: Option<String>,
    pub modified_time: Option<String>,
    pub published_time: Option<String>,
    pub site_name: Option<String>,
    pub tags: Option<Vec<String>>,
    pub title: Option<String>,
    pub url: Option<String>,
}
