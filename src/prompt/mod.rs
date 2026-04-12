use crate::{PromptCmd, PROJECT_DIR};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const SEEDANCE_LIMIT: usize = 3000;
const MIDJOURNEY_LIMIT: usize = 1000;

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
        .unwrap_or_else(|_| {
            eprintln!("현재 프로젝트가 없습니다.");
            std::process::exit(1);
        })
        .trim()
        .to_string();
    avp.join("projects").join(current)
}

fn limit_for(prompt_type: &str) -> usize {
    match prompt_type {
        "seedance" => SEEDANCE_LIMIT,
        "midjourney" => MIDJOURNEY_LIMIT,
        _ => {
            eprintln!("알 수 없는 프롬프트 유형: {} (seedance/midjourney)", prompt_type);
            std::process::exit(1);
        }
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

    let meta = PromptMeta {
        target: target.to_string(),
        prompt_type: prompt_type.to_string(),
        text: text.to_string(),
        char_count,
        within_limit,
        created_at: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
    };

    let json = serde_json::to_string_pretty(&meta).unwrap();
    let filename = format!("{}_{}.json", target, prompt_type);
    fs::write(dir.join("prompts").join(&filename), &json).unwrap();

    let status = if within_limit { "✅" } else { "⚠️ 초과!" };
    println!("{} 프롬프트 저장: {} ({})", status, target, prompt_type);
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
                    let limit = limit_for(&meta.prompt_type);
                    let status = if meta.within_limit { "✅" } else { "⚠️" };
                    println!("{} 글자수: {}/{}", status, meta.char_count, limit);
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    println!("{}", meta.text);
                    println!();
                    found = true;
                }
            }
        }
    }

    if !found {
        eprintln!("'{}'에 대한 프롬프트를 찾을 수 없습니다.", target);
    }
}

fn check(prompt_type: &str, text: &str) {
    let limit = limit_for(prompt_type);
    let count = text.len();
    let label = match prompt_type {
        "seedance" => "Seedance",
        "midjourney" => "Midjourney",
        _ => prompt_type,
    };

    if count <= limit {
        println!("✅ {} 프롬프트: {}/{}자 — OK", label, count, limit);
    } else {
        println!("⚠️  {} 프롬프트: {}/{}자 — {}자 초과!", label, count, limit, count - limit);
    }
}
