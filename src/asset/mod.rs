use crate::{AssetCmd, PROJECT_DIR};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct AssetMeta {
    pub name: String,
    pub asset_type: String,
    pub image: Option<String>,
    pub prompt: Option<String>,
    pub keywords: Option<String>,
    pub threat: Option<String>,
    pub created_at: String,
}

fn current_project_dir() -> PathBuf {
    let avp = std::env::current_dir().unwrap().join(PROJECT_DIR);
    let current = fs::read_to_string(avp.join("current"))
        .unwrap_or_else(|_| {
            eprintln!("현재 프로젝트가 없습니다. `{} project init` 또는 `{} project use`로 설정하세요.", crate::BIN_NAME, crate::BIN_NAME);
            std::process::exit(1);
        })
        .trim()
        .to_string();
    avp.join("projects").join(current)
}

fn type_to_dir(asset_type: &str) -> &str {
    match asset_type {
        "character" => "characters",
        "monster" => "monsters",
        "background" => "backgrounds",
        "keycut" => "keycuts",
        other => {
            eprintln!("알 수 없는 에셋 유형: {} (character/monster/background/keycut)", other);
            std::process::exit(1);
        }
    }
}

pub fn run(cmd: AssetCmd) {
    match cmd {
        AssetCmd::Add { r#type, name, image, prompt, keywords, threat } => {
            add(&r#type, &name, image, prompt, keywords, threat);
        }
        AssetCmd::List { r#type } => list(r#type.as_deref()),
        AssetCmd::Show { name } => show(&name),
        AssetCmd::Remove { name } => remove(&name),
    }
}

fn add(asset_type: &str, name: &str, image: Option<String>, prompt: Option<String>, keywords: Option<String>, threat: Option<String>) {
    let dir = current_project_dir();
    let type_dir = dir.join("assets").join(type_to_dir(asset_type));

    // 이미지 파일 복사
    let stored_image = if let Some(ref img_path) = image {
        let src = PathBuf::from(img_path);
        if !src.exists() {
            eprintln!("이미지 파일을 찾을 수 없습니다: {}", img_path);
            std::process::exit(1);
        }
        let ext = src.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_else(|| "png".to_string());
        let dest_name = format!("{}.{}", name, ext);
        let dest = type_dir.join(&dest_name);
        fs::copy(&src, &dest).unwrap();
        Some(dest_name)
    } else {
        None
    };

    let meta = AssetMeta {
        name: name.to_string(),
        asset_type: asset_type.to_string(),
        image: stored_image.clone(),
        prompt,
        keywords,
        threat,
        created_at: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
    };

    let json = serde_json::to_string_pretty(&meta).unwrap();
    fs::write(type_dir.join(format!("{}.json", name)), &json).unwrap();

    println!("✅ 에셋 추가: [{}] {}", asset_type, name);
    if let Some(img) = &stored_image {
        println!("   이미지: {}", img);
    }
    if let Some(kw) = &meta.keywords {
        println!("   키워드: {}", kw);
    }

    // 키컷 에셋 추가 시 연속 프레임 카운트 리셋
    if asset_type == "keycut" {
        crate::project::reset_consecutive_frames(0);
        println!("   🔄 연속 프레임 카운트 리셋됨");
    }

    crate::git::auto_commit(&format!("asset: [{}] {} 추가", asset_type, name));
}

fn list(type_filter: Option<&str>) {
    let dir = current_project_dir();
    let types: Vec<&str> = match type_filter {
        Some(t) => vec![type_to_dir(t)],
        None => vec!["characters", "monsters", "backgrounds", "keycuts"],
    };

    for t in types {
        let asset_dir = dir.join("assets").join(t);
        let label = match t {
            "characters" => "캐릭터",
            "monsters" => "몬스터",
            "backgrounds" => "배경",
            "keycuts" => "키컷",
            _ => t,
        };

        let entries: Vec<_> = fs::read_dir(&asset_dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
            .collect();

        if entries.is_empty() {
            println!("⬜ {} — 없음", label);
        } else {
            println!("✅ {} ({}개)", label, entries.len());
            for entry in entries {
                if let Ok(json) = fs::read_to_string(entry.path()) {
                    if let Ok(meta) = serde_json::from_str::<AssetMeta>(&json) {
                        let img_mark = if meta.image.is_some() { "🖼" } else { "  " };
                        let threat_str = meta.threat.map(|t| format!(" [{}]", t)).unwrap_or_default();
                        println!("  {} {}{}", img_mark, meta.name, threat_str);
                    }
                }
            }
        }
    }
}

fn show(name: &str) {
    let dir = current_project_dir();
    let types = ["characters", "monsters", "backgrounds", "keycuts"];

    for t in &types {
        let path = dir.join("assets").join(t).join(format!("{}.json", name));
        if path.exists() {
            let json = fs::read_to_string(&path).unwrap();
            let meta: AssetMeta = serde_json::from_str(&json).unwrap();
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("🎨 {} — [{}]", meta.name, meta.asset_type);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            if let Some(img) = &meta.image { println!("이미지: {}", img); }
            if let Some(kw) = &meta.keywords { println!("키워드: {}", kw); }
            if let Some(th) = &meta.threat { println!("위협등급: {}", th); }
            if let Some(pr) = &meta.prompt { println!("프롬프트:\n{}", pr); }
            println!("생성: {}", meta.created_at);
            return;
        }
    }
    eprintln!("에셋을 찾을 수 없습니다: {}", name);
}

fn remove(name: &str) {
    let dir = current_project_dir();
    let types = ["characters", "monsters", "backgrounds", "keycuts"];

    for t in &types {
        let json_path = dir.join("assets").join(t).join(format!("{}.json", name));
        if json_path.exists() {
            // 이미지도 같이 삭제
            if let Ok(json) = fs::read_to_string(&json_path) {
                if let Ok(meta) = serde_json::from_str::<AssetMeta>(&json) {
                    if let Some(img) = &meta.image {
                        let img_path = dir.join("assets").join(t).join(img);
                        let _ = fs::remove_file(img_path);
                    }
                }
            }
            fs::remove_file(&json_path).unwrap();
            println!("🗑 에셋 삭제: {}", name);
            crate::git::auto_commit(&format!("asset: {} 삭제", name));
            return;
        }
    }
    eprintln!("에셋을 찾을 수 없습니다: {}", name);
}
