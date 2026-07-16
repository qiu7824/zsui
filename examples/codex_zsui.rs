#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::{
    env,
    fs::{self, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use serde::Deserialize;
use zsui::{
    native_window, Dpi, NativeWindowSmokeRunOptions, Rect, ZsWorkbenchActionSpec,
    ZsWorkbenchComposerSpec, ZsWorkbenchContentBlock, ZsWorkbenchConversationGroupSpec,
    ZsWorkbenchConversationSpec, ZsWorkbenchIcon, ZsWorkbenchInspectorSpec, ZsWorkbenchMessageRole,
    ZsWorkbenchMessageSpec, ZsWorkbenchNoticeLevel, ZsWorkbenchSidebarSpec, ZsWorkbenchSpec,
    ZsWorkbenchToolStatus,
};

const SURFACE: Rect = Rect {
    x: 0,
    y: 0,
    width: 1440,
    height: 900,
};
const MAX_VISIBLE_SESSIONS: usize = 10;

#[derive(Debug, Clone, Deserialize)]
struct CodexSessionIndexEntry {
    id: String,
    thread_name: String,
    updated_at: String,
}

#[derive(Debug)]
struct CodexData {
    home: PathBuf,
    sessions: Vec<CodexSessionIndexEntry>,
    malformed_lines: usize,
}

impl CodexData {
    fn latest(&self) -> Option<&CodexSessionIndexEntry> {
        self.sessions.first()
    }

    fn is_live(&self) -> bool {
        !self.sessions.is_empty()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let codex_home = codex_home_from_args(&args).unwrap_or_else(default_codex_home);
    let data = load_codex_data(&codex_home)?;
    let workbench = codex_workbench(&data);
    let layout = workbench.layout(SURFACE, Dpi::standard());
    let builder = native_window("Codex ZSUI")
        .size(SURFACE.width as u32, SURFACE.height as u32)
        .min_size(900, 640)
        .workbench(workbench.clone());

    if args.iter().any(|arg| arg == "--smoke") {
        let artifact_dir = "target/codex-zsui";
        fs::create_dir_all(artifact_dir)?;
        let report = builder.run_smoke(
            NativeWindowSmokeRunOptions::new(1600)
                .screenshot_file(format!("{artifact_dir}/window.png"))
                .require_screenshot(cfg!(windows)),
        )?;
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    if args.iter().any(|arg| arg == "--manifest") {
        let draw_plan = builder
            .native_draw_plan()
            .expect("workbench builder should carry a draw plan");
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "component": "codex_zsui",
                "codex_home": data.home,
                "data_source": data.home.join("session_index.jsonl"),
                "draw_command_count": draw_plan.command_count(),
                "inspector_visible": layout.metrics.inspector.is_some(),
                "live_data": data.is_live(),
                "malformed_lines": data.malformed_lines,
                "message_count": layout.messages.len(),
                "region_count": layout.regions.len(),
                "session_count": data.sessions.len(),
                "text_command_count": draw_plan.text_count(),
                "title": workbench.title,
            }))?
        );
        return Ok(());
    }

    builder.run()?;
    Ok(())
}

fn codex_home_from_args(args: &[String]) -> Option<PathBuf> {
    args.windows(2)
        .find(|pair| pair[0] == "--codex-home")
        .map(|pair| PathBuf::from(&pair[1]))
}

fn default_codex_home() -> PathBuf {
    env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(|home| PathBuf::from(home).join(".codex")))
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".codex")))
        .unwrap_or_else(|| PathBuf::from(".codex"))
}

fn load_codex_data(home: &Path) -> Result<CodexData, Box<dyn std::error::Error>> {
    let index_path = home.join("session_index.jsonl");
    if !index_path.is_file() {
        return Ok(CodexData {
            home: home.to_path_buf(),
            sessions: Vec::new(),
            malformed_lines: 0,
        });
    }

    let mut sessions = Vec::new();
    let mut malformed_lines = 0;
    for line in BufReader::new(File::open(index_path)?).lines() {
        match serde_json::from_str::<CodexSessionIndexEntry>(&line?) {
            Ok(entry) if !entry.id.is_empty() && !entry.thread_name.trim().is_empty() => {
                sessions.push(entry);
            }
            Ok(_) | Err(_) => malformed_lines += 1,
        }
    }
    sessions.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    sessions.dedup_by(|left, right| left.id == right.id);

    Ok(CodexData {
        home: home.to_path_buf(),
        sessions,
        malformed_lines,
    })
}

