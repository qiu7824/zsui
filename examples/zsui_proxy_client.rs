#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

//! VLESS proxy client UI demo.
//!
//! The demo parses `vless://` share links and simulates traffic so window
//! memory, repaint and flicker behavior can be measured on a realistic
//! client layout. It never opens a network connection. Links are imported at
//! runtime (`--import`, `ZSUI_PROXY_DEMO_LINK` or the text box) and are never
//! written to disk.

use std::{
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use zsui::{
    button, column, list, native_window, row, section, styled_text, text, textbox, toggle, AppCx,
    ColorRole, Dp, NativeWindowSmokeRunOptions, SemanticTextStyle, TextRole, UiInvalidationHandle,
    ViewNode, ZsuiError, ZsuiResult,
};

const WINDOW_TITLE: &str = "ZSUI 代理演示";
const LOG_CAPACITY: usize = 3;

/// Synthetic node used by measurements. The address is in TEST-NET-3 and the
/// credentials are placeholders, so screenshots never expose a real server.
const DEMO_LINK: &str = "vless://8f1c2d3e-4b5a-4c6d-8e7f-0a1b2c3d4e5f@203.0.113.10:443\
?encryption=none&flow=xtls-rprx-vision&security=reality&sni=www.example.com&fp=chrome\
&pbk=DemoPublicKeyForLayoutOnly0000000000000000&sid=0123456789abcdef&spx=%2F&type=tcp\
&headerType=none#%E6%BC%94%E7%A4%BA%E8%8A%82%E7%82%B9-Tokyo";

#[derive(Debug, Clone, PartialEq, Eq)]
struct VlessNode {
    name: String,
    user_id: String,
    host: String,
    port: u16,
    encryption: String,
    flow: Option<String>,
    security: Option<String>,
    sni: Option<String>,
    fingerprint: Option<String>,
    public_key: Option<String>,
    short_id: Option<String>,
    spider_x: Option<String>,
    network: String,
    header_type: Option<String>,
}

impl VlessNode {
    fn address(&self) -> String {
        if self.host.contains(':') {
            format!("[{}]:{}", self.host, self.port)
        } else {
            format!("{}:{}", self.host, self.port)
        }
    }

    fn same_endpoint(&self, other: &Self) -> bool {
        self.host == other.host && self.port == other.port && self.user_id == other.user_id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinkError {
    Scheme,
    MissingUserId,
    MissingHost,
    InvalidPort,
    InvalidEscape,
}

impl LinkError {
    const fn message(self) -> &'static str {
        match self {
            Self::Scheme => "不是 vless:// 链接",
            Self::MissingUserId => "链接缺少用户 ID",
            Self::MissingHost => "链接缺少服务器地址",
            Self::InvalidPort => "端口无效",
            Self::InvalidEscape => "链接中的百分号编码无效",
        }
    }
}

fn parse_vless_link(link: &str) -> Result<VlessNode, LinkError> {
    let link = link.trim();
    let rest = link
        .get(..8)
        .filter(|scheme| scheme.eq_ignore_ascii_case("vless://"))
        .map(|_| &link[8..])
        .ok_or(LinkError::Scheme)?;
    let (rest, fragment) = rest.split_once('#').unwrap_or((rest, ""));
    let (authority, query) = rest.split_once('?').unwrap_or((rest, ""));
    let authority = authority.trim_end_matches('/');
    let (user_id, endpoint) = authority.rsplit_once('@').ok_or(LinkError::MissingUserId)?;
    let user_id = percent_decode(user_id)?;
    if user_id.is_empty() {
        return Err(LinkError::MissingUserId);
    }
    let (host, port) = if let Some(bracketed) = endpoint.strip_prefix('[') {
        let (host, port) = bracketed.split_once("]:").ok_or(LinkError::InvalidPort)?;
        (host, port)
    } else {
        endpoint.rsplit_once(':').ok_or(LinkError::InvalidPort)?
    };
    if host.is_empty() {
        return Err(LinkError::MissingHost);
    }
    let port = port
        .parse::<u16>()
        .ok()
        .filter(|port| *port != 0)
        .ok_or(LinkError::InvalidPort)?;

    let mut node = VlessNode {
        name: percent_decode(fragment)?,
        user_id,
        host: host.to_string(),
        port,
        encryption: "none".to_string(),
        flow: None,
        security: None,
        sni: None,
        fingerprint: None,
        public_key: None,
        short_id: None,
        spider_x: None,
        network: "tcp".to_string(),
        header_type: None,
    };
    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        let Some(value) = Some(percent_decode(value)?).filter(|value| !value.is_empty()) else {
            continue;
        };
        match key {
            "encryption" => node.encryption = value,
            "flow" => node.flow = Some(value),
            "security" => node.security = Some(value),
            "sni" => node.sni = Some(value),
            "fp" => node.fingerprint = Some(value),
            "pbk" => node.public_key = Some(value),
            "sid" => node.short_id = Some(value),
            "spx" => node.spider_x = Some(value),
            "type" => node.network = value,
            "headerType" => node.header_type = Some(value),
            _ => {}
        }
    }
    if node.name.is_empty() {
        node.name = node.address();
    }
    Ok(node)
}

fn percent_decode(value: &str) -> Result<String, LinkError> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let digits = bytes
                .get(index + 1..index + 3)
                .and_then(|digits| std::str::from_utf8(digits).ok())
                .and_then(|digits| u8::from_str_radix(digits, 16).ok())
                .ok_or(LinkError::InvalidEscape)?;
            decoded.push(digits);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).map_err(|_| LinkError::InvalidEscape)
}

