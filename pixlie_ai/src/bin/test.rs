use postcard::{from_bytes, to_allocvec};
use rocksdb::{Options, SliceTransform, DB};
use std::str;

const NODES_CHUNK_PREFIX: &str = "nodes/chunk/";

fn get_chunk_id_and_node_ids(id: &u32) -> (u32, Vec<u32>) {
    let chunk_id = id / 100;
    (chunk_id, (chunk_id * 100..(chunk_id * 100 + 100)).collect())
}

fn main() {
    // println!(
    //     "Chunk IDs for node ID 122: {:?}",
    //     get_chunk_id_and_node_ids(&6)
    // );

    let tempdir = tempfile::Builder::new()
        .prefix("_path_for_rocksdb_storage2")
        .tempdir()
        .expect("Failed to create temporary path for the _path_for_rocksdb_storage2.");
    let db_path = tempdir.path();
    let prefix_extractor = SliceTransform::create_fixed_prefix(NODES_CHUNK_PREFIX.len());
    let mut opts = Options::default();
    opts.create_if_missing(true);
    opts.set_prefix_extractor(prefix_extractor);
    let db = DB::open(&opts, db_path).unwrap();

    {
        db.put(
            format!("{}1", NODES_CHUNK_PREFIX),
            to_allocvec("random value 1234567890").unwrap(),
        )
        .unwrap();
        db.put(
            to_allocvec("edges/chunk/2").unwrap(),
            to_allocvec("random value 1234567890").unwrap(),
        )
        .unwrap();
        db.put(
            format!("{}2", NODES_CHUNK_PREFIX),
            to_allocvec("random value 1234567890").unwrap(),
        )
        .unwrap();
        db.put(
            to_allocvec("edges/chunk/3").unwrap(),
            to_allocvec("random value 1234567890").unwrap(),
        )
        .unwrap();
        db.put(
            format!("{}3", NODES_CHUNK_PREFIX),
            to_allocvec("random value 1234567890").unwrap(),
        )
        .unwrap();
        println!("Saved random values to DB");
    }

    {
        // let db = DB::open(&opts, db_path).unwrap();
        println!("Reading DB");
        let prefix = format!("{}", NODES_CHUNK_PREFIX);
        for item in db.prefix_iterator(&prefix) {
            let (key, value) = match item {
                Ok(item) => item,
                Err(err) => {
                    println!("Error in item: {:?}", err);
                    return;
                }
            };
            let key = str::from_utf8(&key).unwrap();
            // let key: &str = match from_bytes(&key) {
            //     Ok(v) => v,
            //     Err(e) => {
            //         println!("Error in key: {:?}", e);
            //         return;
            //     }
            // };
            let value: String = match from_bytes(&value) {
                Ok(v) => v,
                Err(e) => {
                    println!("Error in value: {:?}", e);
                    return;
                }
            };
            println!("Saw {:?} {:?}", key, value);
        }
    }
}
