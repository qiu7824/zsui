#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MainTimerTask {
    StartupRecovery,
    VvWatch,
    VvShow,
    Paste,
    SearchDebounce,
    HiddenReclaim,
    ClipboardRetry,
    DpiFit,
    ScrollFade,
    EdgeAutoHide,
    OutsideHide,
    CloudSync,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MainTimerIds {
    pub startup_recovery: usize,
    pub vv_watch: usize,
    pub vv_show: usize,
    pub paste: usize,
    pub search_debounce: usize,
    pub hidden_reclaim: usize,
    pub clipboard_retry: usize,
    pub dpi_fit: usize,
    pub scroll_fade: usize,
    pub edge_auto_hide: usize,
    pub outside_hide: usize,
    pub cloud_sync: usize,
}

pub fn main_timer_task_for_id(timer_id: usize, ids: MainTimerIds) -> Option<MainTimerTask> {
    if timer_id == ids.startup_recovery {
        Some(MainTimerTask::StartupRecovery)
    } else if timer_id == ids.vv_watch {
        Some(MainTimerTask::VvWatch)
    } else if timer_id == ids.vv_show {
        Some(MainTimerTask::VvShow)
    } else if timer_id == ids.paste {
        Some(MainTimerTask::Paste)
    } else if timer_id == ids.search_debounce {
        Some(MainTimerTask::SearchDebounce)
    } else if timer_id == ids.hidden_reclaim {
        Some(MainTimerTask::HiddenReclaim)
    } else if timer_id == ids.clipboard_retry {
        Some(MainTimerTask::ClipboardRetry)
    } else if timer_id == ids.dpi_fit {
        Some(MainTimerTask::DpiFit)
    } else if timer_id == ids.scroll_fade {
        Some(MainTimerTask::ScrollFade)
    } else if timer_id == ids.edge_auto_hide {
        Some(MainTimerTask::EdgeAutoHide)
    } else if timer_id == ids.outside_hide {
        Some(MainTimerTask::OutsideHide)
    } else if timer_id == ids.cloud_sync {
        Some(MainTimerTask::CloudSync)
    } else {
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsTimerTask {
    HideScrollbar,
    ClearSaveHint,
    DpiFit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SettingsTimerIds {
    pub hide_scrollbar: usize,
    pub clear_save_hint: usize,
    pub dpi_fit: usize,
}

pub fn settings_timer_task_for_id(
    timer_id: usize,
    ids: SettingsTimerIds,
) -> Option<SettingsTimerTask> {
    if timer_id == ids.hide_scrollbar {
        Some(SettingsTimerTask::HideScrollbar)
    } else if timer_id == ids.clear_save_hint {
        Some(SettingsTimerTask::ClearSaveHint)
    } else if timer_id == ids.dpi_fit {
        Some(SettingsTimerTask::DpiFit)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_timer_ids_map_to_stable_tasks() {
        let ids = MainTimerIds {
            startup_recovery: 1,
            vv_watch: 2,
            vv_show: 3,
            paste: 4,
            search_debounce: 5,
            hidden_reclaim: 6,
            clipboard_retry: 7,
            dpi_fit: 8,
            scroll_fade: 9,
            edge_auto_hide: 10,
            outside_hide: 11,
            cloud_sync: 12,
        };

        assert_eq!(
            main_timer_task_for_id(1, ids),
            Some(MainTimerTask::StartupRecovery)
        );
        assert_eq!(main_timer_task_for_id(8, ids), Some(MainTimerTask::DpiFit));
        assert_eq!(
            main_timer_task_for_id(12, ids),
            Some(MainTimerTask::CloudSync)
        );
        assert_eq!(main_timer_task_for_id(99, ids), None);
    }

    #[test]
    fn settings_timer_ids_map_to_stable_tasks() {
        let ids = SettingsTimerIds {
            hide_scrollbar: 21,
            clear_save_hint: 22,
            dpi_fit: 23,
        };

        assert_eq!(
            settings_timer_task_for_id(21, ids),
            Some(SettingsTimerTask::HideScrollbar)
        );
        assert_eq!(
            settings_timer_task_for_id(23, ids),
            Some(SettingsTimerTask::DpiFit)
        );
        assert_eq!(settings_timer_task_for_id(99, ids), None);
    }
}
