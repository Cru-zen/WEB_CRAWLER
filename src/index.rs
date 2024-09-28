use dashmap::{DashMap, DashSet};
use tracing::info;
use serde_json::Error as SerdeError;

pub struct Index {
    pub inner: DashMap<String, DashSet<String>>,
    file: Option<String>,
}

impl Index {
    pub fn new(file: Option<String>) -> Self {
        Self {
            inner: DashMap::new(),
            file,
        }
    }

    // Optional: you could provide a method to add items to the index
    pub fn add_entry(&self, key: String, value: String) {
        self.inner
            .entry(key)
            .or_insert_with(DashSet::new)
            .insert(value);
    }

    // Optional: Serialize and write the index to a file
    // pub fn write_to_file(&self, path: &str) -> Result<(), SerdeError> {
    //     let mut file = std::fs::File::create(path).map_err(|e| {
    //         warn!("Failed to create file: {}", e);
    //         e
    //     })?;
    //     serde_json::to_writer(&mut file, &self.inner).map_err(|e| {
    //         warn!("Failed to serialize index: {}", e);
    //         e
    //     })
    // }
}

impl Drop for Index {
    fn drop(&mut self) {
        match self.file {
            Some(ref path) => {
                let mut file = std::fs::File::create(path).unwrap();
                match serde_json::to_writer(&mut file, &self.inner) {
                    Ok(_) => info!("Index written to {}", path),
                    Err(e) => info!("Failed to write index to {}: {}", path, e),
                }
            }
            None => {
                for e in self.inner.iter() {
                    info!("{} contained {} link(s)", e.key(), e.value().len());
                    for e in e.value().iter() {
                        info!("\t-> {}", e.key());
                    }
                }
            }
        }
    }
}