fn codex_workbench(data: &CodexData) -> ZsWorkbenchSpec {
    let selected_id = data.latest().map(|session| session.id.as_str());
    let recent_sessions = data.sessions.iter().take(MAX_VISIBLE_SESSIONS);
    let mut recent_group = ZsWorkbenchConversationGroupSpec::new("recent", "最近任务");
    for session in recent_sessions {
        recent_group = recent_group.conversation(
            ZsWorkbenchConversationSpec::new(session.id.clone(), session.thread_name.clone())
                .subtitle(format_timestamp(&session.updated_at))
                .selected(selected_id == Some(session.id.as_str())),
        );
    }
    if data.sessions.is_empty() {
        recent_group = recent_group.conversation(
            ZsWorkbenchConversationSpec::new("empty", "未找到本机任务索引")
                .subtitle("可通过 --codex-home 指定目录")
                .selected(true),
        );
    }

    let sidebar = ZsWorkbenchSidebarSpec::new("Codex")
        .primary_action(ZsWorkbenchActionSpec::new(
            "new-task",
            "新建任务",
            ZsWorkbenchIcon::Add,
        ))
        .primary_action(ZsWorkbenchActionSpec::new(
            "search",
            "搜索",
            ZsWorkbenchIcon::Search,
        ))
        .group(recent_group)
        .footer_action(ZsWorkbenchActionSpec::new(
            "settings",
            "设置",
            ZsWorkbenchIcon::Settings,
        ));

    let composer = ZsWorkbenchComposerSpec::new("要求后续变更")
        .mode("本地")
        .model("ZSUI Native")
        .action(ZsWorkbenchActionSpec::new(
            "attach",
            "",
            ZsWorkbenchIcon::Attach,
        ))
        .action(ZsWorkbenchActionSpec::new("mode", "本地", ZsWorkbenchIcon::Tool).selected(true));

    let latest_title = data
        .latest()
        .map(|session| session.thread_name.as_str())
        .unwrap_or("Codex ZSUI");
    let latest_time = data
        .latest()
        .map(|session| format_timestamp(&session.updated_at))
        .unwrap_or_else(|| "本机数据未连接".to_string());
    let source_path = data.home.join("session_index.jsonl");

    let mut workbench = ZsWorkbenchSpec::new(latest_title, sidebar, composer)
        .subtitle(format!("本机 Codex · {latest_time}"))
        .toolbar_action(ZsWorkbenchActionSpec::new(
            "open-location",
            "打开位置",
            ZsWorkbenchIcon::Folder,
        ))
        .toolbar_action(ZsWorkbenchActionSpec::new(
            "more",
            "更多",
            ZsWorkbenchIcon::More,
        ))
        .message(
            ZsWorkbenchMessageSpec::new("current-task", ZsWorkbenchMessageRole::User)
                .block(ZsWorkbenchContentBlock::paragraph(latest_title)),
        );

    if data.is_live() {
        workbench = workbench.message(
            ZsWorkbenchMessageSpec::new("local-index", ZsWorkbenchMessageRole::Assistant)
                .block(ZsWorkbenchContentBlock::paragraph(
                    "本机任务索引可用。侧栏显示最近任务，界面状态由应用数据显式构建。",
                ))
                .block(ZsWorkbenchContentBlock::tool(
                    "读取任务索引",
                    format!("{} 个任务 · 只读", data.sessions.len()),
                    ZsWorkbenchToolStatus::Succeeded,
                ))
                .block(ZsWorkbenchContentBlock::notice(
                    "未读取认证信息、对话正文或附件。",
                    ZsWorkbenchNoticeLevel::Info,
                )),
        );
    } else {
        workbench = workbench.message(
            ZsWorkbenchMessageSpec::new("local-index", ZsWorkbenchMessageRole::Assistant).block(
                ZsWorkbenchContentBlock::notice(
                    format!("未找到 {}", source_path.display()),
                    ZsWorkbenchNoticeLevel::Warning,
                ),
            ),
        );
    }

    workbench.inspector(
        ZsWorkbenchInspectorSpec::new("环境信息")
            .selected_tab("local")
            .tab(ZsWorkbenchActionSpec::new(
                "local",
                "本地",
                ZsWorkbenchIcon::App,
            ))
            .tab(ZsWorkbenchActionSpec::new(
                "data",
                "数据",
                ZsWorkbenchIcon::Code,
            ))
            .body(format!(
                "数据源\n{}\n\n任务数\n{}\n\n最近更新\n{}\n\n访问模式\n只读",
                source_path.display(),
                data.sessions.len(),
                latest_time,
            )),
    )
}

fn format_timestamp(value: &str) -> String {
    let value = value.trim();
    if value.len() >= 16 && value.as_bytes().get(10) == Some(&b'T') {
        format!("{} {}", &value[0..10], &value[11..16])
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn reads_and_orders_codex_session_index() {
        let root = env::temp_dir().join(format!("zsui-codex-demo-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let mut index = File::create(root.join("session_index.jsonl")).unwrap();
        writeln!(
            index,
            r#"{{"id":"older","thread_name":"Older","updated_at":"2026-07-14T08:00:00Z"}}"#
        )
        .unwrap();
        writeln!(index, "not-json").unwrap();
        writeln!(
            index,
            r#"{{"id":"newer","thread_name":"Newer","updated_at":"2026-07-15T09:30:00Z"}}"#
        )
        .unwrap();

        let data = load_codex_data(&root).unwrap();
        assert_eq!(data.sessions.len(), 2);
        assert_eq!(data.sessions[0].id, "newer");
        assert_eq!(data.malformed_lines, 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn builds_offline_shell_when_index_is_missing() {
        let data = CodexData {
            home: PathBuf::from("missing"),
            sessions: Vec::new(),
            malformed_lines: 0,
        };
        let spec = codex_workbench(&data);
        assert_eq!(spec.title, "Codex ZSUI");
        assert_eq!(spec.messages.len(), 2);
    }

    #[test]
    fn timestamp_is_compact_for_sidebar_rows() {
        assert_eq!(
            format_timestamp("2026-07-15T09:31:03.8333473Z"),
            "2026-07-15 09:31"
        );
    }
}
