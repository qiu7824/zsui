use serde::Serialize;

use crate::{
    app, AppCx, AppEvent, CommandId, Dpi, HostCapabilities, NativeDrawPlan,
    NativeMainWindowHandles, NativeMainWindowRequest, NativeRuntimeDriver,
    NativeRuntimeStartupRequest, NativeRuntimeStartupResult, Rect, SettingsPageSpec, TraySpec,
    UiCommand, View, ViewEvent, ViewEventCx, ViewLayoutCx, ViewNode, ViewPaintCx, WindowSpec,
    ZsuiAppDeclarationReport, ZsuiError, ZsuiResult,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductAdapterIdentity {
    pub product_id: String,
    pub display_name: String,
    pub version: String,
}

impl ProductAdapterIdentity {
    pub fn new(
        product_id: impl Into<String>,
        display_name: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        Self {
            product_id: product_id.into(),
            display_name: display_name.into(),
            version: version.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductUiProjection {
    pub app_name: String,
    pub main_window: WindowSpec,
    pub status_item_tooltip: Option<String>,
    pub status_item: Option<TraySpec>,
    pub settings_pages: Vec<SettingsPageSpec>,
}

impl ProductUiProjection {
    pub fn new(app_name: impl Into<String>, main_window: WindowSpec) -> Self {
        Self {
            app_name: app_name.into(),
            main_window,
            status_item_tooltip: None,
            status_item: None,
            settings_pages: Vec::new(),
        }
    }

    pub fn status_item_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        let tooltip = tooltip.into();
        self.status_item_tooltip = Some(tooltip.clone());
        let mut status_item = self.status_item.take().unwrap_or_default();
        status_item.tooltip = Some(tooltip);
        self.status_item = Some(status_item);
        self
    }

    pub fn status_item(mut self, status_item: TraySpec) -> Self {
        if self.status_item_tooltip.is_none() {
            self.status_item_tooltip = status_item.tooltip.clone();
        }
        self.status_item = Some(status_item);
        self
    }

    pub fn tray(self, tray: TraySpec) -> Self {
        self.status_item(tray)
    }

    pub fn status_item_declared(&self) -> bool {
        self.status_item.is_some() || self.status_item_tooltip.is_some()
    }

    pub fn declaration_report_for(
        &self,
        capabilities: &HostCapabilities,
    ) -> ZsuiAppDeclarationReport {
        let mut builder = app(self.app_name.clone()).window(self.main_window.clone());
        if let Some(status_item) = &self.status_item {
            builder = builder.tray(status_item.clone());
        }
        for page in &self.settings_pages {
            builder = builder.settings_page(page.clone());
        }
        builder.declaration_report_for(capabilities)
    }

    pub fn ensure_valid_for(&self, capabilities: &HostCapabilities) -> ZsuiResult<()> {
        self.declaration_report_for(capabilities).ensure_valid()
    }

    pub fn status_item_for_startup(&self) -> Option<TraySpec> {
        self.status_item.clone().or_else(|| {
            self.status_item_tooltip
                .clone()
                .map(|tooltip| TraySpec::new().tooltip(tooltip))
        })
    }

    pub fn status_item_tooltip_for_startup(&self) -> Option<String> {
        self.status_item_tooltip.clone().or_else(|| {
            self.status_item
                .as_ref()
                .and_then(|item| item.tooltip.clone())
        })
    }

    pub fn settings_pages_for_startup(&self) -> Vec<SettingsPageSpec> {
        self.settings_pages.clone()
    }

    pub fn startup_request_for(
        &self,
        capabilities: &HostCapabilities,
    ) -> ZsuiResult<NativeRuntimeStartupRequest> {
        self.ensure_valid_for(capabilities)?;
        Ok(NativeRuntimeStartupRequest {
            app_name: self.app_name.clone(),
            main_window: NativeMainWindowRequest::from_zsui_window_for_host(
                &self.main_window,
                capabilities,
            ),
            status_item_tooltip: self.status_item_tooltip_for_startup(),
            status_item: self.status_item_for_startup(),
            settings_pages: self.settings_pages_for_startup(),
        })
    }

    pub fn startup_request(&self) -> ZsuiResult<NativeRuntimeStartupRequest> {
        self.startup_request_for(&HostCapabilities::all_supported(
            crate::PlatformName::Unknown,
        ))
    }

    pub fn settings_pages(mut self, pages: impl IntoIterator<Item = SettingsPageSpec>) -> Self {
        self.settings_pages.extend(pages);
        self
    }

    pub fn settings_page(mut self, page: SettingsPageSpec) -> Self {
        self.settings_pages.push(page);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ProductAdapterSurface {
    Identity,
    StateProjection,
    CommandExecutor,
    SettingsModel,
    AsyncEventBridge,
    AiCapabilityCatalog,
    AiPlanExecutor,
}

impl ProductAdapterSurface {
    pub const fn surface_name(self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::StateProjection => "state_projection",
            Self::CommandExecutor => "command_executor",
            Self::SettingsModel => "settings_model",
            Self::AsyncEventBridge => "async_event_bridge",
            Self::AiCapabilityCatalog => "ai_capability_catalog",
            Self::AiPlanExecutor => "ai_plan_executor",
        }
    }
}

pub const REQUIRED_PRODUCT_ADAPTER_SURFACES: [ProductAdapterSurface; 7] = [
    ProductAdapterSurface::Identity,
    ProductAdapterSurface::StateProjection,
    ProductAdapterSurface::CommandExecutor,
    ProductAdapterSurface::SettingsModel,
    ProductAdapterSurface::AsyncEventBridge,
    ProductAdapterSurface::AiCapabilityCatalog,
    ProductAdapterSurface::AiPlanExecutor,
];

pub fn required_product_adapter_surface_names() -> Vec<&'static str> {
    REQUIRED_PRODUCT_ADAPTER_SURFACES
        .iter()
        .map(|surface| surface.surface_name())
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ProductAdapterTask {
    ProvideIdentity,
    ProjectProductState,
    ExecuteUiCommand,
    BindSettings,
    BridgeAsyncEvents,
    PublishAiCatalog,
    PlanAiInvocation,
    ExecuteAiPlan,
}

impl ProductAdapterTask {
    pub const fn task_name(self) -> &'static str {
        match self {
            Self::ProvideIdentity => "provide_identity",
            Self::ProjectProductState => "project_product_state",
            Self::ExecuteUiCommand => "execute_ui_command",
            Self::BindSettings => "bind_settings",
            Self::BridgeAsyncEvents => "bridge_async_events",
            Self::PublishAiCatalog => "publish_ai_catalog",
            Self::PlanAiInvocation => "plan_ai_invocation",
            Self::ExecuteAiPlan => "execute_ai_plan",
        }
    }
}

pub const REQUIRED_PRODUCT_ADAPTER_TASKS: [ProductAdapterTask; 8] = [
    ProductAdapterTask::ProvideIdentity,
    ProductAdapterTask::ProjectProductState,
    ProductAdapterTask::ExecuteUiCommand,
    ProductAdapterTask::BindSettings,
    ProductAdapterTask::BridgeAsyncEvents,
    ProductAdapterTask::PublishAiCatalog,
    ProductAdapterTask::PlanAiInvocation,
    ProductAdapterTask::ExecuteAiPlan,
];

pub fn required_product_adapter_task_names() -> Vec<&'static str> {
    REQUIRED_PRODUCT_ADAPTER_TASKS
        .iter()
        .map(|task| task.task_name())
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ProductAiProviderFamily {
    Llm,
    Skill,
    ProductTool,
}

impl ProductAiProviderFamily {
    pub const fn provider_family_name(self) -> &'static str {
        match self {
            Self::Llm => "llm",
            Self::Skill => "skill",
            Self::ProductTool => "product_tool",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ProductAiExecutorBoundary {
    LlmExecutor,
    SkillRegistry,
    ProductAdapterTools,
}

impl ProductAiExecutorBoundary {
    pub const fn boundary_name(self) -> &'static str {
        match self {
            Self::LlmExecutor => "llm_executor",
            Self::SkillRegistry => "skill_registry",
            Self::ProductAdapterTools => "product_adapter_tools",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductAiCapabilityDescriptor {
    pub capability_id: String,
    pub provider_family: ProductAiProviderFamily,
    pub action_name: String,
    pub source_surface: String,
    pub required_context_names: Vec<String>,
    pub result_kind: String,
}

impl ProductAiCapabilityDescriptor {
    pub fn new(
        capability_id: impl Into<String>,
        provider_family: ProductAiProviderFamily,
        action_name: impl Into<String>,
        source_surface: impl Into<String>,
        result_kind: impl Into<String>,
    ) -> Self {
        Self {
            capability_id: capability_id.into(),
            provider_family,
            action_name: action_name.into(),
            source_surface: source_surface.into(),
            required_context_names: Vec::new(),
            result_kind: result_kind.into(),
        }
    }

    pub fn required_context(mut self, name: impl Into<String>) -> Self {
        self.required_context_names.push(name.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductAiInvocation {
    pub invocation_id: String,
    pub capability_id: String,
    pub source_surface: String,
    pub input_text: Option<String>,
    pub item_ids: Vec<String>,
    pub payload_json: Option<String>,
}

impl ProductAiInvocation {
    pub fn new(capability_id: impl Into<String>) -> Self {
        let capability_id = capability_id.into();
        Self {
            invocation_id: capability_id.clone(),
            capability_id,
            source_surface: "unknown".to_string(),
            input_text: None,
            item_ids: Vec::new(),
            payload_json: None,
        }
    }

    pub fn invocation_id(mut self, id: impl Into<String>) -> Self {
        self.invocation_id = id.into();
        self
    }

    pub fn source_surface(mut self, surface: impl Into<String>) -> Self {
        self.source_surface = surface.into();
        self
    }

    pub fn input_text(mut self, text: impl Into<String>) -> Self {
        self.input_text = Some(text.into());
        self
    }

    pub fn item_id(mut self, id: impl Into<String>) -> Self {
        self.item_ids.push(id.into());
        self
    }

    pub fn payload_json(mut self, payload: impl Into<String>) -> Self {
        self.payload_json = Some(payload.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductAiExecutionPlan {
    pub invocation: ProductAiInvocation,
    pub provider_family: ProductAiProviderFamily,
    pub executor_boundary: ProductAiExecutorBoundary,
    pub route_name: String,
    pub result_kind: String,
}

impl ProductAiExecutionPlan {
    pub fn from_descriptor(
        invocation: ProductAiInvocation,
        descriptor: ProductAiCapabilityDescriptor,
    ) -> Self {
        let executor_boundary = match descriptor.provider_family {
            ProductAiProviderFamily::Llm => ProductAiExecutorBoundary::LlmExecutor,
            ProductAiProviderFamily::Skill => ProductAiExecutorBoundary::SkillRegistry,
            ProductAiProviderFamily::ProductTool => ProductAiExecutorBoundary::ProductAdapterTools,
        };
        Self {
            invocation,
            provider_family: descriptor.provider_family,
            executor_boundary,
            route_name: descriptor.action_name,
            result_kind: descriptor.result_kind,
        }
    }

    pub const fn executor_boundary_name(&self) -> &'static str {
        self.executor_boundary.boundary_name()
    }

    pub const fn provider_family_name(&self) -> &'static str {
        self.provider_family.provider_family_name()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductAiResult {
    pub invocation_id: String,
    pub result_kind: String,
    pub output_text: Option<String>,
    pub emitted_events: Vec<AppEvent>,
}

impl ProductAiResult {
    pub fn text(
        invocation_id: impl Into<String>,
        result_kind: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            invocation_id: invocation_id.into(),
            result_kind: result_kind.into(),
            output_text: Some(text.into()),
            emitted_events: Vec::new(),
        }
    }

    pub fn event(mut self, event: AppEvent) -> Self {
        self.emitted_events.push(event);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductAdapterReuseChecklist {
    pub surface_names: Vec<&'static str>,
    pub task_names: Vec<&'static str>,
    pub ai_provider_family_names: Vec<&'static str>,
    pub ai_executor_boundary_names: Vec<&'static str>,
}

pub fn product_adapter_reuse_checklist() -> ProductAdapterReuseChecklist {
    ProductAdapterReuseChecklist {
        surface_names: required_product_adapter_surface_names(),
        task_names: required_product_adapter_task_names(),
        ai_provider_family_names: vec![
            ProductAiProviderFamily::Llm.provider_family_name(),
            ProductAiProviderFamily::Skill.provider_family_name(),
            ProductAiProviderFamily::ProductTool.provider_family_name(),
        ],
        ai_executor_boundary_names: vec![
            ProductAiExecutorBoundary::LlmExecutor.boundary_name(),
            ProductAiExecutorBoundary::SkillRegistry.boundary_name(),
            ProductAiExecutorBoundary::ProductAdapterTools.boundary_name(),
        ],
    }
}

pub trait ProductAdapterHost {
    fn identity(&self) -> ProductAdapterIdentity;
    fn project_ui(&self) -> ProductUiProjection;
    fn execute_ui_command(&mut self, command: UiCommand) -> ZsuiResult<Vec<AppEvent>>;

    fn settings_pages(&self) -> Vec<SettingsPageSpec> {
        self.project_ui().settings_pages
    }

    fn poll_async_event(&mut self) -> Option<AppEvent> {
        None
    }

    fn product_ai_capabilities(&self) -> Vec<ProductAiCapabilityDescriptor> {
        Vec::new()
    }

    fn plan_ai_invocation(
        &mut self,
        invocation: ProductAiInvocation,
    ) -> ZsuiResult<ProductAiExecutionPlan> {
        let Some(descriptor) = self
            .product_ai_capabilities()
            .into_iter()
            .find(|descriptor| descriptor.capability_id == invocation.capability_id)
        else {
            return Err(ZsuiError::unsupported(
                "product_ai_capability",
                format!("unknown AI capability `{}`", invocation.capability_id),
            ));
        };
        Ok(ProductAiExecutionPlan::from_descriptor(
            invocation, descriptor,
        ))
    }

    fn execute_ai_plan(&mut self, plan: ProductAiExecutionPlan) -> ZsuiResult<ProductAiResult> {
        Err(ZsuiError::unsupported(
            "product_ai_execution",
            format!(
                "no executor is attached for `{}` through `{}`",
                plan.invocation.capability_id,
                plan.executor_boundary_name()
            ),
        ))
    }

    fn request_shutdown(&mut self) {}
}

pub trait ProductViewAdapterHost<Msg>: ProductAdapterHost {
    fn project_view(&self) -> ViewNode<Msg>;
    fn update_view_message(&mut self, message: Msg, cx: &mut AppCx) -> ZsuiResult<Vec<AppEvent>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ZsuiReusableRuntimeHarnessStage {
    ProjectUi,
    StartNativeRuntime,
    DispatchUiCommand,
    PollNativeEvent,
    PollProductEvent,
    PlanAiInvocation,
    ExecuteAiPlan,
    RequestShutdown,
}

impl ZsuiReusableRuntimeHarnessStage {
    pub const fn stage_name(self) -> &'static str {
        match self {
            Self::ProjectUi => "project_ui",
            Self::StartNativeRuntime => "start_native_runtime",
            Self::DispatchUiCommand => "dispatch_ui_command",
            Self::PollNativeEvent => "poll_native_event",
            Self::PollProductEvent => "poll_product_event",
            Self::PlanAiInvocation => "plan_ai_invocation",
            Self::ExecuteAiPlan => "execute_ai_plan",
            Self::RequestShutdown => "request_shutdown",
        }
    }
}

pub const ZSUI_REUSABLE_RUNTIME_HARNESS_STAGES: [ZsuiReusableRuntimeHarnessStage; 8] = [
    ZsuiReusableRuntimeHarnessStage::ProjectUi,
    ZsuiReusableRuntimeHarnessStage::StartNativeRuntime,
    ZsuiReusableRuntimeHarnessStage::DispatchUiCommand,
    ZsuiReusableRuntimeHarnessStage::PollNativeEvent,
    ZsuiReusableRuntimeHarnessStage::PollProductEvent,
    ZsuiReusableRuntimeHarnessStage::PlanAiInvocation,
    ZsuiReusableRuntimeHarnessStage::ExecuteAiPlan,
    ZsuiReusableRuntimeHarnessStage::RequestShutdown,
];

pub fn zsui_reusable_runtime_harness_stage_names() -> Vec<&'static str> {
    ZSUI_REUSABLE_RUNTIME_HARNESS_STAGES
        .iter()
        .map(|stage| stage.stage_name())
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductAdapterRuntimeSmokeRequest {
    pub command: UiCommand,
    pub ai_invocation: Option<ProductAiInvocation>,
}

impl ProductAdapterRuntimeSmokeRequest {
    pub const fn new(command: UiCommand) -> Self {
        Self {
            command,
            ai_invocation: None,
        }
    }

    pub const fn quick() -> Self {
        Self::new(UiCommand::app(PRODUCT_ADAPTER_SMOKE_COMMAND))
    }

    pub fn ai_invocation(mut self, invocation: ProductAiInvocation) -> Self {
        self.ai_invocation = Some(invocation);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductAdapterRuntimeSmokeReport {
    pub product: ProductAdapterIdentity,
    pub app_name: String,
    pub main_window_title: String,
    pub status_item_declared: bool,
    pub settings_page_count: usize,
    pub ai_capability_count: usize,
    pub started: bool,
    pub handles_created: bool,
    pub command_id: &'static str,
    pub command_event_count: usize,
    pub native_event_polled: Option<AppEvent>,
    pub product_event_polled: Option<AppEvent>,
    pub ai_invocation_id: Option<String>,
    pub ai_capability_id: Option<String>,
    pub ai_result_kind: Option<String>,
    pub ai_output_present: bool,
    pub ai_emitted_event_count: usize,
    pub shutdown_requested: bool,
    pub started_after_shutdown: bool,
    pub exercised_stage_names: Vec<&'static str>,
    pub missing_stage_names: Vec<&'static str>,
    pub errors: Vec<String>,
    pub harness_smoke_complete: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProductViewRuntimeSmokeRequest {
    pub bounds: Rect,
    pub dpi: Dpi,
    pub events: Vec<ViewEvent>,
}

impl ProductViewRuntimeSmokeRequest {
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            dpi: Dpi::standard(),
            events: Vec::new(),
        }
    }

    pub fn dpi(mut self, dpi: Dpi) -> Self {
        self.dpi = dpi;
        self
    }

    pub fn event(mut self, event: ViewEvent) -> Self {
        self.events.push(event);
        self
    }

    pub fn events(mut self, events: impl IntoIterator<Item = ViewEvent>) -> Self {
        self.events.extend(events);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductViewRuntimeSmokeReport {
    pub product: ProductAdapterIdentity,
    pub view_projected: bool,
    pub layout_node_count: usize,
    pub view_event_count: usize,
    pub view_message_count: usize,
    pub app_command_count: usize,
    pub ui_command_count: usize,
    pub dispatched_ui_command_count: usize,
    pub product_event_count: usize,
    pub draw_command_count: usize,
    pub text_command_count: usize,
    pub errors: Vec<String>,
    pub view_smoke_complete: bool,
}

pub fn product_adapter_runtime_smoke_example_names() -> Vec<&'static str> {
    vec![
        "product_adapter",
        "product_adapter_smoke",
        "product_adapter_native_driver",
        "product_adapter_view",
    ]
}

#[derive(Debug, Clone)]
pub struct ZsuiReusableRuntimeHarness<Driver, Product>
where
    Driver: NativeRuntimeDriver<UiCommand, AppEvent>,
    Product: ProductAdapterHost,
{
    driver: Driver,
    product: Product,
    capabilities: HostCapabilities,
    handles: Option<NativeMainWindowHandles<Driver::WindowHandle>>,
    started: bool,
}

impl<Driver, Product> ZsuiReusableRuntimeHarness<Driver, Product>
where
    Driver: NativeRuntimeDriver<UiCommand, AppEvent>,
    Product: ProductAdapterHost,
{
    pub fn new(driver: Driver, product: Product, capabilities: HostCapabilities) -> Self {
        Self {
            driver,
            product,
            capabilities,
            handles: None,
            started: false,
        }
    }

    pub fn driver(&self) -> &Driver {
        &self.driver
    }

    pub fn driver_mut(&mut self) -> &mut Driver {
        &mut self.driver
    }

    pub fn product(&self) -> &Product {
        &self.product
    }

    pub fn product_mut(&mut self) -> &mut Product {
        &mut self.product
    }

    pub const fn handles(&self) -> Option<NativeMainWindowHandles<Driver::WindowHandle>> {
        self.handles
    }

    pub const fn started(&self) -> bool {
        self.started
    }

    pub fn start(&mut self) -> ZsuiResult<NativeMainWindowHandles<Driver::WindowHandle>> {
        let projection = self.product.project_ui();
        let startup_request = projection.startup_request_for(&self.capabilities)?;
        let result = self.driver.start_runtime(startup_request);
        match result {
            NativeRuntimeStartupResult::Started(handles) => {
                self.started = true;
                self.handles = Some(handles);
                Ok(handles)
            }
            NativeRuntimeStartupResult::Failed => Err(ZsuiError::host(
                "start_native_runtime",
                "native runtime driver reported startup failure",
            )),
        }
    }

    pub fn dispatch_ui_command(&mut self, command: UiCommand) -> ZsuiResult<Vec<AppEvent>> {
        let events = self.product.execute_ui_command(command.clone())?;
        self.driver.dispatch_ui_command(command);
        Ok(events)
    }

    pub fn poll_native_event(&mut self) -> Option<AppEvent> {
        self.driver.poll_application_event()
    }

    pub fn poll_product_event(&mut self) -> Option<AppEvent> {
        self.product.poll_async_event()
    }

    pub fn poll_application_event(&mut self) -> Option<AppEvent> {
        self.poll_native_event()
            .or_else(|| self.poll_product_event())
    }

    pub fn route_ai_invocation(
        &mut self,
        invocation: ProductAiInvocation,
    ) -> ZsuiResult<ProductAiResult> {
        let plan = self.product.plan_ai_invocation(invocation)?;
        self.product.execute_ai_plan(plan)
    }

    pub fn request_shutdown(&mut self) {
        self.product.request_shutdown();
        self.driver.request_shutdown();
        self.started = false;
    }

    pub fn run_smoke(
        &mut self,
        request: ProductAdapterRuntimeSmokeRequest,
    ) -> ProductAdapterRuntimeSmokeReport {
        let mut exercised_stage_names = Vec::new();
        let mut errors = Vec::new();
        let product = self.product.identity();
        let projection = self.product.project_ui();
        let ai_capabilities = self.product.product_ai_capabilities();
        push_smoke_stage(
            &mut exercised_stage_names,
            ZsuiReusableRuntimeHarnessStage::ProjectUi,
        );

        let mut started = false;
        let mut handles_created = false;
        match self.start() {
            Ok(_) => {
                started = true;
                handles_created = self.handles().is_some();
                push_smoke_stage(
                    &mut exercised_stage_names,
                    ZsuiReusableRuntimeHarnessStage::StartNativeRuntime,
                );
            }
            Err(err) => errors.push(err.to_string()),
        }

        let command_id = ui_command_id_name(&request.command);
        let mut command_event_count = 0;
        if started {
            match self.dispatch_ui_command(request.command) {
                Ok(events) => {
                    command_event_count = events.len();
                    push_smoke_stage(
                        &mut exercised_stage_names,
                        ZsuiReusableRuntimeHarnessStage::DispatchUiCommand,
                    );
                }
                Err(err) => errors.push(err.to_string()),
            }
        }

        let native_event_polled = self.poll_native_event();
        push_smoke_stage(
            &mut exercised_stage_names,
            ZsuiReusableRuntimeHarnessStage::PollNativeEvent,
        );
        let product_event_polled = self.poll_product_event();
        push_smoke_stage(
            &mut exercised_stage_names,
            ZsuiReusableRuntimeHarnessStage::PollProductEvent,
        );

        let mut ai_invocation_id = None;
        let mut ai_capability_id = None;
        let mut ai_result_kind = None;
        let mut ai_output_present = false;
        let mut ai_emitted_event_count = 0;
        if let Some(invocation) = request.ai_invocation {
            ai_invocation_id = Some(invocation.invocation_id.clone());
            ai_capability_id = Some(invocation.capability_id.clone());
            match self.product.plan_ai_invocation(invocation) {
                Ok(plan) => {
                    push_smoke_stage(
                        &mut exercised_stage_names,
                        ZsuiReusableRuntimeHarnessStage::PlanAiInvocation,
                    );
                    match self.product.execute_ai_plan(plan) {
                        Ok(result) => {
                            ai_result_kind = Some(result.result_kind);
                            ai_output_present = result.output_text.is_some();
                            ai_emitted_event_count = result.emitted_events.len();
                            push_smoke_stage(
                                &mut exercised_stage_names,
                                ZsuiReusableRuntimeHarnessStage::ExecuteAiPlan,
                            );
                        }
                        Err(err) => errors.push(err.to_string()),
                    }
                }
                Err(err) => errors.push(err.to_string()),
            }
        }

        self.request_shutdown();
        push_smoke_stage(
            &mut exercised_stage_names,
            ZsuiReusableRuntimeHarnessStage::RequestShutdown,
        );
        let started_after_shutdown = self.started();
        let missing_stage_names = missing_smoke_stage_names(&exercised_stage_names);
        let harness_smoke_complete = errors.is_empty()
            && started
            && handles_created
            && !started_after_shutdown
            && missing_stage_names.is_empty();
        let status_item_declared = projection.status_item_declared();

        ProductAdapterRuntimeSmokeReport {
            product,
            app_name: projection.app_name,
            main_window_title: projection.main_window.title,
            status_item_declared,
            settings_page_count: projection.settings_pages.len(),
            ai_capability_count: ai_capabilities.len(),
            started,
            handles_created,
            command_id,
            command_event_count,
            native_event_polled,
            product_event_polled,
            ai_invocation_id,
            ai_capability_id,
            ai_result_kind,
            ai_output_present,
            ai_emitted_event_count,
            shutdown_requested: true,
            started_after_shutdown,
            exercised_stage_names,
            missing_stage_names,
            errors,
            harness_smoke_complete,
        }
    }

    pub fn run_view_smoke<Msg>(
        &mut self,
        request: ProductViewRuntimeSmokeRequest,
    ) -> ProductViewRuntimeSmokeReport
    where
        Product: ProductViewAdapterHost<Msg>,
        Msg: Clone,
    {
        let product = self.product.identity();
        let mut errors = Vec::new();
        let mut view = self.product.project_view();
        let mut layout_cx = ViewLayoutCx::new(request.bounds, request.dpi);
        let layout = view.layout(&mut layout_cx);

        let mut view_event_cx = ViewEventCx::new();
        for event in &request.events {
            view.event(&mut view_event_cx, event);
        }
        let messages = view_event_cx.into_messages();

        let mut app_cx = AppCx::new();
        let mut product_event_count = 0;
        for message in messages.iter().cloned() {
            match self.product.update_view_message(message, &mut app_cx) {
                Ok(events) => product_event_count += events.len(),
                Err(err) => errors.push(err.to_string()),
            }
        }

        let ui_commands: Vec<_> = app_cx.ui_commands().to_vec();
        let mut dispatched_ui_command_count = 0;
        for command in &ui_commands {
            match self.dispatch_ui_command(command.clone()) {
                Ok(_) => dispatched_ui_command_count += 1,
                Err(err) => errors.push(err.to_string()),
            }
        }

        let mut paint_cx = ViewPaintCx::new(request.dpi);
        view.paint(&mut paint_cx);
        let draw_plan: NativeDrawPlan = paint_cx.into_plan();
        let view_projected = true;
        let view_smoke_complete = errors.is_empty()
            && view_projected
            && layout.children.len() > 0
            && request.events.len() == messages.len()
            && ui_commands.len() == dispatched_ui_command_count;

        ProductViewRuntimeSmokeReport {
            product,
            view_projected,
            layout_node_count: layout.children.len(),
            view_event_count: request.events.len(),
            view_message_count: messages.len(),
            app_command_count: app_cx.commands().len(),
            ui_command_count: ui_commands.len(),
            dispatched_ui_command_count,
            product_event_count,
            draw_command_count: draw_plan.command_count(),
            text_command_count: draw_plan.text_count(),
            errors,
            view_smoke_complete,
        }
    }
}

pub fn ui_command_id_name(command: &UiCommand) -> &'static str {
    command.id.0
}

pub const PRODUCT_ADAPTER_SMOKE_COMMAND: CommandId = CommandId("zsui.product_adapter.smoke");

fn push_smoke_stage(
    exercised_stage_names: &mut Vec<&'static str>,
    stage: ZsuiReusableRuntimeHarnessStage,
) {
    let stage_name = stage.stage_name();
    if !exercised_stage_names.contains(&stage_name) {
        exercised_stage_names.push(stage_name);
    }
}

fn missing_smoke_stage_names(exercised_stage_names: &[&'static str]) -> Vec<&'static str> {
    ZSUI_REUSABLE_RUNTIME_HARNESS_STAGES
        .iter()
        .map(|stage| stage.stage_name())
        .filter(|stage_name| !exercised_stage_names.contains(stage_name))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, PlatformName, SettingsItemSpec, Size, TraySpec};

    #[derive(Default)]
    struct RecordingDriver {
        start_requests: Vec<NativeRuntimeStartupRequest>,
        commands: Vec<UiCommand>,
        events: Vec<AppEvent>,
        shutdown: bool,
    }

    impl NativeRuntimeDriver<UiCommand, AppEvent> for RecordingDriver {
        type WindowHandle = u32;

        fn start_runtime(
            &mut self,
            request: NativeRuntimeStartupRequest,
        ) -> NativeRuntimeStartupResult<Self::WindowHandle> {
            self.start_requests.push(request);
            NativeRuntimeStartupResult::Started(NativeMainWindowHandles { main: 1, quick: 2 })
        }

        fn dispatch_ui_command(&mut self, command: UiCommand) {
            self.commands.push(command);
        }

        fn poll_application_event(&mut self) -> Option<AppEvent> {
            self.events.pop()
        }

        fn request_shutdown(&mut self) {
            self.shutdown = true;
        }
    }

    struct DemoProduct {
        command_count: usize,
        async_event_sent: bool,
        shutdown: bool,
    }

    impl DemoProduct {
        fn new() -> Self {
            Self {
                command_count: 0,
                async_event_sent: false,
                shutdown: false,
            }
        }
    }

    impl ProductAdapterHost for DemoProduct {
        fn identity(&self) -> ProductAdapterIdentity {
            ProductAdapterIdentity::new("demo", "Demo", "0.1.0")
        }

        fn project_ui(&self) -> ProductUiProjection {
            ProductUiProjection::new("Demo", WindowSpec::new("Demo").size(640, 420))
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
            self.command_count += 1;
            Ok(vec![AppEvent::Custom {
                id: ui_command_id_name(&command).to_string(),
                payload: None,
            }])
        }

        fn poll_async_event(&mut self) -> Option<AppEvent> {
            if self.async_event_sent {
                return None;
            }
            self.async_event_sent = true;
            Some(AppEvent::Custom {
                id: "product.async".to_string(),
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
            Ok(
                ProductAiResult::text(plan.invocation.invocation_id, plan.result_kind, "summary")
                    .event(AppEvent::Custom {
                        id: "ai.completed".to_string(),
                        payload: Some(plan.route_name),
                    }),
            )
        }

        fn request_shutdown(&mut self) {
            self.shutdown = true;
        }
    }

    #[cfg(all(feature = "button", feature = "label"))]
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum DemoViewMsg {
        SaveClicked,
    }

    #[cfg(all(feature = "button", feature = "label"))]
    impl ProductViewAdapterHost<DemoViewMsg> for DemoProduct {
        fn project_view(&self) -> ViewNode<DemoViewMsg> {
            crate::column(vec![
                crate::text("Demo"),
                crate::button("Save")
                    .id(crate::WidgetId::new(11))
                    .on_click(DemoViewMsg::SaveClicked),
            ])
        }

        fn update_view_message(
            &mut self,
            message: DemoViewMsg,
            cx: &mut AppCx,
        ) -> ZsuiResult<Vec<AppEvent>> {
            match message {
                DemoViewMsg::SaveClicked => {
                    cx.command(Command::custom("demo.view.save"));
                    cx.ui_command(UiCommand::app(PRODUCT_ADAPTER_SMOKE_COMMAND));
                    Ok(vec![AppEvent::Custom {
                        id: "demo.view.save".to_string(),
                        payload: None,
                    }])
                }
            }
        }
    }

    #[test]
    fn runtime_harness_starts_driver_from_product_projection() {
        let mut harness = ZsuiReusableRuntimeHarness::new(
            RecordingDriver::default(),
            DemoProduct::new(),
            HostCapabilities::all_supported(PlatformName::Unknown),
        );

        let handles = harness.start().expect("runtime should start");

        assert_eq!(handles.main, 1);
        assert!(harness.started());
        assert_eq!(harness.driver().start_requests.len(), 1);
        assert_eq!(harness.driver().start_requests[0].app_name, "Demo");
        assert!(harness.driver().start_requests[0].status_item.is_some());
        assert_eq!(
            harness.driver().start_requests[0]
                .status_item
                .as_ref()
                .expect("status item should be passed to driver")
                .menu
                .items
                .len(),
            2
        );
        assert_eq!(
            harness.driver().start_requests[0].settings_pages[0].id,
            "general"
        );
        assert_eq!(
            harness.driver().start_requests[0].main_window.size,
            Size {
                width: 640,
                height: 420
            }
        );
    }

    #[test]
    fn runtime_harness_dispatches_commands_through_product_and_driver() {
        let mut harness = ZsuiReusableRuntimeHarness::new(
            RecordingDriver::default(),
            DemoProduct::new(),
            HostCapabilities::all_supported(PlatformName::Unknown),
        );

        let events = harness
            .dispatch_ui_command(UiCommand::app(PRODUCT_ADAPTER_SMOKE_COMMAND))
            .expect("command should dispatch");

        assert_eq!(events.len(), 1);
        assert_eq!(harness.product().command_count, 1);
        assert_eq!(
            harness.driver().commands[0].id,
            PRODUCT_ADAPTER_SMOKE_COMMAND
        );
    }

    #[test]
    fn runtime_harness_routes_ai_invocation_through_product_boundary() {
        let mut harness = ZsuiReusableRuntimeHarness::new(
            RecordingDriver::default(),
            DemoProduct::new(),
            HostCapabilities::all_supported(PlatformName::Unknown),
        );

        let result = harness
            .route_ai_invocation(
                ProductAiInvocation::new("demo.summarize")
                    .invocation_id("run-1")
                    .source_surface("main_window")
                    .input_text("hello world"),
            )
            .expect("AI invocation should route through product");

        assert_eq!(result.invocation_id, "run-1");
        assert_eq!(result.output_text.as_deref(), Some("summary"));
        assert_eq!(result.emitted_events.len(), 1);
    }

    #[test]
    fn runtime_harness_shutdown_calls_both_sides() {
        let mut harness = ZsuiReusableRuntimeHarness::new(
            RecordingDriver::default(),
            DemoProduct::new(),
            HostCapabilities::all_supported(PlatformName::Unknown),
        );

        harness.request_shutdown();

        assert!(harness.driver().shutdown);
        assert!(harness.product().shutdown);
        assert!(!harness.started());
    }

    #[test]
    fn product_adapter_reuse_checklist_exposes_required_surfaces() {
        let checklist = product_adapter_reuse_checklist();

        assert!(checklist.surface_names.contains(&"identity"));
        assert!(checklist.task_names.contains(&"execute_ai_plan"));
        assert!(checklist.ai_provider_family_names.contains(&"llm"));
        assert!(checklist
            .ai_executor_boundary_names
            .contains(&"skill_registry"));
        assert_eq!(zsui_reusable_runtime_harness_stage_names().len(), 8);
    }

    #[test]
    fn runtime_harness_smoke_report_exercises_all_reusable_stages() {
        let mut harness = ZsuiReusableRuntimeHarness::new(
            RecordingDriver {
                events: vec![AppEvent::Started],
                ..RecordingDriver::default()
            },
            DemoProduct::new(),
            HostCapabilities::all_supported(PlatformName::Unknown),
        );
        let request = ProductAdapterRuntimeSmokeRequest::quick().ai_invocation(
            ProductAiInvocation::new("demo.summarize")
                .invocation_id("smoke")
                .source_surface("main_window")
                .input_text("selected text"),
        );

        let report = harness.run_smoke(request);

        assert!(report.harness_smoke_complete);
        assert_eq!(report.product.product_id, "demo");
        assert!(report.status_item_declared);
        assert_eq!(report.settings_page_count, 1);
        assert_eq!(report.command_id, PRODUCT_ADAPTER_SMOKE_COMMAND.0);
        assert_eq!(report.command_event_count, 1);
        assert_eq!(report.native_event_polled, Some(AppEvent::Started));
        assert_eq!(
            report.product_event_polled,
            Some(AppEvent::Custom {
                id: "product.async".to_string(),
                payload: None
            })
        );
        assert_eq!(report.ai_result_kind.as_deref(), Some("text"));
        assert!(report.ai_output_present);
        assert_eq!(report.ai_emitted_event_count, 1);
        assert!(report.missing_stage_names.is_empty());
        assert!(report.errors.is_empty());
    }

    #[cfg(all(feature = "button", feature = "label"))]
    #[test]
    fn runtime_harness_routes_typed_view_messages_through_product_adapter() {
        let mut harness = ZsuiReusableRuntimeHarness::new(
            RecordingDriver::default(),
            DemoProduct::new(),
            HostCapabilities::all_supported(PlatformName::Unknown),
        );
        let report = harness.run_view_smoke(
            ProductViewRuntimeSmokeRequest::new(crate::Rect {
                x: 0,
                y: 0,
                width: 240,
                height: 80,
            })
            .event(crate::ViewEvent::Click {
                widget: crate::WidgetId::new(11),
            }),
        );

        assert!(report.view_smoke_complete);
        assert_eq!(report.view_event_count, 1);
        assert_eq!(report.view_message_count, 1);
        assert_eq!(report.app_command_count, 1);
        assert_eq!(report.ui_command_count, 1);
        assert_eq!(report.dispatched_ui_command_count, 1);
        assert_eq!(report.product_event_count, 1);
        assert_eq!(report.text_command_count, 2);
        assert!(report.draw_command_count >= 3);
        assert_eq!(harness.product().command_count, 1);
        assert_eq!(
            harness.driver().commands[0].id,
            PRODUCT_ADAPTER_SMOKE_COMMAND
        );
    }
}
