// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the Business Source License 1.1 (the "License");
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.pixlie.com/ai/license

use super::{extract_entites_from_lines, EntityExtraction};
use crate::entity::ExtractedEntity;
use crate::error::PiResult;
use log::{error, info};
use pyo3::types::{PyAnyMethods, PyList, PyModule};
use pyo3::{FromPyObject, Python};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(FromPyObject)]
pub struct GlinerEntity {
    pub start: u32,
    pub end: u32,
    pub text: String,
    pub label: String,
    pub score: f32,
}

const PYTHON_CODE: &str = r#"
from gliner import GLiNER
from dataclasses import dataclass

@dataclass
class Extracted:
    start: int
    end: int
    text: str
    label: str
    score: float

def extract_entities(text, labels):
    # Initialize GLiNER with the base model
    model = GLiNER.from_pretrained("EmergentMethods/gliner_medium_news-v2.1")

    # Perform entity prediction
    entities = model.predict_entities(text, labels)

    print(entities)
    return [Extracted(**x) for x in entities]

"#;

pub async fn extract_entities<T>(
    payload: &T,
    path_to_gliner_home: &str,
) -> PiResult<Vec<ExtractedEntity>>
where
    T: EntityExtraction,
{
    // We use PyO3 to call the Python code that uses GLiNER to extract entities
    let mut path_to_site_packages = PathBuf::from(path_to_gliner_home);
    path_to_site_packages.push(".venv/lib/python3.12/site-packages");
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| -> PiResult<Vec<ExtractedEntity>> {
        let os = PyModule::import_bound(py, "os")?;
        let chdir = os.getattr("chdir")?;
        chdir.call1((path_to_gliner_home,));
        let sys = PyModule::import_bound(py, "sys")?;
        let path = sys.getattr("path")?;
        path.call_method1("append", (path_to_site_packages,))?;
        let extractor = PyModule::from_code_bound(py, PYTHON_CODE, "extractor.py", "extractor")?;
        let gliner_entities: Vec<GlinerEntity> = extractor
            .call_method1(
                "extract_entities",
                (payload.get_payload(), payload.get_labels_to_extract()),
            )?
            .extract()?;

        Ok(gliner_entities
            .iter()
            .map(|x| ExtractedEntity {
                label: x.label.clone(),
                matching_text: x.text.clone(),
            })
            .collect())
    })
}
