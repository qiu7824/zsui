#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[path = "../../../../examples/support/device_demo.rs"]
mod model;
use model::{Message, State};
use slint::{ComponentHandle, ModelRc, SharedString, Timer, TimerMode, VecModel};
use std::{cell::RefCell, rc::Rc};

slint::slint! {
    import { Button, CheckBox, LineEdit, Slider, ProgressIndicator } from "std-widgets.slint";
    export struct DeviceRow { id: int, name: string, kind: string, status: string }
    export component DeviceWindow inherits Window {
        title: "设备控制台 / Device Console · Slint";
        preferred-width: 960px;
        preferred-height: 680px;
        background: #f7f8fa;
        in property<bool> empty;
        in property<[DeviceRow]> devices;
        in-out property<string> query;
        in-out property<bool> online-only;
        in-out property<float> load: 42;
        in property<string> selection;
        callback query-changed(string);
        callback online-changed(bool);
        callback load-changed(float);
        callback selected(int);
        callback reset();
        if !root.empty: VerticalLayout {
            padding: 24px;
            spacing: 12px;
            alignment: start;
            Text { text: "设备控制台 / Device Console"; font-size: 26px; font-weight: 700; height: 36px; }
            Text { text: "6 台设备 / 6 devices     在线 4 / Online 4     平均延迟 / Average latency 20 ms"; height: 24px; }
            HorizontalLayout {
                height: 36px;
                spacing: 14px;
                LineEdit { text <=> root.query; edited => { root.query-changed(self.text); } }
                CheckBox { text: "仅在线 / Online only"; checked <=> root.online-only; toggled => { root.online-changed(self.checked); } }
                Button { text: "重置 / Reset"; width: 120px; clicked => { root.reset(); } }
            }
            Text { text: "设备 / Device                                      类型 / Type             状态 / Status"; height: 24px; }
            for device in root.devices: HorizontalLayout {
                height: 42px;
                spacing: 12px;
                Text { text: device.name; vertical-alignment: center; horizontal-stretch: 1; }
                Text { text: device.kind; width: 90px; vertical-alignment: center; }
                Text { text: device.status; width: 180px; vertical-alignment: center; }
                Button { text: "选择 / Select"; width: 120px; clicked => { root.selected(device.id); } }
            }
            Text { text: "模拟负载 / Simulated load: " + round(root.load) + "%"; height: 24px; }
            Slider { value <=> root.load; minimum: 0; maximum: 100; height: 30px; changed => { root.load-changed(self.value); } }
            ProgressIndicator { progress: root.load / 100; height: 12px; }
            Text { text: root.selection; height: 24px; }
        }
    }
}

fn refresh(ui: &DeviceWindow, state: &State) {
    ui.set_query(state.query.clone().into());
    ui.set_online_only(state.online_only);
    ui.set_load(state.load);
    ui.set_selection(state.selection().into());
    ui.set_devices(ModelRc::new(VecModel::from(
        state
            .visible()
            .into_iter()
            .map(|(id, device)| DeviceRow {
                id: id as i32,
                name: SharedString::from(device.name),
                kind: SharedString::from(device.kind),
                status: model::status(device).into(),
            })
            .collect::<Vec<_>>(),
    )));
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = DeviceWindow::new()?;
    let state = Rc::new(RefCell::new(State::default()));
    ui.set_empty(model::has_flag("--benchmark-empty"));
    refresh(&ui, &state.borrow());
    macro_rules! handler {
        ($method:ident, $message:expr) => {{
            let weak = ui.as_weak();
            let state = state.clone();
            ui.$method(move |value| {
                let mut state = state.borrow_mut();
                state.update($message(value));
                if let Some(ui) = weak.upgrade() {
                    refresh(&ui, &state);
                }
            });
        }};
    }
    handler!(on_query_changed, |value: SharedString| Message::Query(
        value.to_string()
    ));
    handler!(on_online_changed, Message::OnlineOnly);
    handler!(on_load_changed, Message::Load);
    handler!(on_selected, |value: i32| Message::Select(value as usize));
    ui.on_reset({
        let weak = ui.as_weak();
        let state = state.clone();
        move || {
            state.borrow_mut().update(Message::Reset);
            if let Some(ui) = weak.upgrade() {
                refresh(&ui, &state.borrow());
            }
        }
    });
    let repaint = Timer::default();
    if model::has_flag("--benchmark-repaint") {
        let weak = ui.as_weak();
        repaint.start(
            TimerMode::Repeated,
            std::time::Duration::from_millis(16),
            move || {
                if let Some(ui) = weak.upgrade() {
                    ui.window().request_redraw();
                }
            },
        );
    }
    ui.run()
}