/// Shows enough of a credential to tell nodes apart without displaying it.
fn mask(value: &str, head: usize, tail: usize) -> String {
    let count = value.chars().count();
    if count <= head + tail {
        return "•".repeat(count.max(1));
    }
    let start = value.chars().take(head).collect::<String>();
    let end = value.chars().skip(count - tail).collect::<String>();
    format!("{start}…{end}")
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs();
    format!(
        "{:02}:{:02}:{:02}",
        seconds / 3600,
        seconds / 60 % 60,
        seconds % 60
    )
}

/// Counters written by the simulated transport thread and read by `view`.
#[derive(Debug, Default)]
struct Traffic {
    up_total: AtomicU64,
    down_total: AtomicU64,
    up_rate: AtomicU64,
    down_rate: AtomicU64,
    ticks: AtomicU64,
    /// Transport time advanced by each tick, used by scripted runs.
    simulated_ms: AtomicU64,
}

impl Traffic {
    fn reset(&self) {
        for counter in [
            &self.up_total,
            &self.down_total,
            &self.up_rate,
            &self.down_rate,
            &self.simulated_ms,
        ] {
            counter.store(0, Ordering::Relaxed);
        }
    }
}

/// Simulated transport. It wakes the UI through the invalidation handle
/// while connected and exits as soon as the connection is switched off, or
/// after `limit` ticks so scripted runs end on deterministic values.
fn start_traffic(
    traffic: Arc<Traffic>,
    invalidation: UiInvalidationHandle,
    tick: Duration,
    limit: Option<u64>,
) -> Arc<AtomicBool> {
    let stop = Arc::new(AtomicBool::new(false));
    let worker_stop = stop.clone();
    thread::spawn(move || {
        let mut seed = 0x9e37_79b9_7f4a_7c15_u64;
        let tick_ms = tick.as_millis().max(1) as u64;
        for _ in 0..limit.unwrap_or(u64::MAX) {
            thread::sleep(tick);
            if worker_stop.load(Ordering::Acquire) {
                break;
            }
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let down_rate = 320 * 1024 + seed % (2_400 * 1024);
            let up_rate = 24 * 1024 + (seed >> 24) % (180 * 1024);
            traffic.down_rate.store(down_rate, Ordering::Relaxed);
            traffic.up_rate.store(up_rate, Ordering::Relaxed);
            traffic
                .down_total
                .fetch_add(down_rate * tick_ms / 1000, Ordering::Relaxed);
            traffic
                .up_total
                .fetch_add(up_rate * tick_ms / 1000, Ordering::Relaxed);
            traffic.ticks.fetch_add(1, Ordering::Relaxed);
            traffic.simulated_ms.fetch_add(tick_ms, Ordering::Relaxed);
            invalidation.request_rebuild();
        }
    });
    stop
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Notice {
    Empty,
    Imported(String),
    AlreadyImported(String),
    Failed(&'static str),
}

struct ProxyState {
    link_input: String,
    nodes: Vec<VlessNode>,
    selected: Option<usize>,
    notice: Notice,
    connected_since: Option<Instant>,
    transport_stop: Option<Arc<AtomicBool>>,
    traffic: Arc<Traffic>,
    invalidation: UiInvalidationHandle,
    tick: Duration,
    /// Scripted runs stop after this many ticks and use simulated time.
    tick_limit: Option<u64>,
    started: Instant,
    log: Vec<String>,
}

impl ProxyState {
    fn new(invalidation: UiInvalidationHandle, tick: Duration, tick_limit: Option<u64>) -> Self {
        Self {
            link_input: String::new(),
            nodes: Vec::new(),
            selected: None,
            notice: Notice::Empty,
            connected_since: None,
            transport_stop: None,
            traffic: Arc::new(Traffic::default()),
            invalidation,
            tick,
            tick_limit,
            started: Instant::now(),
            log: Vec::new(),
        }
    }

    /// Wall time, or transport time in scripted runs so screenshots of two
    /// builds can be compared pixel for pixel.
    fn clock(&self, since: Instant) -> Duration {
        if self.tick_limit.is_some() {
            Duration::from_millis(self.traffic.simulated_ms.load(Ordering::Relaxed))
        } else {
            since.elapsed()
        }
    }

    fn record(&mut self, event: impl AsRef<str>) {
        let elapsed = self.clock(self.started).as_secs_f32();
        if self.log.len() == LOG_CAPACITY {
            self.log.remove(0);
        }
        self.log.push(format!("+{elapsed:.1}s  {}", event.as_ref()));
    }

    fn import(&mut self, link: &str) {
        match parse_vless_link(link) {
            Ok(node) => {
                if let Some(index) = self
                    .nodes
                    .iter()
                    .position(|known| known.same_endpoint(&node))
                {
                    self.notice = Notice::AlreadyImported(self.nodes[index].name.clone());
                    self.select(index);
                } else {
                    self.record(format!("导入节点 {}", node.name));
                    self.notice = Notice::Imported(node.name.clone());
                    self.nodes.push(node);
                    self.select(self.nodes.len() - 1);
                }
                // The link carries credentials; do not keep it in the field.
                self.link_input.clear();
            }
            Err(error) => self.notice = Notice::Failed(error.message()),
        }
    }

    fn select(&mut self, index: usize) {
        if index >= self.nodes.len() || self.selected == Some(index) {
            return;
        }
        let reconnect = self.connected_since.is_some();
        self.disconnect();
        self.selected = Some(index);
        if reconnect {
            self.connect();
        }
    }

    fn connect(&mut self) {
        let Some(node) = self.selected.and_then(|index| self.nodes.get(index)) else {
            self.notice = Notice::Failed("请先导入并选择一个节点");
            return;
        };
        let name = node.name.clone();
        self.traffic.reset();
        self.transport_stop = Some(start_traffic(
            self.traffic.clone(),
            self.invalidation.clone(),
            self.tick,
            self.tick_limit,
        ));
        self.connected_since = Some(Instant::now());
        self.record(format!("已连接 {name}（模拟）"));
    }

    fn disconnect(&mut self) {
        if let Some(stop) = self.transport_stop.take() {
            stop.store(true, Ordering::Release);
        }
        if self.connected_since.take().is_some() {
            self.traffic.down_rate.store(0, Ordering::Relaxed);
            self.traffic.up_rate.store(0, Ordering::Relaxed);
            self.record("已断开");
        }
    }
}

#[derive(Debug, Clone)]
enum Msg {
    LinkEdited(String),
    Import,
    ImportSubmitted(String),
    Select(usize),
    ConnectChanged(bool),
}

fn update(state: &mut ProxyState, message: Msg, _cx: &mut AppCx) {
    match message {
        Msg::LinkEdited(value) => state.link_input = value,
        Msg::Import => {
            let link = state.link_input.clone();
            state.import(&link);
        }
        Msg::ImportSubmitted(link) => state.import(&link),
        Msg::Select(index) => state.select(index),
        Msg::ConnectChanged(true) => state.connect(),
        Msg::ConnectChanged(false) => state.disconnect(),
    }
}

fn caption<Msg>(value: impl Into<String>) -> ViewNode<Msg> {
    styled_text(
        value,
        SemanticTextStyle {
            color: ColorRole::SecondaryText,
            ..SemanticTextStyle::for_role(TextRole::Caption)
        },
    )
}

fn detail(label: &str, value: impl Into<String>) -> ViewNode<Msg> {
    row([caption(label).width(Dp::new(72.0)), text(value).flex(1.0)]).gap(Dp::new(8.0))
}

fn view(state: &ProxyState) -> ViewNode<Msg> {
    let notice = match &state.notice {
        Notice::Empty => caption("支持 Reality / TLS、xtls-rprx-vision 等常见参数"),
        Notice::Imported(name) => styled_text(
            format!("已导入：{name}"),
            SemanticTextStyle {
                color: ColorRole::Success,
                ..SemanticTextStyle::for_role(TextRole::Caption)
            },
        ),
        Notice::AlreadyImported(name) => caption(format!("节点已存在：{name}")),
        Notice::Failed(reason) => styled_text(
            format!("导入失败：{reason}"),
            SemanticTextStyle {
                color: ColorRole::Danger,
                ..SemanticTextStyle::for_role(TextRole::Caption)
            },
        ),
    };

    let nodes = if state.nodes.is_empty() {
        caption("还没有节点")
    } else {
        list(state.nodes.iter(), |node| text(node.name.clone()))
            .selected_index(state.selected)
            .on_select(Msg::Select)
    };
    let log = if state.log.is_empty() {
        vec![caption("暂无事件")]
    } else {
        state.log.iter().map(|line| caption(line.clone())).collect()
    };

    let selected = state.selected.and_then(|index| state.nodes.get(index));
    let details = match selected {
        Some(node) => {
            let or_none = |value: &Option<String>| value.clone().unwrap_or_else(|| "无".into());
            vec![
                detail("协议", format!("VLESS · 加密 {}", node.encryption)),
                detail("地址", node.address()),
                detail("用户 ID", mask(&node.user_id, 8, 4)),
                detail("流控", or_none(&node.flow)),
                detail(
                    "传输",
                    match &node.header_type {
                        Some(header) => format!("{} · 伪装 {header}", node.network),
                        None => node.network.clone(),
                    },
                ),
                detail(
                    "安全",
                    match (&node.security, &node.fingerprint) {
                        (Some(security), Some(fingerprint)) => {
                            format!("{security} · 指纹 {fingerprint}")
                        }
                        (security, _) => or_none(security),
                    },
                ),
                detail("SNI", or_none(&node.sni)),
                detail(
                    "公钥",
                    node.public_key
                        .as_deref()
                        .map_or_else(|| "无".into(), |key| mask(key, 6, 4)),
                ),
                detail("Short ID", or_none(&node.short_id)),
            ]
        }
        None => vec![caption("导入链接后在这里查看节点参数")],
    };

    let connected = state.connected_since.is_some();
    let traffic = &state.traffic;
    let status = match state.connected_since {
        Some(since) => format!("已连接 · {}", format_duration(state.clock(since))),
        None => "未连接".to_string(),
    };
    let connection = vec![
        row([
            toggle(connected).on_toggle(Msg::ConnectChanged),
            text(status).flex(1.0),
        ])
        .gap(Dp::new(12.0)),
        detail(
            "速率",
            format!(
                "↓ {}/s   ↑ {}/s",
                format_bytes(traffic.down_rate.load(Ordering::Relaxed)),
                format_bytes(traffic.up_rate.load(Ordering::Relaxed))
            ),
        ),
        detail(
            "累计",
            format!(
                "↓ {}   ↑ {}",
                format_bytes(traffic.down_total.load(Ordering::Relaxed)),
                format_bytes(traffic.up_total.load(Ordering::Relaxed))
            ),
        ),
    ];

    column([
        styled_text(WINDOW_TITLE, SemanticTextStyle::for_role(TextRole::Title)),
        caption("演示模式：只解析 VLESS 分享链接并模拟流量，不建立真实代理连接。"),
        row([
            textbox(state.link_input.clone())
                .placeholder("粘贴 vless:// 链接，按 Enter 或点“导入”")
                .on_change(Msg::LinkEdited)
                .on_submit(Msg::ImportSubmitted)
                .flex(1.0),
            button("导入")
                .enabled(!state.link_input.trim().is_empty())
                .on_click(Msg::Import),
        ])
        .gap(Dp::new(8.0)),
        notice,
        row([
            column([
                section("节点", [nodes]),
                section("连接", connection),
                section("日志", log),
            ])
            .gap(Dp::new(12.0))
            .width(Dp::new(300.0)),
            column([section("节点详情", details)]).flex(1.0),
        ])
        .gap(Dp::new(16.0))
        .flex(1.0),
    ])
    .gap(Dp::new(10.0))
    .padding(Dp::new(20.0))
}

fn arg_value<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].as_str())
}

