use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use std::fs;
use std::io::{self, Stdout};
use std::path::PathBuf;

use crate::PROJECT_DIR;

#[derive(PartialEq, Clone)]
enum Section {
    Projects,
    Assets,
    Cuts,
    Skills,
    Rules,
}

impl Section {
    fn label(&self) -> &str {
        match self {
            Section::Projects => "📋 프로젝트",
            Section::Assets => "🎨 에셋",
            Section::Cuts => "🎬 컷",
            Section::Skills => "📖 스킬",
            Section::Rules => "⚙️  규칙",
        }
    }
    fn all() -> Vec<Section> {
        vec![Section::Projects, Section::Assets, Section::Cuts, Section::Skills, Section::Rules]
    }
}

#[derive(PartialEq)]
enum Focus { Sidebar, Main, Detail }

struct AppState {
    section: Section,
    focus: Focus,
    sidebar_state: ListState,
    main_state: ListState,
    main_items: Vec<(String, String)>, // (id, display)
    detail_text: String,
}

impl AppState {
    fn new() -> Self {
        let mut sidebar_state = ListState::default();
        sidebar_state.select(Some(0));
        let mut s = AppState {
            section: Section::Projects,
            focus: Focus::Sidebar,
            sidebar_state,
            main_state: ListState::default(),
            main_items: vec![],
            detail_text: String::new(),
        };
        s.refresh_main();
        s
    }

    fn refresh_main(&mut self) {
        self.main_items = load_items(&self.section);
        if !self.main_items.is_empty() {
            self.main_state.select(Some(0));
            self.refresh_detail();
        } else {
            self.main_state.select(None);
            self.detail_text = format!("{} 항목이 없습니다.", self.section.label());
        }
    }

    fn refresh_detail(&mut self) {
        let idx = self.main_state.selected().unwrap_or(0);
        if let Some((id, _)) = self.main_items.get(idx) {
            self.detail_text = load_detail(&self.section, id);
        }
    }
}

fn avp_dir() -> PathBuf {
    std::env::current_dir().unwrap().join(PROJECT_DIR)
}

fn current_project() -> Option<String> {
    fs::read_to_string(avp_dir().join("current")).ok().map(|s| s.trim().to_string())
}

fn project_dir(name: &str) -> PathBuf {
    avp_dir().join("projects").join(name)
}

fn skill_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn load_items(section: &Section) -> Vec<(String, String)> {
    match section {
        Section::Projects => {
            let dir = avp_dir().join("projects");
            let current = current_project();
            fs::read_dir(&dir).into_iter().flatten().filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    let marker = if current.as_deref() == Some(&name) { " ◀" } else { "" };
                    (name.clone(), format!("{}{}", name, marker))
                })
                .collect()
        }
        Section::Assets => {
            let Some(proj) = current_project() else { return vec![]; };
            let dir = project_dir(&proj);
            let mut items = vec![];
            for t in ["characters", "monsters", "backgrounds", "keycuts"] {
                let asset_dir = dir.join("assets").join(t);
                for entry in fs::read_dir(&asset_dir).into_iter().flatten().filter_map(|e| e.ok()) {
                    if entry.path().extension().is_some_and(|ext| ext == "json") {
                        let name = entry.path().file_stem().unwrap().to_string_lossy().to_string();
                        let type_label = match t {
                            "characters" => "👤", "monsters" => "👹",
                            "backgrounds" => "🏞 ", "keycuts" => "🎞 ", _ => "  "
                        };
                        items.push((format!("{}/{}", t, name), format!("{} {}", type_label, name)));
                    }
                }
            }
            items
        }
        Section::Cuts => {
            let Some(proj) = current_project() else { return vec![]; };
            let cuts_dir = project_dir(&proj).join("cuts");
            let mut entries: Vec<_> = fs::read_dir(&cuts_dir).into_iter().flatten()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
                .collect();
            entries.sort_by_key(|e| e.file_name());
            entries.into_iter().filter_map(|e| {
                let content = fs::read_to_string(e.path()).ok()?;
                let v: serde_json::Value = serde_json::from_str(&content).ok()?;
                let num = v["number"].as_u64()?;
                let title = v["title"].as_str().unwrap_or("").to_string();
                let stage = v["stage"].as_str().unwrap_or("").to_string();
                let done = v["done"].as_bool().unwrap_or(false);
                let mark = if done { "✅" } else { "⬜" };
                Some((format!("cut_{:03}", num), format!("{} #{:02} {} [{}]", mark, num, title, stage)))
            }).collect()
        }
        Section::Skills => {
            let skill = skill_dir();
            let mut items = vec![("SKILL.md".to_string(), "📄 SKILL.md".to_string())];
            let refs = skill.join("references");
            if let Ok(rd) = fs::read_dir(&refs) {
                for e in rd.filter_map(|e| e.ok()) {
                    if e.path().extension().is_some_and(|ext| ext == "md") {
                        let name = e.file_name().to_string_lossy().to_string();
                        items.push((format!("references/{}", name), format!("📄 references/{}", name)));
                    }
                }
            }
            items
        }
        Section::Rules => {
            let rules = skill_dir().join("rules");
            if let Ok(rd) = fs::read_dir(&rules) {
                rd.filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
                    .map(|e| {
                        let name = e.file_name().to_string_lossy().to_string();
                        (name.clone(), format!("⚙️  {}", name))
                    })
                    .collect()
            } else {
                vec![]
            }
        }
    }
}

