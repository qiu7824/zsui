use serde_json::json;
use zsui::{
    AppEvent, Command, NativeWindowRuntimeDriver, ProductAdapterHost, ProductAdapterIdentity,
    ProductAdapterRuntimeSmokeRequest, ProductAiCapabilityDescriptor, ProductAiExecutionPlan,
    ProductAiInvocation, ProductAiProviderFamily, ProductAiResult, ProductUiProjection,
    SettingsItemSpec, SettingsPageSpec, TraySpec, UiCommand, Window, ZsuiResult,
    ZsuiReusableRuntimeHarness, PRODUCT_ADAPTER_SMOKE_COMMAND,
};

struct NativeDriverDemoProduct {
    async_event_sent: bool,
}

impl ProductAdapterHost for NativeDriverDemoProduct {
    fn identity(&self) -> ProductAdapterIdentity {
        ProductAdapterIdentity::new("native-driver-demo", "Native Driver Demo", "0.1.0")
    }

    fn project_ui(&self) -> ProductUiProjection {
        ProductUiProjection::new(
            "Native Driver Demo",
            Window::new("Native Driver Demo").size(720, 460),
        )
        .status_item(
            TraySpec::new()
                .tooltip("Native Driver Demo")
                .item("Open", Command::ShowMainWindow)
                .item("Quit", Command::Quit),
        )
        .settings_page(
            SettingsPageSpec::new("general", "General").item(SettingsItemSpec::toggle(
                "launch_on_startup",
                "Launch on startup",
                false,
            )),
        )
    }

    fn execute_ui_command(&mut self, command: UiCommand) -> ZsuiResult<Vec<AppEvent>> {
        Ok(vec![AppEvent::Custom {
            id: command.id.0.to_string(),
            payload: Some("handled_by_product_adapter".to_string()),
        }])
    }

    fn poll_async_event(&mut self) -> Option<AppEvent> {
        if self.async_event_sent {
            return None;
        }
        self.async_event_sent = true;
        Some(AppEvent::Custom {
            id: "native-driver-demo.async".to_string(),
            payload: None,
        })
    }

    fn product_ai_capabilities(&self) -> Vec<ProductAiCapabilityDescriptor> {
        vec![ProductAiCapabilityDescriptor::new(
            "native-driver-demo.summarize",
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
            "summary from native driver demo",
        ))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut harness = ZsuiReusableRuntimeHarness::new(
        NativeWindowRuntimeDriver::new(),
        NativeDriverDemoProduct {
            async_event_sent: false,
        },
        zsui::HostCapabilities::current_native_window_host(),
    );
    let request =
        ProductAdapterRuntimeSmokeRequest::new(UiCommand::app(PRODUCT_ADAPTER_SMOKE_COMMAND))
            .ai_invocation(
                ProductAiInvocation::new("native-driver-demo.summarize")
                    .invocation_id("native-driver-demo-smoke")
                    .source_surface("main_window")
                    .input_text("selected text"),
            );
    let report = harness.run_smoke(request);
    let driver_report = harness.driver().report();

    assert!(report.harness_smoke_complete);
    assert!(driver_report.startup_request_count >= 1);
    assert!(driver_report.handles_created);
    assert_eq!(driver_report.status_menu_entry_count, 2);
    assert_eq!(driver_report.settings_page_count, 1);
    assert!(driver_report
        .native_operation_names
        .contains(&"create_status_item"));
    assert!(driver_report
        .native_operation_names
        .contains(&"bind_settings_pages"));
    assert!(driver_report
        .native_operation_names
        .contains(&"destroy_status_item"));
    assert!(driver_report
        .native_operation_names
        .contains(&"clear_settings_pages"));
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "product_adapter": report,
            "native_driver": driver_report,
        }))?
    );
    Ok(())
}
