#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#[path = "../../../../examples/support/device_demo.rs"]
mod model;
use model::{Message, State};
use slint::{ComponentHandle, ModelRc, SharedString, Timer, TimerMode, VecModel};
use std::{cell::RefCell, rc::Rc};

slint::slint! {
 import {Button,CheckBox,LineEdit,Slider,ProgressIndicator} from "std-widgets.slint";
 export struct DeviceRow{id:int,name:string,kind:string,online:bool}
 component Caption inherits Text{color:#616161;font-size:12px;height:16px;}
 component StatCard inherits Rectangle{
  in property<string> heading;in property<string> value;in property<string> unit;in property<bool> accent;
  background:white;border-radius:12px;height:96px;
  Caption{x:16px;y:14px;width:parent.width - 32px;text:root.heading;}
  HorizontalLayout{x:16px;y:37px;width:parent.width - 32px;height:40px;spacing:6px;alignment:start;
   Text{text:root.value;font-size:28px;font-weight:600;color:root.accent?#0067c0:#1b1b1b;vertical-alignment:center;}
   Caption{text:root.unit;vertical-alignment:center;}
  }
 }
 export component DeviceWindow inherits Window{
  title:"设备控制台 · Slint";preferred-width:960px;preferred-height:680px;background:#f3f3f3;
  in property<bool> empty;in property<[DeviceRow]> devices;in-out property<string> query;in-out property<bool> online-only;in-out property<float> load:42;
  in property<int> selected-id;in property<string> selected-name;in property<string> selected-kind;in property<bool> selected-online;in property<string> selected-latency;
  callback query-changed(string);callback online-changed(bool);callback load-changed(float);callback selected(int);callback reset();
  if !root.empty:VerticalLayout{
   padding:24px;spacing:16px;alignment:start;
   HorizontalLayout{height:60px;
    VerticalLayout{spacing:5px;alignment:center;Text{text:"设备控制台";font-size:28px;font-weight:600;color:#1b1b1b;}Caption{text:"查看连接状态，调整模拟负载";}}
    Rectangle{horizontal-stretch:1;}
    Caption{text:"DEVICE CONSOLE  /  本地演示";vertical-alignment:center;}
   }
   Rectangle{height:96px;
    StatCard{x:0px;width:(parent.width - 32px)/3;heading:"在线设备";value:"04";unit:"/ 6 台";accent:true;}
    StatCard{x:(parent.width - 32px)/3 + 16px;width:(parent.width - 32px)/3;heading:"平均响应";value:"20";unit:"ms";}
    StatCard{x:2*((parent.width - 32px)/3 + 16px);width:(parent.width - 32px)/3;heading:"模拟负载";value:"" + round(root.load);unit:"%";}
   }
   HorizontalLayout{height:440px;spacing:16px;
    Rectangle{background:white;border-radius:12px;horizontal-stretch:1;
     VerticalLayout{padding:18px;spacing:12px;alignment:start;
      HorizontalLayout{height:24px;Text{text:"设备列表";font-weight:600;font-size:14px;}Rectangle{horizontal-stretch:1;}Caption{text:root.devices.length + " 台设备";}}
      HorizontalLayout{height:34px;spacing:12px;LineEdit{text <=> root.query;placeholder-text:"搜索名称或类型";edited=>{root.query-changed(self.text);}}CheckBox{text:"仅在线";checked <=> root.online-only;toggled=>{root.online-changed(self.checked);}}}
      VerticalLayout{spacing:5px;alignment:start;
       for device in root.devices:Rectangle{height:48px;background:device.id==root.selected-id?#f3f3f3:white;border-radius:8px;
        Rectangle{x:7px;y:10px;width:3px;height:28px;border-radius:2px;background:device.id==root.selected-id?#0067c0:white;}
        Text{x:22px;y:5px;width:parent.width - 138px;height:20px;text:device.name;font-size:14px;font-weight:600;color:device.id==root.selected-id?#0067c0:#1b1b1b;}
        Caption{x:22px;y:27px;width:parent.width - 138px;text:device.kind;}
        Caption{x:parent.width - 110px;y:16px;width:66px;text:device.online?"● 在线":"○ 离线";color:device.online?#0f7b0f:#616161;}
        Button{x:parent.width - 35px;y:10px;width:28px;height:28px;text:"›";clicked=>{root.selected(device.id);}}
       }
      }
     }
    }
    Rectangle{background:white;border-radius:12px;width:260px;
     VerticalLayout{padding:18px;spacing:10px;alignment:start;
      Caption{text:"DEVICE " + (root.selected-id+1);}
      Text{text:root.selected-name;font-size:20px;font-weight:600;height:28px;color:#1b1b1b;}
      Caption{text:root.selected-kind;}
      Rectangle{height:8px;}
      HorizontalLayout{height:24px;Caption{text:"连接状态";}Rectangle{horizontal-stretch:1;}Text{text:root.selected-online?"在线":"离线";font-size:14px;font-weight:600;color:root.selected-online?#0f7b0f:#616161;}}
      HorizontalLayout{height:24px;Caption{text:"响应时间";}Rectangle{horizontal-stretch:1;}Text{text:root.selected-latency;font-size:14px;font-weight:600;}}
      Rectangle{height:1px;background:#d1d1d1;}
      HorizontalLayout{height:28px;Text{text:"负载模拟";font-size:14px;font-weight:600;}Rectangle{horizontal-stretch:1;}Text{text:round(root.load)+"%";font-size:14px;color:#0067c0;}}
      Slider{value <=> root.load;minimum:0;maximum:100;height:28px;changed=>{root.load-changed(self.value);}}
      ProgressIndicator{progress:root.load/100;height:4px;}
      Caption{text:"拖动滑块调整演示数据";}
      Rectangle{vertical-stretch:1;}
      Button{text:"恢复默认";primary:true;height:34px;clicked=>{root.reset();}}
     }
    }
   }
  }
 }
}
fn refresh(ui: &DeviceWindow, state: &State) {
    let device = model::DEVICES[state.selected];
    ui.set_query(state.query.clone().into());
    ui.set_online_only(state.online_only);
    ui.set_load(state.load);
    ui.set_selected_id(state.selected as i32);
    ui.set_selected_name(device.name.split(" / ").next().unwrap().into());
    ui.set_selected_kind(device.kind.into());
    ui.set_selected_online(device.online);
    ui.set_selected_latency(
        if device.online {
            format!("{} ms", device.latency)
        } else {
            "—".into()
        }
        .into(),
    );
    ui.set_devices(ModelRc::new(VecModel::from(
        state
            .visible()
            .into_iter()
            .map(|(id, d)| DeviceRow {
                id: id as i32,
                name: d.name.split(" / ").next().unwrap().into(),
                kind: d.kind.into(),
                online: d.online,
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
        ($method:ident,$message:expr) => {{
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
    handler!(on_query_changed, |v: SharedString| Message::Query(
        v.to_string()
    ));
    handler!(on_online_changed, Message::OnlineOnly);
    handler!(on_load_changed, Message::Load);
    handler!(on_selected, |v: i32| Message::Select(v as usize));
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
        let state = state.clone();
        let started = std::time::Instant::now();
        repaint.start(
            TimerMode::Repeated,
            std::time::Duration::from_millis(16),
            move || {
                if let Some(ui) = weak.upgrade() {
                    let value = model::benchmark_load(started);
                    state.borrow_mut().load = value;
                    ui.set_load(value);
                }
            },
        );
    }
    ui.run()
}
