pub mod data;
pub mod effects;
pub mod model;
pub mod ui;

use std::sync::{Arc, Mutex, MutexGuard};

pub type SharedModel = Arc<Mutex<model::AppModel>>;

pub fn lock_model(model: &SharedModel) -> MutexGuard<'_, model::AppModel> {
    model
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
