use zsui::{
    button, column, text, AppCx, AppEvent, Command, HostCapabilities, NativeMainWindowHandles,
    NativeRuntimeDriver, NativeRuntimeStartupRequest, NativeRuntimeStartupResult,
    ProductAdapterHost, ProductAdapterIdentity, ProductUiProjection, ProductViewAdapterHost,
    ProductViewRuntimeSmokeRequest, UiCommand, ViewEvent, ViewNode, WidgetId, WindowSpec,
    ZsuiResult, ZsuiReusableRuntimeHarness, PRODUCT_ADAPTER_SMOKE_COMMAND,
};

const SAVE_BUTTON: WidgetId = WidgetId::new(1);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Msg {
    SaveClicked,
}

#[derive(Default)]
struct DemoDriver {
    commands: Vec<UiCommand>,
    shutdown: bool,
}

impl NativeRuntimeDriver<UiCommand, AppEvent> for DemoDriver {
    type WindowHandle = u32;

    fn start_runtime(
        &mut self,
        _request: NativeRuntimeStartupRequest,
    ) -> NativeRuntimeStartupResult<Self::WindowHandle> {
        NativeRuntimeStartupResult::Started(NativeMainWindowHandles { main: 1, quick: 2 })
    }

    fn dispatch_ui_command(&mut self, command: UiCommand) {
        self.commands.push(command);
    }

    fn poll_application_event(&mut self) -> Option<AppEvent> {
        None
    }

    fn request_shutdown(&mut self) {
        self.shutdown = true;
    }
}

struct DemoProduct;

impl ProductAdapterHost for DemoProduct {
    fn identity(&self) -> ProductAdapterIdentity {
        ProductAdapterIdentity::new("view-demo", "View Demo", "0.1.0")
    }

    fn project_ui(&self) -> ProductUiProjection {
        ProductUiProjection::new("View Demo", WindowSpec::new("View Demo").size(640, 420))
    }

    fn execute_ui_command(&mut self, command: UiCommand) -> ZsuiResult<Vec<AppEvent>> {
        Ok(vec![AppEvent::Custom {
            id: command.id.0.to_string(),
            payload: None,
        }])
    }
}

impl ProductViewAdapterHost<Msg> for DemoProduct {
    fn project_view(&self) -> ViewNode<Msg> {
        column(vec![
            text("View-driven product adapter"),
            button("Save").id(SAVE_BUTTON).on_click(Msg::SaveClicked),
        ])
    }

    fn update_view_message(&mut self, msg: Msg, cx: &mut AppCx) -> ZsuiResult<Vec<AppEvent>> {
        match msg {
            Msg::SaveClicked => {
                cx.command(Command::custom("view-demo.save"));
                cx.ui_command(UiCommand::app(PRODUCT_ADAPTER_SMOKE_COMMAND));
                Ok(vec![AppEvent::Custom {
                    id: "view-demo.save".to_string(),
                    payload: None,
                }])
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut harness = ZsuiReusableRuntimeHarness::new(
        DemoDriver::default(),
        DemoProduct,
        HostCapabilities::windows_scaffold(),
    );
    let report = harness.run_view_smoke(
        ProductViewRuntimeSmokeRequest::new(zsui::Rect {
            x: 0,
            y: 0,
            width: 320,
            height: 96,
        })
        .event(ViewEvent::Click {
            widget: SAVE_BUTTON,
        }),
    );

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
