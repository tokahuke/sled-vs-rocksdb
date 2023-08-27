use std::time::Instant;

#[cfg(feature = "jemallocator")]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

/// Path of Sled DB in disk.
pub const SLED_PATH: &str = "./sled";

/// Path to RocksDB in disk.
pub const ROCKS_PATH: &str = "./rocks";

/// This is the concatenation merge operator in Sled.
#[cfg(feature = "sled")]
fn sled_cat(_key: &[u8], val: Option<&[u8]>, new: &[u8]) -> Option<Vec<u8>> {
    Some(val.into_iter().flatten().chain(new).cloned().collect())
}

/// This is the concatenation merge operator in RocksDB.
#[cfg(feature = "rocksdb")]
fn rocks_cat(_key: &[u8], val: Option<&[u8]>, new: &mut rocksdb::MergeOperands) -> Option<Vec<u8>> {
    Some(
        val.into_iter()
            .flatten()
            .chain(new.into_iter().flatten())
            .cloned()
            .collect(),
    )
}

/// Quick and dirty slice to u32.
fn from_bytes(b: &[u8]) -> u32 {
    u32::from_le_bytes([b[0], b[1], b[2], b[3]])
}

#[cfg(feature = "sled")]
fn sled_main() {
    // This is how we initialize Sled.
    let sled_db = {
        let config = sled::Config::default()
            .path(SLED_PATH)
            .use_compression(true)
            .cache_capacity(1_000_000_000)
            .flush_every_ms(Some(200))
            .print_profile_on_drop(true);

        let db = config.open().unwrap();
        db.set_merge_operator(sled_cat);

        db
    };

    // 1. Fill each DB with consecutive integers, all holding ntegers from 0 to 9 concatenated.

    // This is how we do it in Sled.
    let tic = Instant::now();

    for i in 0..1_000_000u32 {
        for j in 0..10u32 {
            sled_db.merge(&i.to_le_bytes(), &j.to_le_bytes()).unwrap();
        }
    }

    println!("Sled: {:?}", tic.elapsed());

    // 2. Now, sum all integers contained in all keys.

    // This is how we do it in Sled.
    let tic = Instant::now();
    let count = sled_db
        .iter()
        .map(Result::unwrap)
        .map(|(_, val)| val.as_ref().windows(4).map(from_bytes).collect::<Vec<_>>())
        .flatten()
        .map(|i| i as u64)
        .sum::<u64>();
    dbg!(count);

    println!("Sled: {:?}", tic.elapsed());
}

#[cfg(feature = "rocksdb")]
fn rocksdb_main() {
    // This is how we initialize RocksDB.
    let rocks_db = {
        let mut options = rocksdb::Options::default();
        options.create_if_missing(true);
        options.set_merge_operator("rocks_cat", rocks_cat, None);
        options.set_compression_type(rocksdb::DBCompressionType::Lz4);

        rocksdb::DB::open(&options, ROCKS_PATH).unwrap()
    };

    // 1. Fill each DB with consecutive integers, all holding ntegers from 0 to 9 concatenated.

    // This is how we do it in RocksDB.
    let tic = Instant::now();
    for i in 0..1_000_000u32 {
        for j in 0..10u32 {
            rocks_db.merge(&i.to_le_bytes(), &j.to_le_bytes()).unwrap();
        }
    }

    println!("RocksDB: {:?}", tic.elapsed());

    // 2. Now, sum all integers contained in all keys.

    // This is how we do it in RocksDB.
    let tic = Instant::now();
    let count = rocks_db
        .iterator(rocksdb::IteratorMode::Start)
        .map(|(_, val)| val.as_ref().windows(4).map(from_bytes).collect::<Vec<_>>())
        .flatten()
        .map(|i| i as u64)
        .sum::<u64>();
    dbg!(count);

    println!("RocksDB: {:?}", tic.elapsed());
}

fn main() {
    #[cfg(feature = "sled")]
    {
        sled_main();
    }

    #[cfg(feature = "rocksdb")]
    {
        rocksdb_main();
    }
}
