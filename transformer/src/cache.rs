use crate::transforms::SignalValue;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// The latest transformed value of one signal.
#[derive(Debug, Clone, Serialize)]
pub struct CachedSignal {
    pub path: String,
    pub value: SignalValue,
    pub transform: String,
    pub updated_at: String,
}

/// Shared latest-value store: the subscriber writes, the API reads.
pub type SignalCache = Arc<RwLock<HashMap<String, CachedSignal>>>;

pub fn new_cache() -> SignalCache {
    Arc::new(RwLock::new(HashMap::new()))
}
