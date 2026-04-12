use crate::{CutCmd, PROJECT_DIR};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct CutMeta {
    pub number: u32,
    pub title: String,
    pub phase: u8,
    pub description: Option<String>,
    pub linked_assets: Vec<String>,
    pub start_frame: Option<String>,
    pub last_frame: Option<String>,
    pub video: Option<String>,
    pub done: bool,
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

fn next_cut_number(dir: &PathBuf) -> u32 {
    let cuts_dir = dir.join("cuts");
    fs::read_dir(&cuts_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            e.path()
                .file_stem()
                .and_then(|s| s.to_string_lossy().strip_prefix("cut_").map(|n| n.parse::<u32>().ok()).flatten())
        })
        .max()
        .unwrap_or(0)
        + 1
}

pub fn run(cmd: CutCmd) {
    match cmd {
        CutCmd::Add { title, phase, desc } => add(&title, phase, desc),
        CutCmd::List => list(),
        CutCmd::Show { number } => show(number),
        CutCmd::Done { number, video, last_frame } => done(number, video, last_frame),
        CutCmd::Link { number, asset } => link(number, &asset),
    }
}

fn add(title: &str, phase: Option<u8>, desc: Option<String>) {
    let dir = current_project_dir();
    let number = next_cut_number(&dir);

    let meta = CutMeta {
        number,
        title: title.to_string(),
        phase: phase.unwrap_or(2),
        description: desc,
        linked_assets: vec![],
        start_frame: None,
        last_frame: None,
        video: None,
        done: false,
        created_at: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
    };

    let json = serde_json::to_string_pretty(&meta).unwrap();
    fs::write(dir.join("cuts").join(format!("cut_{:03}.json", number)), &json).unwrap();

    println!("✅ 컷 #{} 추가: {}", number, title);
    println!("   Phase: {}", meta.phase);
}

fn list() {
    let dir = current_project_dir();
    let cuts_dir = dir.join("cuts");

    let mut entries: Vec<_> = fs::read_dir(&cuts_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    if entries.is_empty() {
        println!("컷이 없습니다.");
        return;
    }

    println!("🎬 컷 목록:");
    for entry in entries {
        if let Ok(json) = fs::read_to_string(entry.path()) {
            if let Ok(meta) = serde_json::from_str::<CutMeta>(&json) {
                let status = if meta.done { "✅" } else { "⬜" };
                let assets = if meta.linked_assets.is_empty() {
                    String::new()
                } else {
                    format!(" [에셋: {}]", meta.linked_assets.join(", "))
                };
                println!("  {} #{:02} — {} | Phase {}{}", status, meta.number, meta.title, meta.phase, assets);
            }
        }
    }
}

fn show(number: u32) {
    let dir = current_project_dir();
    let path = dir.join("cuts").join(format!("cut_{:03}.json", number));

    if !path.exists() {
        eprintln!("컷 #{}을 찾을 수 없습니다.", number);
        return;
    }

    let meta: CutMeta = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🎬 컷 #{:02}: {}", meta.number, meta.title);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Phase: {}", meta.phase);
    println!("상태: {}", if meta.done { "✅ 완성" } else { "⬜ 미완성" });
    if let Some(desc) = &meta.description { println!("설명: {}", desc); }
    if !meta.linked_assets.is_empty() { println!("에셋: {}", meta.linked_assets.join(", ")); }
    if let Some(sf) = &meta.start_frame { println!("시작 프레임: {}", sf); }
    if let Some(lf) = &meta.last_frame { println!("마지막 프레임: {}", lf); }
    if let Some(v) = &meta.video { println!("영상: {}", v); }
}

fn done(number: u32, video: Option<String>, last_frame: Option<String>) {
    let dir = current_project_dir();
    let path = dir.join("cuts").join(format!("cut_{:03}.json", number));

    if !path.exists() {
        eprintln!("컷 #{}을 찾을 수 없습니다.", number);
        return;
    }

    let mut meta: CutMeta = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    meta.done = true;

    // 영상 파일 복사
    if let Some(ref v) = video {
        let src = PathBuf::from(v);
        if src.exists() {
            let dest = dir.join("videos").join(format!("cut_{:03}.mp4", number));
            fs::copy(&src, &dest).unwrap();
            meta.video = Some(format!("cut_{:03}.mp4", number));
        }
    }

    // 마지막 프레임 복사
    if let Some(ref lf) = last_frame {
        let src = PathBuf::from(lf);
        if src.exists() {
            let ext = src.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_else(|| "png".to_string());
            let dest = dir.join("frames").join(format!("cut_{:03}_last.{}", number, ext));
            fs::copy(&src, &dest).unwrap();
            meta.last_frame = Some(format!("cut_{:03}_last.{}", number, ext));
        }
    }

    let json = serde_json::to_string_pretty(&meta).unwrap();
    fs::write(&path, &json).unwrap();
    println!("✅ 컷 #{} 완성!", number);
}

fn link(number: u32, asset: &str) {
    let dir = current_project_dir();
    let path = dir.join("cuts").join(format!("cut_{:03}.json", number));

    if !path.exists() {
        eprintln!("컷 #{}을 찾을 수 없습니다.", number);
        return;
    }

    let mut meta: CutMeta = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    if !meta.linked_assets.contains(&asset.to_string()) {
        meta.linked_assets.push(asset.to_string());
    }

    let json = serde_json::to_string_pretty(&meta).unwrap();
    fs::write(&path, &json).unwrap();
    println!("🔗 컷 #{} ← 에셋 '{}'", number, asset);
}
