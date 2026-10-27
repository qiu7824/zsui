//! Read-only system-font mappings shared by text engines on one UI thread.
//!
//! The weak discovery slot owns no font data. Engines own the pool, databases
//! own the mappings, and the final engine release drops the pool's metadata.

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex, Weak},
    time::SystemTime,
};

use cosmic_text::fontdb::{Database, Source};

type FontBytes = dyn AsRef<[u8]> + Send + Sync;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct FileRevision {
    path: PathBuf,
    bytes: u64,
    modified: SystemTime,
}

#[derive(Default)]
pub(super) struct FontFilePool {
    files: Mutex<HashMap<FileRevision, Weak<FontBytes>>>,
}

thread_local! {
    static ACTIVE_FILES: RefCell<Weak<FontFilePool>> = const { RefCell::new(Weak::new()) };
    static SHARE_FILES: Cell<bool> = const { Cell::new(false) };
}

/// The native application owns the policy; one-window applications do not pay
/// for a sharing index. The marker keeps restoration on the creating thread.
#[cfg_attr(not(any(test, feature = "windows-font-sharing")), allow(dead_code))]
pub(crate) struct SharingScope {
    previous: bool,
    _thread: std::marker::PhantomData<std::rc::Rc<()>>,
}

#[cfg_attr(not(any(test, feature = "windows-font-sharing")), allow(dead_code))]
impl SharingScope {
    pub(crate) fn for_windows(count: usize) -> Self {
        let previous = SHARE_FILES.with(|state| state.replace(state.get() || count > 1));
        Self {
            previous,
            _thread: std::marker::PhantomData,
        }
    }
}

impl Drop for SharingScope {
    fn drop(&mut self) {
        SHARE_FILES.with(|state| state.set(self.previous));
    }
}

fn thread_pool() -> Arc<FontFilePool> {
    ACTIVE_FILES.with(|slot| {
        let mut slot = slot.borrow_mut();
        if let Some(pool) = slot.upgrade() {
            return pool;
        }
        let pool = Arc::new(FontFilePool::default());
        *slot = Arc::downgrade(&pool);
        pool
    })
}

