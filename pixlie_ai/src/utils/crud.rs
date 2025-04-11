use crate::error::PiError;
use crate::{config::Settings, error::PiResult};
use log::error;
use postcard::{from_bytes, to_allocvec};
use rocksdb::DB;
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;

const PIXLIE_AI_DB: &str = "pixlie_ai";

fn get_pixlie_ai_db() -> PiResult<DB> {
    let settings = Settings::get_cli_settings()?;
    let path_to_storage_dir = match settings.path_to_storage_dir {
        Some(ref path) => path.clone(),
        None => {
            error!("Cannot find path to storage directory");
            return Err(PiError::InternalError(
                "Cannot find path to storage directory".to_string(),
            ));
        }
    };
    let mut path = PathBuf::from(path_to_storage_dir);
    path.push(format!("{}.rocksdb", PIXLIE_AI_DB));
    Ok(DB::open_default(path)?)
}

pub trait CrudItem {
    fn get_id(&self) -> String;
}

pub trait Crud {
    type Item: Clone + Serialize + DeserializeOwned + CrudItem;

    fn get_collection_name() -> &'static str;

    fn create(item: Self::Item) -> PiResult<Self::Item> {
        let mut items = Self::read_list()?;
        items.push(item.clone());
        let item_ids: Vec<String> = items.iter().map(|x| x.get_id()).collect();

        let db = get_pixlie_ai_db()?;
        match to_allocvec(&item_ids) {
            Ok(payload) => match db.put(format!("{}/ids", Self::get_collection_name()), payload) {
                Ok(_) => {}
                Err(err) => {
                    error!("Error writing item IDs: {}", err);
                    return Err(err.into());
                }
            },
            Err(err) => {
                error!("Error serializing item IDs: {}", err);
                return Err(err.into());
            }
        };
        match to_allocvec(&item) {
            Ok(payload) => match db.put(
                format!("{}/{}", Self::get_collection_name(), item.get_id()),
                payload,
            ) {
                Ok(_) => {}
                Err(err) => {
                    error!("Error writing item {}: {}", item.get_id(), err);
                    return Err(err.into());
                }
            },
            Err(err) => {
                error!("Error serializing item {}: {}", item.get_id(), err);
                return Err(err.into());
            }
        }
        db.flush()?;
        Ok(item)
    }

    fn read_list() -> PiResult<Vec<Self::Item>> {
        let db = get_pixlie_ai_db()?;
        match db.get(format!("{}/ids", Self::get_collection_name())) {
            Ok(item_ids) => match item_ids {
                Some(item_ids) => {
                    let mut items: Vec<Self::Item> = vec![];
                    let item_ids: Vec<String> = match from_bytes(&item_ids) {
                        Ok(item_ids) => item_ids,
                        Err(err) => {
                            error!("Error deserializing item IDs: {}", err);
                            return Err(err.into());
                        }
                    };
                    for item_id in item_ids {
                        match db.get(format!("{}/{}", Self::get_collection_name(), item_id)) {
                            Ok(item) => match item {
                                Some(item) => {
                                    let item = match from_bytes(&item) {
                                        Ok(item) => item,
                                        Err(err) => {
                                            error!("Error deserializing item {}: {}", item_id, err);
                                            continue;
                                        }
                                    };
                                    items.push(item);
                                }
                                None => {
                                    error!("item {} not found", item_id);
                                }
                            },
                            Err(err) => {
                                error!("Error reading item {}: {}", item_id, err);
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

    fn read_item(id: &str) -> PiResult<Self::Item> {
        let db = get_pixlie_ai_db()?;
        match db.get(format!("{}/{}", Self::get_collection_name(), id)) {
            Ok(item) => match item {
                Some(item) => match from_bytes(&item) {
                    Ok(item) => Ok(item),
                    Err(err) => {
                        error!("Error deserializing item {}: {}", id, err);
                        return Err(err.into());
                    }
                },
                None => {
                    error!("Item with ID {} not found", id);
                    Err(PiError::CrudError("Item not found".to_string()))
                }
            },
            Err(err) => Err(err.into()),
        }
    }

    fn update(uuid: &str, item: Self::Item) -> PiResult<String> {
        let db = get_pixlie_ai_db()?;
        match to_allocvec(&item) {
            Ok(payload) => {
                match db.put(format!("{}/{}", Self::get_collection_name(), uuid), payload) {
                    Ok(_) => {}
                    Err(err) => {
                        error!("Error writing item {}: {}", item.get_id(), err);
                        return Err(err.into());
                    }
                }
            }
            Err(err) => {
                error!("Error serializing item {}: {}", item.get_id(), err);
                return Err(err.into());
            }
        }
        Ok(item.get_id())
    }

    fn delete(uuid: &str) -> PiResult<String> {
        let db = get_pixlie_ai_db()?;
        match db.delete(format!("{}/{}", Self::get_collection_name(), uuid)) {
            Ok(_) => {
                let mut items = Self::read_list()?;
                items.retain(|item| item.get_id() != uuid);
                let item_ids: Vec<String> = items.iter().map(|x| x.get_id()).collect();
                match to_allocvec(&item_ids) {
                    Ok(payload) => {
                        match db.put(format!("{}/ids", Self::get_collection_name()), payload) {
                            Ok(_) => {}
                            Err(err) => {
                                error!("Error writing item IDs: {}", err);
                                return Err(err.into());
                            }
                        }
                    }
                    Err(err) => {
                        error!("Error serializing item IDs: {}", err);
                        return Err(err.into());
                    }
                }
                Ok(uuid.to_string())
            }
            Err(err) => {
                error!("Error deleting item {}: {}", uuid, err);
                return Err(err.into());
            }
        }
    }
}
