mod disk;
mod lru_cache;
mod memory;

pub use disk::SimpleDiskCache;
pub use lru_cache::Cache as LruCache;
pub use memory::InMemoryCache;