fn load_detail(section: &Section, id: &str) -> String {
    match section {
        Section::Projects => {
            let path = project_dir(id).join("project.json");
            fs::read_to_string(&path).unwrap_or_else(|_| "(읽기 실패)".to_string()) // LINT_ALLOW: TUI 표시용 fallback
        }
        Section::Assets => {
            let Some(proj) = current_project() else { return "(프로젝트 없음)".to_string(); };
            let path = project_dir(&proj).join("assets").join(format!("{}.json", id));
            fs::read_to_string(&path).unwrap_or_else(|_| "(읽기 실패)".to_string()) // LINT_ALLOW: TUI 표시용 fallback
        }
        Section::Cuts => {
            let Some(proj) = current_project() else { return "(프로젝트 없음)".to_string(); };
            let path = project_dir(&proj).join("cuts").join(format!("{}.json", id));
            fs::read_to_string(&path).unwrap_or_else(|_| "(읽기 실패)".to_string()) // LINT_ALLOW: TUI 표시용 fallback
        }
        Section::Skills => {
            let path = skill_dir().join(id);
            fs::read_to_string(&path).unwrap_or_else(|_| "(읽기 실패)".to_string()) // LINT_ALLOW: TUI 표시용 fallback
        }
        Section::Rules => {
            let path = skill_dir().join("rules").join(id);
            fs::read_to_string(&path).unwrap_or_else(|_| "(읽기 실패)".to_string()) // LINT_ALLOW: TUI 표시용 fallback
        }
    }
}

pub fn run() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState::new();
    let result = run_app(&mut terminal, &mut state);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, state: &mut AppState) -> io::Result<()> {
    loop {
        terminal.draw(|f| draw(f, state))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press { continue; }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Tab => {
                    state.focus = match state.focus {
                        Focus::Sidebar => Focus::Main,
                        Focus::Main => Focus::Detail,
                        Focus::Detail => Focus::Sidebar,
                    };
                }
                KeyCode::Down | KeyCode::Char('j') => match state.focus {
                    Focus::Sidebar => {
                        let sections = Section::all();
                        let i = state.sidebar_state.selected().unwrap_or(0);
                        let next = (i + 1) % sections.len();
                        state.sidebar_state.select(Some(next));
                        state.section = sections[next].clone();
                        state.refresh_main();
                    }
                    Focus::Main => {
                        if !state.main_items.is_empty() {
                            let i = state.main_state.selected().unwrap_or(0);
                            let next = (i + 1) % state.main_items.len();
                            state.main_state.select(Some(next));
                            state.refresh_detail();
                        }
                    }
                    _ => {}
                },
                KeyCode::Up | KeyCode::Char('k') => match state.focus {
                    Focus::Sidebar => {
                        let sections = Section::all();
                        let i = state.sidebar_state.selected().unwrap_or(0);
                        let prev = if i == 0 { sections.len() - 1 } else { i - 1 };
                        state.sidebar_state.select(Some(prev));
                        state.section = sections[prev].clone();
                        state.refresh_main();
                    }
                    Focus::Main => {
                        if !state.main_items.is_empty() {
                            let i = state.main_state.selected().unwrap_or(0);
                            let prev = if i == 0 { state.main_items.len() - 1 } else { i - 1 };
                            state.main_state.select(Some(prev));
                            state.refresh_detail();
                        }
                    }
                    _ => {}
                },
                KeyCode::Char('r') => state.refresh_main(),
                _ => {}
            }
        }
    }
}

fn draw(f: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(f.area());

    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Percentage(40), Constraint::Min(30)])
        .split(chunks[0]);

    // 사이드바
    let sections = Section::all();
    let sidebar_items: Vec<ListItem> = sections.iter()
        .map(|s| ListItem::new(s.label())).collect();
    let sidebar_block = Block::default()
        .title("avp TUI")
        .borders(Borders::ALL)
        .border_style(if state.focus == Focus::Sidebar { Style::default().fg(Color::Cyan) } else { Style::default() });
    let sidebar = List::new(sidebar_items)
        .block(sidebar_block)
        .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .highlight_symbol("▶ ");
    let mut sidebar_state = state.sidebar_state.clone();
    f.render_stateful_widget(sidebar, main[0], &mut sidebar_state);

    // 메인 리스트
    let main_items: Vec<ListItem> = state.main_items.iter()
        .map(|(_, disp)| ListItem::new(disp.as_str())).collect();
    let main_block = Block::default()
        .title(state.section.label())
        .borders(Borders::ALL)
        .border_style(if state.focus == Focus::Main { Style::default().fg(Color::Cyan) } else { Style::default() });
    let main_list = List::new(main_items)
        .block(main_block)
        .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .highlight_symbol("▶ ");
    let mut main_state = state.main_state.clone();
    f.render_stateful_widget(main_list, main[1], &mut main_state);

    // 디테일
    let detail_block = Block::default()
        .title("상세")
        .borders(Borders::ALL)
        .border_style(if state.focus == Focus::Detail { Style::default().fg(Color::Cyan) } else { Style::default() });
    let detail = Paragraph::new(state.detail_text.as_str())
        .block(detail_block)
        .wrap(Wrap { trim: false });
    f.render_widget(detail, main[2]);

    // 하단 도움말
    let help = Paragraph::new("Tab: 포커스 이동  ↑↓/jk: 선택  r: 새로고침  q/Esc: 종료")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[1]);
}
