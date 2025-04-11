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

    fn _update_index(db: &DB, item_ids: &Vec<String>) -> PiResult<()> {
        match to_allocvec(item_ids) {
            Ok(payload) => match db.put(format!("{}/ids", Self::get_collection_name()), payload) {
                Ok(_) => {
                    db.flush()?;
                }
                Err(err) => {
                    error!("Error writing item IDs: {}", err);
                    return Err(err.into());
                }
            },
            Err(err) => {
                error!("Error serializing item IDs: {}", err);
                return Err(err.into());
            }
        }
        Ok(())
    }

    fn create(item: Self::Item) -> PiResult<Self::Item> {
        let mut items = Self::read_list()?;
        let db = get_pixlie_ai_db()?;
        let item_id = item.get_id();
        match to_allocvec(&item) {
            Ok(payload) => match db.put(
                format!("{}/{}", Self::get_collection_name(), item_id),
                payload,
            ) {
                Ok(_) => {
                    db.flush()?;
                    items.push(item.clone());
                    Self::_update_index(&db, &items.iter().map(|x| x.get_id()).collect())?;
                }
                Err(err) => {
                    error!("Error writing item {}: {}", item_id, err);
                    return Err(err.into());
                }
            },
            Err(err) => {
                error!("Error serializing item {}: {}", item_id, err);
                return Err(err.into());
            }
        }
        db.flush()?;
        Ok(item)
    }

    fn _read_index(db: &DB) -> PiResult<Vec<String>> {
        match db.get(format!("{}/ids", Self::get_collection_name())) {
            Ok(item_ids) => match item_ids {
                Some(item_ids) => match from_bytes(&item_ids) {
                    Ok(item_ids) => Ok(item_ids),
                    Err(err) => {
                        error!("Error deserializing item IDs: {}", err);
                        Err(err.into())
                    }
                },
                None => Ok(vec![]),
            },
            Err(err) => Err(err.into()),
        }
    }

    fn read_list() -> PiResult<Vec<Self::Item>> {
        let db = get_pixlie_ai_db()?;
        let item_ids = Self::_read_index(&db)?;
        return Ok(item_ids
            .iter()
            .filter_map(|item_id| {
                match db.get(format!("{}/{}", Self::get_collection_name(), item_id)) {
                    Ok(item) => match item {
                        Some(item) => match from_bytes(&item) {
                            Ok(item) => Some(item),
                            Err(err) => {
                                error!("Error deserializing item {}: {}", item_id, err);
                                None
                            }
                        },
                        None => {
                            error!("Item {} not found", item_id);
                            None
                        }
                    },
                    Err(err) => {
                        error!("Error reading item {}: {}", item_id, err);
                        None
                    }
                }
            })
            .collect::<Vec<Self::Item>>());
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
                    Err(PiError::CrudNotFoundError(id.to_string()).into())
                }
            },
            Err(err) => Err(err.into()),
        }
    }

    fn update(uuid: &str, item: Self::Item) -> PiResult<String> {
        let db = get_pixlie_ai_db()?;
        let item_ids = Self::_read_index(&db)?;
        if !item_ids.contains(&uuid.to_string()) {
            error!("Item with ID {} not found", uuid);
            return Err(PiError::CrudNotFoundError(uuid.to_string()).into());
        }
        match to_allocvec(&item) {
            Ok(payload) => {
                match db.put(format!("{}/{}", Self::get_collection_name(), uuid), payload) {
                    Ok(_) => {
                        db.flush()?;
                        Ok(item.get_id())
                    }
                    Err(err) => {
                        error!("Error writing item {}: {}", item.get_id(), err);
                        Err(err.into())
                    }
                }
            }
            Err(err) => {
                error!("Error serializing item {}: {}", item.get_id(), err);
                Err(err.into())
            }
        }
    }

    fn delete(uuid: &str) -> PiResult<String> {
        let db = get_pixlie_ai_db()?;
        let mut item_ids = Self::_read_index(&db)?;
        if !item_ids.contains(&uuid.to_string()) {
            error!("Item with ID {} not found", uuid);
            return Err(PiError::CrudNotFoundError(uuid.to_string()).into());
        }
        item_ids.retain(|id| id != uuid);
        Self::_update_index(&db, &item_ids)?;
        match db.delete(format!("{}/{}", Self::get_collection_name(), uuid)) {
            Ok(_) => {
                let _ = db.flush();
            }
            Err(err) => {
                error!("Error deleting item {}: {}", uuid, err);
            }
        }
        Ok(uuid.to_string())
    }
}
