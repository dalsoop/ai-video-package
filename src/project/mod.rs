use crate::{ProjectCmd, PROJECT_DIR};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct ProjectMeta {
    pub name: String,
    pub genre: String,
    pub project_type: String,
    pub phase: u8,
    pub created_at: String,
}

fn avp_dir() -> PathBuf {
    std::env::current_dir().unwrap().join(PROJECT_DIR)
}

fn project_dir(name: &str) -> PathBuf {
    avp_dir().join("projects").join(name)
}

fn current_file() -> PathBuf {
    avp_dir().join("current")
}

fn current_project_name() -> Option<String> {
    fs::read_to_string(current_file()).ok().map(|s| s.trim().to_string())
}

pub fn run(cmd: ProjectCmd) {
    match cmd {
        ProjectCmd::Init { name, genre, r#type, phase } => init(&name, genre, r#type, phase),
        ProjectCmd::List => list(),
        ProjectCmd::Status { name } => {
            let target = name.or_else(current_project_name);
            match target {
                Some(n) => show_status(&n),
                None => eprintln!("프로젝트를 지정하거나 `{} project use <이름>`으로 선택하세요.", crate::BIN_NAME),
            }
        }
        ProjectCmd::Use { name } => use_project(&name),
    }
}

fn init(name: &str, genre: Option<String>, project_type: Option<String>, phase: u8) {
    let dir = project_dir(name);
    if dir.exists() {
        eprintln!("이미 존재하는 프로젝트: {}", name);
        std::process::exit(1);
    }

    // 프로젝트 디렉토리 구조 생성
    fs::create_dir_all(dir.join("assets/characters")).unwrap();
    fs::create_dir_all(dir.join("assets/monsters")).unwrap();
    fs::create_dir_all(dir.join("assets/backgrounds")).unwrap();
    fs::create_dir_all(dir.join("assets/keycuts")).unwrap();
    fs::create_dir_all(dir.join("cuts")).unwrap();
    fs::create_dir_all(dir.join("prompts")).unwrap();
    fs::create_dir_all(dir.join("frames")).unwrap();
    fs::create_dir_all(dir.join("videos")).unwrap();

    let meta = ProjectMeta {
        name: name.to_string(),
        genre: genre.unwrap_or_else(|| "미정".to_string()),
        project_type: project_type.unwrap_or_else(|| "단편".to_string()),
        phase,
        created_at: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
    };

    let json = serde_json::to_string_pretty(&meta).unwrap();
    fs::write(dir.join("project.json"), &json).unwrap();

    // 현재 프로젝트로 설정
    fs::create_dir_all(avp_dir()).unwrap();
    fs::write(current_file(), name).unwrap();

    println!("✅ 프로젝트 생성: {}", name);
    println!("   경로: {}", dir.display());
    println!("   장르: {}", meta.genre);
    println!("   유형: {}", meta.project_type);
    println!("   Phase: {}", meta.phase);
    println!();
    println!("디렉토리 구조:");
    println!("  assets/characters/  — 캐릭터 이미지");
    println!("  assets/monsters/    — 몬스터 이미지");
    println!("  assets/backgrounds/ — 배경 이미지");
    println!("  assets/keycuts/     — 키컷 이미지");
    println!("  cuts/               — 컷 메타데이터");
    println!("  prompts/            — 프롬프트 저장");
    println!("  frames/             — 마지막 프레임 캡처");
    println!("  videos/             — 완성 영상");
}

fn list() {
    let projects_dir = avp_dir().join("projects");
    if !projects_dir.exists() {
        println!("프로젝트가 없습니다.");
        return;
    }

    let current = current_project_name();

    let mut entries: Vec<_> = fs::read_dir(&projects_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    if entries.is_empty() {
        println!("프로젝트가 없습니다.");
        return;
    }

    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();
        let marker = if current.as_deref() == Some(&name) { " ◀ current" } else { "" };
        let meta_path = entry.path().join("project.json");
        if let Ok(json) = fs::read_to_string(&meta_path) {
            if let Ok(meta) = serde_json::from_str::<ProjectMeta>(&json) {
                println!("  {} — {} / {} / Phase {}{}", name, meta.project_type, meta.genre, meta.phase, marker);
            }
        }
    }
}

fn use_project(name: &str) {
    let dir = project_dir(name);
    if !dir.exists() {
        eprintln!("프로젝트를 찾을 수 없습니다: {}", name);
        std::process::exit(1);
    }
    fs::write(current_file(), name).unwrap();
    println!("✅ 현재 프로젝트: {}", name);
}

fn show_status(name: &str) {
    let dir = project_dir(name);
    let meta_path = dir.join("project.json");

    if !meta_path.exists() {
        eprintln!("프로젝트를 찾을 수 없습니다: {}", name);
        return;
    }

    let meta: ProjectMeta = serde_json::from_str(&fs::read_to_string(&meta_path).unwrap()).unwrap();

    println!("📋 프로젝트: {}", meta.name);
    println!("   유형: {} / 장르: {} / Phase: {}", meta.project_type, meta.genre, meta.phase);
    println!("   생성: {}", meta.created_at);
    println!();

    // 에셋 카운트
    let asset_types = ["characters", "monsters", "backgrounds", "keycuts"];
    println!("🎨 에셋:");
    for t in &asset_types {
        let asset_dir = dir.join("assets").join(t);
        let count = fs::read_dir(&asset_dir)
            .map(|rd| rd.filter_map(|e| e.ok()).filter(|e| e.path().extension().is_some_and(|ext| ext == "json")).count())
            .unwrap_or(0);
        let label = match *t {
            "characters" => "캐릭터",
            "monsters" => "몬스터",
            "backgrounds" => "배경",
            "keycuts" => "키컷",
            _ => t,
        };
        let check = if count > 0 { "✅" } else { "⬜" };
        println!("  {} {} — {}개", check, label, count);
    }
    println!();

    // 컷 카운트
    let cuts_dir = dir.join("cuts");
    let total_cuts = fs::read_dir(&cuts_dir)
        .map(|rd| rd.filter_map(|e| e.ok()).filter(|e| e.path().extension().is_some_and(|ext| ext == "json")).count())
        .unwrap_or(0);
    let done_cuts = fs::read_dir(&cuts_dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter(|e| {
                    if let Ok(json) = fs::read_to_string(e.path()) {
                        json.contains("\"done\":true") || json.contains("\"done\": true")
                    } else {
                        false
                    }
                })
                .count()
        })
        .unwrap_or(0);
    println!("🎬 컷: {}/{} 완성", done_cuts, total_cuts);
}

pub fn status_current() {
    match current_project_name() {
        Some(name) => show_status(&name),
        None => {
            eprintln!("현재 프로젝트가 없습니다.");
            eprintln!("  {} project init <이름> 으로 생성하세요.", crate::BIN_NAME);
        }
    }
}
