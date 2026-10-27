//! Shared deterministic data and interactions for the device-console benchmark.
#![allow(dead_code)]

pub const TITLE: &str = "设备控制台 / Device Console";
pub const WIDTH: u32 = 960;
pub const HEIGHT: u32 = 680;

#[derive(Clone, Copy)]
pub struct Device {
    pub name: &'static str,
    pub kind: &'static str,
    pub online: bool,
    pub latency: u32,
}

pub const DEVICES: [Device; 6] = [
    Device {
        name: "工作站 A / Workstation A",
        kind: "Desktop",
        online: true,
        latency: 12,
    },
    Device {
        name: "文件节点 / File node",
        kind: "Storage",
        online: true,
        latency: 24,
    },
    Device {
        name: "采集器 / Sensor",
        kind: "IoT",
        online: true,
        latency: 8,
    },
    Device {
        name: "测试设备 / Test device",
        kind: "Lab",
        online: false,
        latency: 0,
    },
    Device {
        name: "边缘节点 / Edge node",
        kind: "Gateway",
        online: true,
        latency: 36,
    },
    Device {
        name: "备用节点 / Standby",
        kind: "Backup",
        online: false,
        latency: 0,
    },
];

#[derive(Clone)]
pub struct State {
    pub query: String,
    pub online_only: bool,
    pub load: f32,
    pub selected: usize,
}

impl Default for State {
    fn default() -> Self {
        Self {
            query: String::new(),
            online_only: false,
            load: 42.0,
            selected: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    Query(String),
    OnlineOnly(bool),
    Load(f32),
    Select(usize),
    Reset,
    Repaint,
}

impl State {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Query(query) => self.query = query,
            Message::OnlineOnly(value) => self.online_only = value,
            Message::Load(value) => self.load = value.clamp(0.0, 100.0),
            Message::Select(index) if index < DEVICES.len() => self.selected = index,
            Message::Reset => *self = Self::default(),
            _ => {}
        }
    }

    pub fn visible(&self) -> Vec<(usize, Device)> {
        let query = self.query.to_lowercase();
        DEVICES
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, device)| {
                (!self.online_only || device.online)
                    && (device.name.to_lowercase().contains(&query)
                        || device.kind.to_lowercase().contains(&query))
            })
            .collect()
    }

    pub fn selection(&self) -> String {
        format!("已选择 / Selected: {}", DEVICES[self.selected].name)
    }
}

pub fn status(device: Device) -> String {
    if device.online {
        format!("在线 / Online · {} ms", device.latency)
    } else {
        "离线 / Offline".into()
    }
}

pub fn has_flag(flag: &str) -> bool {
    std::env::args().any(|arg| arg == flag)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn filters_and_selection_survive_independently_and_reset_restores_fixture() {
        let mut state = State::default();
        state.update(Message::Select(4));
        state.update(Message::Query("STORAGE".into()));
        assert_eq!(
            state.visible().iter().map(|(i, _)| *i).collect::<Vec<_>>(),
            vec![1]
        );
        assert_eq!(state.selected, 4);
        state.update(Message::Query(String::new()));
        state.update(Message::OnlineOnly(true));
        assert_eq!(state.visible().len(), 4);
        state.update(Message::Load(110.0));
        assert_eq!(state.load, 100.0);
        state.update(Message::Reset);
        assert_eq!(state.visible().len(), 6);
        assert_eq!(state.selected, 0);
        assert_eq!(state.load, 42.0);
    }
}
