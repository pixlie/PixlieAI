use postcard::{from_bytes, to_allocvec};
use rocksdb::{Direction, IteratorMode, DB};

fn get_chunk_id_and_node_ids(id: &u32) -> (u32, Vec<u32>) {
    let chunk_id = id / 100;
    (chunk_id, (chunk_id * 100..(chunk_id * 100 + 100)).collect())
}

fn main() {
    println!(
        "Chunk IDs for node ID 122: {:?}",
        get_chunk_id_and_node_ids(&6)
    );

    let tempdir = tempfile::Builder::new()
        .prefix("_path_for_rocksdb_storage2")
        .tempdir()
        .expect("Failed to create temporary path for the _path_for_rocksdb_storage2.");
    let path = tempdir.path();
    {
        let db = DB::open_default(path).unwrap();
        db.put(
            to_allocvec("nodes/chunk/1").unwrap(),
            to_allocvec("random value 1234567890").unwrap(),
        )
        .unwrap();
        db.put(
            to_allocvec("edges/chunk/2").unwrap(),
            to_allocvec("random value 1234567890").unwrap(),
        )
        .unwrap();
        db.put(
            to_allocvec("nodes/chunk/2").unwrap(),
            to_allocvec("random value 1234567890").unwrap(),
        )
        .unwrap();
        db.put(
            to_allocvec("edges/chunk/3").unwrap(),
            to_allocvec("random value 1234567890").unwrap(),
        )
        .unwrap();
        db.put(
            to_allocvec("nodes/chunk/3").unwrap(),
            to_allocvec("random value 1234567890").unwrap(),
        )
        .unwrap();
        let iter = db.iterator(IteratorMode::From(
            &to_allocvec("edges/chunk/2").unwrap(),
            Direction::Forward,
        )); // Always iterates forward
        println!("0 modulus 100: {}", 0u32.rem_euclid(100));
    }
}