fn arg_u64(args: &[String], name: &str, default: u64) -> u64 {
    arg_value(args, name)
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn main() -> ZsuiResult<()> {
    let args = std::env::args().collect::<Vec<_>>();
    let flag = |name: &str| args.iter().any(|argument| argument == name);
    let measure = arg_value(&args, "--measure").map(PathBuf::from);
    let width = arg_u64(&args, "--width", 800).clamp(480, 4096) as u32;
    let height = arg_u64(&args, "--height", 680).clamp(400, 4096) as u32;
    let default_tick = if measure.is_some() { 100 } else { 500 };
    let tick = Duration::from_millis(arg_u64(&args, "--tick-ms", default_tick).max(16));

    let invalidation = UiInvalidationHandle::new();
    let tick_limit = arg_value(&args, "--ticks").and_then(|value| value.parse().ok());
    let mut state = ProxyState::new(invalidation.clone(), tick, tick_limit);
    let link = arg_value(&args, "--import")
        .map(str::to_string)
        .or_else(|| std::env::var("ZSUI_PROXY_DEMO_LINK").ok())
        .or_else(|| flag("--demo-node").then(|| DEMO_LINK.to_string()));
    if let Some(link) = link {
        state.import(&link);
    }
    if flag("--connect") {
        state.connect();
    }
    let traffic = state.traffic.clone();

    let builder = native_window(WINDOW_TITLE)
        .size(width, height)
        .min_size(width.min(680), height.min(560))
        .always_on_top(flag("--always-on-top"))
        .invalidation_handle(invalidation)
        .stateful_view(state, view, update);

    let Some(path) = measure else {
        builder.run()?;
        return Ok(());
    };
    let duration_ms = arg_u64(&args, "--duration-ms", 4_000).max(500);
    let mut options = NativeWindowSmokeRunOptions::new(duration_ms);
    if let Some(screenshot) = arg_value(&args, "--screenshot") {
        options = options.screenshot_file(screenshot).require_screenshot(true);
    }
    let report = builder.run_smoke(options)?;
    if !report.visible_window_was_created() {
        return Err(ZsuiError::host(
            "proxy_demo_measure",
            "the native proxy demo window was not created",
        ));
    }
    let memory = report.process_memory_during_runtime.ok_or_else(|| {
        ZsuiError::host(
            "proxy_demo_measure",
            "process memory was not sampled while the native window was alive",
        )
    })?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| ZsuiError::host("create_proxy_demo_dir", error.to_string()))?;
    }
    let document = serde_json::json!({
        "schema": "zsui.ui-memory-comparison/v1",
        "framework": "zsui",
        "scenario": "proxy-client",
        "platform": std::env::consts::OS,
        "architecture": std::env::consts::ARCH,
        "sample_point": "before_teardown",
        "window_width": width,
        "window_height": height,
        "duration_ms": duration_ms,
        "tick_ms": tick.as_millis() as u64,
        "simulated_ticks": traffic.ticks.load(Ordering::Relaxed),
        "source": memory.source,
        "resident_bytes": memory.resident_bytes,
        "peak_resident_bytes": memory.peak_resident_bytes,
        "private_resident_bytes": memory.private_bytes,
        "proportional_set_size_bytes": memory.proportional_set_size_bytes,
        "virtual_bytes": memory.virtual_bytes,
    });
    fs::write(
        path,
        serde_json::to_vec_pretty(&document)
            .map_err(|error| ZsuiError::host("serialize_proxy_demo", error.to_string()))?,
    )
    .map_err(|error| ZsuiError::host("write_proxy_demo", error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_reality_vision_share_links() {
        let node = parse_vless_link(DEMO_LINK).expect("demo link parses");
        assert_eq!(node.name, "演示节点-Tokyo");
        assert_eq!(node.user_id, "8f1c2d3e-4b5a-4c6d-8e7f-0a1b2c3d4e5f");
        assert_eq!(node.address(), "203.0.113.10:443");
        assert_eq!(node.flow.as_deref(), Some("xtls-rprx-vision"));
        assert_eq!(node.security.as_deref(), Some("reality"));
        assert_eq!(node.sni.as_deref(), Some("www.example.com"));
        assert_eq!(node.fingerprint.as_deref(), Some("chrome"));
        assert_eq!(node.short_id.as_deref(), Some("0123456789abcdef"));
        assert_eq!(node.spider_x.as_deref(), Some("/"));
        assert_eq!(node.network, "tcp");
        assert_eq!(node.header_type.as_deref(), Some("none"));
    }

    #[test]
    fn parses_bracketed_ipv6_and_defaults_the_name() {
        let node = parse_vless_link("VLESS://id@[2001:db8::1]:8443").expect("ipv6 parses");
        assert_eq!(node.host, "2001:db8::1");
        assert_eq!(node.port, 8443);
        assert_eq!(node.name, "[2001:db8::1]:8443");
        assert_eq!(node.encryption, "none");
    }

    #[test]
    fn rejects_malformed_links() {
        assert_eq!(parse_vless_link("vmess://abc"), Err(LinkError::Scheme));
        assert_eq!(
            parse_vless_link("vless://example.com:443"),
            Err(LinkError::MissingUserId)
        );
        assert_eq!(
            parse_vless_link("vless://id@example.com:0"),
            Err(LinkError::InvalidPort)
        );
        assert_eq!(
            parse_vless_link("vless://id@example.com:443#%E6%BC"),
            Err(LinkError::InvalidEscape)
        );
    }

    #[test]
    fn masks_credentials_and_formats_counters() {
        assert_eq!(mask("8f1c2d3e-4b5a-4c6d", 8, 4), "8f1c2d3e…4c6d");
        assert_eq!(mask("abc", 2, 2), "•••");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_duration(Duration::from_secs(3_725)), "01:02:05");
    }
}
