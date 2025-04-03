// Copyright 2025 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use serde::{Deserialize, Serialize};
use ts_rs::TS;

// The project settings node contains high level settings that guide the flow of a project
#[derive(Clone, Default, Deserialize, Serialize, TS)]
pub struct ProjectSettings {
    pub extract_data_only_from_specified_links: bool,
    pub crawl_direct_links_from_specified_links: bool,
    pub crawl_within_domains_of_specified_links: bool,
}
