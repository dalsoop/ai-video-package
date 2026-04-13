use crate::SkillCmd;
use git2::{Repository, Signature, IndexAddOption, DiffOptions, StatusOptions};

/// 스킬 레포 경로 — 바이너리가 설치된 소스 디렉토리
fn skill_repo_dir() -> std::path::PathBuf {
    // Cargo.toml이 있는 디렉토리 = 스킬 레포 루트
    // 빌드 시 env!로 고정
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn open_repo() -> Repository {
    Repository::open(skill_repo_dir()).unwrap_or_else(|e| {
        eprintln!("스킬 레포를 열 수 없습니다: {}", e);
        eprintln!("  경로: {}", skill_repo_dir().display());
        std::process::exit(1);
    })
}

pub fn run(cmd: SkillCmd) {
    match cmd {
        SkillCmd::Status => status(),
        SkillCmd::Push { message } => push(message),
        SkillCmd::Diff => diff(),
        SkillCmd::Log { count } => log(count),
    }
}

fn status() {
    let repo = open_repo();
    let mut opts = StatusOptions::new();
    opts.include_untracked(true);

    let statuses = repo.statuses(Some(&mut opts)).unwrap();

    if statuses.is_empty() {
        println!("✅ 스킬 파일: 변경 없음 (clean)");
    } else {
        println!("📝 스킬 파일 변경사항:");
        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("?");
            let status = entry.status();
            let mark = if status.is_wt_new() {
                "추가"
            } else if status.is_wt_modified() {
                "수정"
            } else if status.is_wt_deleted() {
                "삭제"
            } else if status.is_index_new() {
                "스테이지(추가)"
            } else if status.is_index_modified() {
                "스테이지(수정)"
            } else {
                "변경"
            };
            println!("  [{}] {}", mark, path);
        }
        println!();
        println!("  `avp skill push` 로 커밋+push 하세요.");
    }

    println!();
    println!("스킬 경로: {}", skill_repo_dir().display());

    // 스킬 파일 목록
    let skill_files = ["SKILL.md", "references/video-cut.md", "references/keycut-image.md",
        "references/character.md", "references/background.md", "references/enemy.md",
        "references/storyboard.md"];
    println!();
    println!("📄 스킬 파일:");
    for f in &skill_files {
        let path = skill_repo_dir().join(f);
        let exists = if path.exists() { "✅" } else { "❌" };
        println!("  {} {}", exists, f);
    }
}

fn push(message: Option<String>) {
    let repo = open_repo();

    // 스테이지
    let mut index = repo.index().unwrap();
    index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None).unwrap();
    index.write().unwrap();

    let tree_oid = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();

    let sig = Signature::now("avp", "avp@local").unwrap();
    let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());

    let msg = message.unwrap_or_else(|| "skill: 스킬 파일 업데이트".to_string());

    match parent {
        Some(ref p) => {
            repo.commit(Some("HEAD"), &sig, &sig, &msg, &tree, &[p]).unwrap();
        }
        None => {
            repo.commit(Some("HEAD"), &sig, &sig, &msg, &tree, &[]).unwrap();
        }
    }

    println!("✅ 커밋 완료: {}", msg);

    // push (git2로는 remote push가 복잡하므로 시스템 git 사용)
    let push_result = std::process::Command::new("git")
        .args(["push"])
        .current_dir(skill_repo_dir())
        .output();

    match push_result {
        Ok(output) if output.status.success() => {
            println!("✅ push 완료");
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("up-to-date") || stderr.contains("Everything up-to-date") {
                println!("✅ push 완료 (이미 최신)");
            } else {
                eprintln!("⚠️  push 실패: {}", stderr.trim());
            }
        }
        Err(e) => {
            eprintln!("⚠️  push 실행 실패: {} (git CLI 필요)", e);
        }
    }
}

fn diff() {
    let repo = open_repo();
    let mut opts = DiffOptions::new();

    let diff = repo.diff_index_to_workdir(None, Some(&mut opts)).unwrap();

    if diff.deltas().len() == 0 {
        println!("변경사항 없음");
        return;
    }

    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        let prefix = match line.origin() {
            '+' => "\x1b[32m+",
            '-' => "\x1b[31m-",
            _ => " ",
        };
        print!("{}{}\x1b[0m", prefix, std::str::from_utf8(line.content()).unwrap_or(""));
        true
    }).unwrap();
}

fn log(count: usize) {
    let repo = open_repo();
    let mut revwalk = repo.revwalk().unwrap();
    revwalk.push_head().ok();

    println!("📜 스킬 레포 이력:");
    for (i, oid) in revwalk.enumerate() {
        if i >= count { break; }
        if let Ok(oid) = oid {
            if let Ok(commit) = repo.find_commit(oid) {
                let msg = commit.message().unwrap_or("(no message)"); // LINT_ALLOW: 표시용
                let time = commit.time();
                let ts = chrono::DateTime::from_timestamp(time.seconds(), 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_default();
                let short = &oid.to_string()[..7];
                println!("  {} {} {}", short, ts, msg.trim());
            }
        }
    }
}
