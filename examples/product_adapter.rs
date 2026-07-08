use zsui::{
    AppEvent, Command, CommandId, HostCapabilities, NativeMainWindowHandles, NativeRuntimeDriver,
    NativeRuntimeStartupRequest, NativeRuntimeStartupResult, PlatformName, ProductAdapterHost,
    ProductAdapterIdentity, ProductAiCapabilityDescriptor, ProductAiExecutionPlan,
    ProductAiInvocation, ProductAiProviderFamily, ProductAiResult, ProductUiProjection,
    SettingsItemSpec, SettingsPageSpec, TraySpec, UiCommand, Window, ZsuiResult,
    ZsuiReusableRuntimeHarness,
};

#[derive(Default)]
struct DemoDriver {
    commands: Vec<UiCommand>,
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

    fn request_shutdown(&mut self) {}
}

struct DemoProduct;

impl ProductAdapterHost for DemoProduct {
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
            "summary from product adapter",
        ))
    }
}

fn main() -> ZsuiResult<()> {
    let mut harness = ZsuiReusableRuntimeHarness::new(
        DemoDriver::default(),
        DemoProduct,
        HostCapabilities::all_supported(PlatformName::Unknown),
    );

    harness.start()?;
    harness.dispatch_ui_command(UiCommand::app(CommandId("demo.refresh")))?;
    let ai = harness.route_ai_invocation(
        ProductAiInvocation::new("demo.summarize")
            .invocation_id("example")
            .source_surface("main_window")
            .input_text("selected text"),
    )?;
    assert_eq!(
        ai.output_text.as_deref(),
        Some("summary from product adapter")
    );
    harness.request_shutdown();
    Ok(())
}
