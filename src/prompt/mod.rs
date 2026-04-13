use crate::{PromptCmd, PROJECT_DIR};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// === 규칙 엔진 (aip과 동일 구조) ===

#[derive(Deserialize)]
struct RulesFile { rules: Vec<Rule> }

#[derive(Deserialize)]
struct Rule {
    id: String,
    severity: String,
    #[serde(rename = "match")]
    match_def: MatchDef,
    actions: Vec<Action>,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum MatchDef {
    #[serde(rename = "list")]
    List { keywords: Vec<String>, #[serde(default)] exceptions: Vec<String>, #[serde(default = "default_scope")] scope: String },
    #[serde(rename = "section_check")]
    SectionCheck { sections: Vec<SectionDef> },
    #[serde(rename = "duplicate")]
    Duplicate { min_word_length: usize, threshold: usize, ignore: Vec<String> },
    #[serde(rename = "regex")]
    Regex { pattern: String, #[serde(default)] allowed_values: Vec<String>, #[serde(default)] range: Option<RangeCheck>, #[serde(default)] default: Option<u32>, #[serde(default = "default_scope")] scope: String },
    #[serde(rename = "contains_korean")]
    ContainsKorean { #[serde(default = "default_scope")] scope: String },
    #[serde(rename = "required_if")]
    RequiredIf { trigger_keywords: Vec<String>, required_keywords: Vec<String>, #[serde(default = "default_scope")] scope: String },
    #[serde(rename = "required_keywords")]
    RequiredKeywords { keywords: Vec<String>, #[serde(default = "default_scope")] scope: String },
    #[serde(rename = "char_limit")]
    CharLimit { max: usize, #[serde(default = "default_scope")] scope: String },
}

fn default_scope() -> String { "all".to_string() }

#[derive(Deserialize)] struct SectionDef { marker: String, min_length: usize }
#[derive(Deserialize, Clone)] struct RangeCheck { min: i64, max: i64 }
#[derive(Deserialize)] struct Action { r#type: String, #[serde(default)] text: String }

fn load_rules(prompt_type: &str) -> Option<RulesFile> {
    let filename = match prompt_type {
        "seedance" => "video_prompt.json",
        "midjourney" => "video_prompt.json", // avp는 영상용 규칙 하나로
        _ => return None,
    };
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("rules").join(filename);
    let content = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

#[derive(Default)]
struct ValidationResult { errors: Vec<String>, warnings: Vec<String> }

fn get_search_text(text: &str, scope: &str) -> String {
    if scope == "all" { text.to_lowercase() }
    else if let Some(name) = scope.strip_prefix("section:") {
        let marker = format!("## {}", name);
        text.split(&marker).nth(1).map(|a| a.split("## ").next().unwrap_or("").trim().to_lowercase()).unwrap_or_default()
    } else { text.to_lowercase() }
}

fn collect_messages(actions: &[Action]) -> String {
    actions.iter().filter(|a| a.r#type == "message" || a.r#type == "suggest").map(|a| a.text.clone()).collect::<Vec<_>>().join(" ")
}

fn validate(text: &str, prompt_type: &str) -> ValidationResult {
    let mut result = ValidationResult::default();
    let rules_file = match load_rules(prompt_type) {
        Some(r) => r,
        None => { result.warnings.push("규칙 파일 로드 실패 — 검증 건너뜀".to_string()); return result; }
    };

    for rule in &rules_file.rules {
        match &rule.match_def {
            MatchDef::List { keywords, exceptions, scope } => {
                let search = get_search_text(text, scope);
                let found: Vec<_> = keywords.iter()
                    .filter(|kw| search.contains(&kw.to_lowercase()) && !exceptions.iter().any(|ex| search.contains(&ex.to_lowercase())))
                    .cloned().collect();
                if !found.is_empty() {
                    let msg = format!("[{}] {} — 감지: [{}]", rule.id, collect_messages(&rule.actions), found.join(", "));
                    push(&mut result, &rule.severity, &msg);
                }
            }
            MatchDef::ContainsKorean { scope } => {
                let search = get_search_text(text, scope);
                if !search.chars().any(|c| ('\u{AC00}'..='\u{D7AF}').contains(&c)) {
                    push(&mut result, &rule.severity, &format!("[{}] {}", rule.id, collect_messages(&rule.actions)));
                }
            }
            MatchDef::RequiredKeywords { keywords, scope } => {
                let search = get_search_text(text, scope);
                let missing: Vec<_> = keywords.iter().filter(|kw| !search.contains(&kw.to_lowercase())).collect();
                if !missing.is_empty() {
                    push(&mut result, &rule.severity, &format!("[{}] {} — 누락: [{}]", rule.id, collect_messages(&rule.actions), missing.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")));
                }
            }
            MatchDef::RequiredIf { trigger_keywords, required_keywords, scope } => {
                let search = get_search_text(text, scope);
                if trigger_keywords.iter().any(|kw| search.contains(&kw.to_lowercase())) {
                    if !required_keywords.iter().any(|kw| search.contains(&kw.to_lowercase())) {
                        push(&mut result, &rule.severity, &format!("[{}] {}", rule.id, collect_messages(&rule.actions)));
                    }
                }
            }
            MatchDef::CharLimit { max, scope } => {
                let search = get_search_text(text, scope);
                if search.len() > *max {
                    push(&mut result, &rule.severity, &format!("[{}] {} ({}자 > {}자)", rule.id, collect_messages(&rule.actions), search.len(), max));
                }
            }
            MatchDef::Regex { pattern, allowed_values, range, default, scope } => {
                let search = get_search_text(text, scope);
                if let Ok(re) = regex_lite::Regex::new(pattern) {
                    for cap in re.captures_iter(&search) {
                        if let Some(val) = cap.get(1) {
                            let v = val.as_str();
                            if !allowed_values.is_empty() && !allowed_values.iter().any(|a| a.to_lowercase() == v) {
                                push(&mut result, &rule.severity, &format!("[{}] {} (값: {})", rule.id, collect_messages(&rule.actions), v));
                            }
                            if let Some(r) = range {
                                if let Ok(n) = v.parse::<i64>() {
                                    if n < r.min || n > r.max {
                                        let def = default.map(|d| format!(", 기본값: {}", d)).unwrap_or_default();
                                        push(&mut result, &rule.severity, &format!("[{}] 범위 초과: {} ({}~{}{})", rule.id, n, r.min, r.max, def));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // avp에서는 section_check/duplicate는 사용하지 않음
            _ => {}
        }
    }
    result
}

fn push(result: &mut ValidationResult, severity: &str, msg: &str) {
    match severity {
        "error" => result.errors.push(msg.to_string()),
        "warn" => result.warnings.push(msg.to_string()),
        _ => result.warnings.push(format!("ℹ️  {}", msg)),
    }
}

fn print_validation(result: &ValidationResult) {
    if result.errors.is_empty() && result.warnings.is_empty() {
        println!("✅ 검증 통과");
        return;
    }
    for e in &result.errors { println!("  ❌ {}", e); }
    for w in &result.warnings { println!("  ⚠️  {}", w); }
}

// === 프롬프트 관리 ===

#[derive(Serialize, Deserialize)]
pub struct PromptMeta {
    pub target: String,
    pub prompt_type: String,
    pub text: String,
    pub char_count: usize,
    pub within_limit: bool,
    pub created_at: String,
}

fn current_project_dir() -> PathBuf {
    let avp = std::env::current_dir().unwrap().join(PROJECT_DIR);
    let current = fs::read_to_string(avp.join("current"))
        .unwrap_or_else(|_| { eprintln!("현재 프로젝트가 없습니다."); std::process::exit(1); })
        .trim().to_string();
    avp.join("projects").join(current)
}

fn limit_for(prompt_type: &str) -> usize {
    match prompt_type {
        "seedance" => 3000,
        "midjourney" => 6000,
        _ => { eprintln!("알 수 없는 유형: {} (seedance/midjourney)", prompt_type); std::process::exit(1); }
    }
}

pub fn run(cmd: PromptCmd) {
    match cmd {
        PromptCmd::Save { target, r#type, text } => save(&target, &r#type, &text),
        PromptCmd::Show { target } => show(&target),
        PromptCmd::Check { r#type, text } => check(&r#type, &text),
    }
}

fn save(target: &str, prompt_type: &str, text: &str) {
    let dir = current_project_dir();
    let limit = limit_for(prompt_type);
    let char_count = text.len();
    let within_limit = char_count <= limit;

    // === 하드 강제: 스타일 접두사 필수 ===
    let meta_path = dir.join("project.json");
    if let Ok(json) = fs::read_to_string(&meta_path) {
        if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&json) {
            let prefix = meta["style_prefix"].as_str().unwrap_or("");
            if prefix.is_empty() {
                eprintln!("❌ 스타일 접두사가 설정되지 않았습니다. 프롬프트 저장 거부.");
                eprintln!("   먼저 `{} project style \"키워드\"` 로 설정하세요.", crate::BIN_NAME);
                std::process::exit(1);
            }
        }
    }

    // === 규칙 엔진 검증 ===
    println!("📋 검증:");
    let validation = validate(text, prompt_type);
    print_validation(&validation);

    if !validation.errors.is_empty() {
        eprintln!();
        eprintln!("❌ 규칙 위반(error)이 있습니다. 프롬프트 저장 거부.");
        std::process::exit(1);
    }

    // === 저장 ===
    let meta = PromptMeta {
        target: target.to_string(),
        prompt_type: prompt_type.to_string(),
        text: text.to_string(),
        char_count,
        within_limit,
        created_at: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
    };

    let filename = format!("{}_{}.json", target, prompt_type);
    fs::write(dir.join("prompts").join(&filename), serde_json::to_string_pretty(&meta).unwrap()).unwrap();

    println!();
    println!("✅ 프롬프트 저장: {} ({})", target, prompt_type);
    println!("   글자수: {}/{}", char_count, limit);

    crate::git::auto_commit(&format!("prompt: {} {} 저장", target, prompt_type));
}

fn show(target: &str) {
    let dir = current_project_dir();
    let prompts_dir = dir.join("prompts");
    let mut found = false;
    for entry in fs::read_dir(&prompts_dir).into_iter().flatten().filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(target) && name.ends_with(".json") {
            if let Ok(json) = fs::read_to_string(entry.path()) {
                if let Ok(meta) = serde_json::from_str::<PromptMeta>(&json) {
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    println!("📝 {} — {}", meta.target, meta.prompt_type);
                    println!("   글자수: {}/{}", meta.char_count, limit_for(&meta.prompt_type));
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    println!("{}", meta.text);
                    println!();
                    found = true;
                }
            }
        }
    }
    if !found { eprintln!("'{}'에 대한 프롬프트를 찾을 수 없습니다.", target); }
}

fn check(prompt_type: &str, text: &str) {
    let limit = limit_for(prompt_type);
    let count = text.len();
    if count <= limit {
        println!("✅ {} 프롬프트: {}/{}자 — OK", prompt_type, count, limit);
    } else {
        println!("⚠️  {} 프롬프트: {}/{}자 — {}자 초과!", prompt_type, count, limit, count - limit);
    }
    println!();
    println!("📋 규칙 검증:");
    let validation = validate(text, prompt_type);
    print_validation(&validation);
}