pub(super) fn share_files(db: &mut Database) -> Option<Arc<FontFilePool>> {
    if !SHARE_FILES.with(Cell::get) {
        let ids = db.faces().map(|face| face.id).collect::<Vec<_>>();
        for id in ids {
            // SAFETY: the existing fontdb mapping contract for installed fonts;
            // this branch preserves the single-window mapping lifecycle.
            unsafe {
                db.make_shared_face_data(id);
            }
        }
        return None;
    }
    let pool = thread_pool();
    let mut files = pool
        .files
        .lock()
        .expect("font file pool should not be poisoned");
    files.retain(|_, data| data.strong_count() > 0);

    // Map a first engine's files in place. Only replace a face when another
    // engine already owns its mapping; no temporary duplicate font database
    // or complete copy of all face names is needed.
    let ids = db.faces().map(|face| face.id).collect::<Vec<_>>();
    for id in ids {
        let Some((source, _)) = db.face_source(id) else {
            continue;
        };
        let path = match &source {
            Source::File(path) | Source::SharedFile(path, _) => Some(path.clone()),
            Source::Binary(_) => None,
        };
        if let Some(path) = path {
            let revision = std::fs::metadata(&path).ok().and_then(|metadata| {
                Some(FileRevision {
                    path: path.clone(),
                    bytes: metadata.len(),
                    modified: metadata.modified().ok()?,
                })
            });
            let cached = revision
                .as_ref()
                .and_then(|key| files.get(key))
                .and_then(Weak::upgrade);
            if let Some(data) = cached {
                let already_shared = matches!(&source, Source::SharedFile(_, current) if Arc::ptr_eq(current, &data));
                if !already_shared {
                    // This happens before layouts/aliases exist. fontdb's slot
                    // reuse preserves enumeration order, verified by the test.
                    if let Some(mut face) = db.face(id).cloned() {
                        face.source = Source::SharedFile(path, data);
                        db.remove_face(id);
                        db.push_face_info(face);
                    }
                }
            } else {
                // SAFETY: this retains the same read-only mapping contract
                // already used by fontdb/cosmic-text for installed font files.
                // Arc keeps the mapping alive until every face releases it.
                if let Some((data, _)) = unsafe { db.make_shared_face_data(id) } {
                    if let Some(revision) = revision {
                        files.insert(revision, Arc::downgrade(&data));
                    }
                }
            }
        }
    }
    drop(files);
    Some(pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmic_text::{fontdb::Family, FontSystem};

    fn font_database() -> Database {
        let (_, system) = FontSystem::new().into_locale_and_db();
        let face = system
            .faces()
            .find(|face| matches!(face.source, Source::File(_)))
            .expect("installed font file")
            .clone();
        let mut db = Database::new();
        db.set_sans_serif_family("Custom UI family");
        db.push_face_info(face);
        db
    }

    fn bytes(db: &Database) -> Arc<FontBytes> {
        match &db.faces().next().unwrap().source {
            Source::SharedFile(_, data) => Arc::clone(data),
            _ => panic!("font was not mapped"),
        }
    }

    #[test]
    fn windows_share_read_only_font_bytes_without_sharing_mutable_databases() {
        let _sharing = SharingScope::for_windows(2);
        let mut first = font_database();
        let mut second = first.clone();
        let before = first.faces().next().unwrap().post_script_name.clone();
        let first_owner = share_files(&mut first).unwrap();
        let second_owner = share_files(&mut second).unwrap();
        assert!(Arc::ptr_eq(&first_owner, &second_owner));
        assert!(Arc::ptr_eq(&bytes(&first), &bytes(&second)));
        assert_eq!(second.family_name(&Family::SansSerif), "Custom UI family");
        assert_eq!(second.faces().next().unwrap().post_script_name, before);
        let weak_owner = Arc::downgrade(&first_owner);
        drop(first);
        drop(first_owner);
        assert!(weak_owner.upgrade().is_some());
        assert!(!bytes(&second).as_ref().as_ref().is_empty());
        drop(second);
        drop(second_owner);
        assert!(weak_owner.upgrade().is_none());
    }

    #[test]
    fn font_mapping_pool_preserves_send_sync_engine_boundary() {
        fn send_sync<T: Send + Sync>() {}
        send_sync::<FontFilePool>();
        send_sync::<Arc<FontBytes>>();
    }

    #[test]
    fn sharing_preserves_installed_face_order_and_generic_families() {
        let _sharing = SharingScope::for_windows(3);
        let (_, mut first) = FontSystem::new().into_locale_and_db();
        let mut second = first.clone();
        let names = first
            .faces()
            .map(|face| (face.post_script_name.clone(), face.index))
            .collect::<Vec<_>>();
        let _first_owner = share_files(&mut first);
        let _second_owner = share_files(&mut second);
        assert_eq!(
            second
                .faces()
                .map(|face| (face.post_script_name.clone(), face.index))
                .collect::<Vec<_>>(),
            names
        );
        for family in [
            Family::Serif,
            Family::SansSerif,
            Family::Monospace,
            Family::Cursive,
            Family::Fantasy,
        ] {
            assert_eq!(first.family_name(&family), second.family_name(&family));
        }
    }

    #[test]
    fn single_window_policy_avoids_the_index_and_restores_after_nested_scopes() {
        let mut db = font_database();
        {
            let _single = SharingScope::for_windows(1);
            assert!(share_files(&mut db).is_none());
            {
                let _multiple = SharingScope::for_windows(3);
                assert!(share_files(&mut db).is_some());
            }
            assert!(share_files(&mut db).is_none());
        }
        assert!(!SHARE_FILES.with(Cell::get));
    }
}
