use zsui::{
    AppEvent, Command, HostCapabilities, NativeMainWindowHandles, NativeRuntimeDriver,
    NativeRuntimeStartupRequest, NativeRuntimeStartupResult, PlatformName, ProductAdapterHost,
    ProductAdapterIdentity, ProductAdapterRuntimeSmokeRequest, ProductAiCapabilityDescriptor,
    ProductAiExecutionPlan, ProductAiInvocation, ProductAiProviderFamily, ProductAiResult,
    ProductUiProjection, SettingsItemSpec, SettingsPageSpec, TraySpec, UiCommand, Window,
    ZsuiResult, ZsuiReusableRuntimeHarness, PRODUCT_ADAPTER_SMOKE_COMMAND,
};

#[derive(Default)]
struct SmokeDriver {
    events: Vec<AppEvent>,
}

impl NativeRuntimeDriver<UiCommand, AppEvent> for SmokeDriver {
    type WindowHandle = u32;

    fn start_runtime(
        &mut self,
        _request: NativeRuntimeStartupRequest,
    ) -> NativeRuntimeStartupResult<Self::WindowHandle> {
        self.events.push(AppEvent::Started);
        NativeRuntimeStartupResult::Started(NativeMainWindowHandles { main: 1, quick: 2 })
    }

    fn dispatch_ui_command(&mut self, _command: UiCommand) {}

    fn poll_application_event(&mut self) -> Option<AppEvent> {
        self.events.pop()
    }

    fn request_shutdown(&mut self) {}
}

struct SmokeProduct {
    async_event_sent: bool,
}

impl ProductAdapterHost for SmokeProduct {
    fn identity(&self) -> ProductAdapterIdentity {
        ProductAdapterIdentity::new("demo", "Demo", "0.1.0")
    }

    fn project_ui(&self) -> ProductUiProjection {
        ProductUiProjection::new("Demo", Window::new("Demo").size(720, 460))
            .status_item(
                TraySpec::new()
                    .tooltip("Demo")
                    .item("Open", Command::ShowMainWindow)
                    .item("Quit", Command::Quit),
            )
            .settings_page(SettingsPageSpec::new("general", "General").item(
                SettingsItemSpec::toggle("launch_on_startup", "Launch on startup", false),
            ))
    }

    fn execute_ui_command(&mut self, command: UiCommand) -> ZsuiResult<Vec<AppEvent>> {
        Ok(vec![AppEvent::Custom {
            id: command.id.0.to_string(),
            payload: None,
        }])
    }

    fn poll_async_event(&mut self) -> Option<AppEvent> {
        if self.async_event_sent {
            return None;
        }
        self.async_event_sent = true;
        Some(AppEvent::Custom {
            id: "demo.async".to_string(),
            payload: None,
        })
    }

    fn product_ai_capabilities(&self) -> Vec<ProductAiCapabilityDescriptor> {
        vec![ProductAiCapabilityDescriptor::new(
            "demo.summarize",
            ProductAiProviderFamily::ProductTool,
            "summarize_selection",
            "main_window",
            "text",
        )
        .required_context("selected_text")]
    }

    fn execute_ai_plan(&mut self, plan: ProductAiExecutionPlan) -> ZsuiResult<ProductAiResult> {
        Ok(ProductAiResult::text(
            plan.invocation.invocation_id,
            plan.result_kind,
            "summary from product adapter smoke",
        ))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut harness = ZsuiReusableRuntimeHarness::new(
        SmokeDriver::default(),
        SmokeProduct {
            async_event_sent: false,
        },
        HostCapabilities::all_supported(PlatformName::Unknown),
    );
    let request = ProductAdapterRuntimeSmokeRequest::quick().ai_invocation(
        ProductAiInvocation::new("demo.summarize")
            .invocation_id("product-adapter-smoke")
            .source_surface("main_window")
            .input_text("selected text"),
    );
    let report = harness.run_smoke(request);

    assert!(report.harness_smoke_complete);
    assert_eq!(report.command_id, PRODUCT_ADAPTER_SMOKE_COMMAND.0);
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
