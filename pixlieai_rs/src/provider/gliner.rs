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
use std::path::PathBuf;

#[derive(FromPyObject)]
pub struct GlinerEntity {
    pub start: u32,
    pub end: u32,
    pub text: String,
    pub label: String,
    pub score: f32,
}

pub async fn extract_entities<T>(payload: &T, path_to_gliner_home: &str) -> Vec<ExtractedEntity>
where
    T: EntityExtraction,
{
    // We use PyO3 to call the Python code that uses GLiNER to extract entities
    let mut path_to_site_packages = PathBuf::from(path_to_gliner_home);
    path_to_site_packages.push(".venv/lib/python3.12/site-packages");
    pyo3::prepare_freethreaded_python();
    let mut extracted = Python::with_gil(|py| -> PiResult<Vec<String>> {
        let os = PyModule::import_bound(py, "os")?;
        let chdir = os.getattr("chdir")?;
        chdir.call1((path_to_gliner_home,));
        let sys = PyModule::import_bound(py, "sys")?;
        let path = sys.getattr("path")?;
        path.call_method1("append", (path_to_site_packages,))?;
        let extractor = PyModule::from_code_bound(
            py,
            r#"
def extract_entities(text, labels):
    from gliner import GLiNER

    # Initialize GLiNER with the base model
    model = GLiNER.from_pretrained("urchade/gliner_mediumv2.1")

    # Perform entity prediction
    entities = model.predict_entities(text, labels, threshold=0.5)

    print(text, labels, entities)
    return entities

"#,
            "extractor.py",
            "extractor",
        )?;
        let list_entity_text: Vec<GlinerEntity> = extractor
            .call_method1(
                "extract_entities",
                (
                    payload.get_payload(),
                    payload
                        .get_labels()
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>(),
                ),
            )?
            .extract()?;
        // assert!(list_entity_text.len() > 0);
        // for x in list_entity_text {
        //     info!("Label: {}, Text: {}", x.label, x.text);
        // }
        Ok(vec![])
    });
    match extracted {
        Ok(extracted) => extract_entites_from_lines(&extracted.join("\n")),
        Err(err) => {
            error!("Error calling GLiNER: {}", err);
            vec![]
        }
    }
}
