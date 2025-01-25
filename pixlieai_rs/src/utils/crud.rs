use crate::{config::Settings, error::PiResult};
use log::error;
use postcard::{from_bytes, to_allocvec};
use rocksdb::DB;
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;

const PIXLIE_AI_DB: &str = "pixlie_ai";

pub trait CrudItem {
    fn get_id(&self) -> String;
}

pub trait Crud {
    type Item: Serialize + DeserializeOwned + CrudItem;

    fn get_collection_name() -> String;

    fn create(item: Self::Item) -> PiResult<Vec<Self::Item>> {
        let mut items = Self::read_list()?;
        items.push(item);
        let mut path = PathBuf::from(&Settings::get_cli_settings()?.path_to_storage_dir.unwrap());
        path.push(format!("{}.rocksdb", PIXLIE_AI_DB));
        let db = DB::open_default(path).unwrap();
        let project_ids: Vec<String> = items.iter().map(|x| x.get_id()).collect();
        match to_allocvec(&project_ids) {
            Ok(payload) => match db.put(format!("{}ids", Self::get_collection_name()), payload) {
                Ok(_) => {}
                Err(err) => {
                    error!("Error writing project IDs: {}", err);
                    return Err(err.into());
                }
            },
            Err(err) => {
                error!("Error serializing project IDs: {}", err);
                return Err(err.into());
            }
        }
        for item in &items {
            match to_allocvec(&item) {
                Ok(payload) => match db.put(
                    format!("{}{}", Self::get_collection_name(), item.get_id()),
                    payload,
                ) {
                    Ok(_) => {}
                    Err(err) => {
                        error!("Error writing project {}: {}", item.get_id(), err);
                        return Err(err.into());
                    }
                },
                Err(err) => {
                    error!("Error serializing project {}: {}", item.get_id(), err);
                    return Err(err.into());
                }
            }
        }
        Ok(items)
    }

    fn read_list() -> PiResult<Vec<Self::Item>> {
        let settings = Settings::get_cli_settings()?;
        let mut path = PathBuf::from(&settings.path_to_storage_dir.unwrap());
        path.push(format!("{}.rocksdb", PIXLIE_AI_DB));
        let db = DB::open_default(path).unwrap();
        match db.get(format!("{}ids", Self::get_collection_name())) {
            Ok(project_ids) => match project_ids {
                Some(project_ids) => {
                    let mut items: Vec<Self::Item> = vec![];
                    let item_ids: Vec<String> = match from_bytes(&project_ids) {
                        Ok(item_ids) => item_ids,
                        Err(err) => {
                            error!("Error deserializing project IDs: {}", err);
                            return Err(err.into());
                        }
                    };
                    for item_id in item_ids {
                        match db.get(format!("{}{}", Self::get_collection_name(), item_id)) {
                            Ok(item) => match item {
                                Some(item) => {
                                    let item = match from_bytes(&item) {
                                        Ok(item) => item,
                                        Err(err) => {
                                            error!(
                                                "Error deserializing project {}: {}",
                                                item_id, err
                                            );
                                            continue;
                                        }
                                    };
                                    items.push(item);
                                }
                                None => {
                                    error!("Project {} not found", item_id);
                                }
                            },
                            Err(err) => {
                                error!("Error reading project {}: {}", item_id, err);
                            }
                        }
                    }
                    Ok(items)
                }
                None => Ok(vec![]),
            },
            Err(err) => Err(err.into()),
        }
    }

    // fn read(&self, id: &str) -> Result<T, String>;
    // fn update(&self, id: &str, item: T) -> Result<(), String>;
    // fn delete(&self, id: &str) -> Result<(), String>;
}
