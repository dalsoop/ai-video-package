use git2::{Repository, Signature, IndexAddOption};
use crate::PROJECT_DIR;

fn avp_dir() -> std::path::PathBuf {
    std::env::current_dir().unwrap().join(PROJECT_DIR)
}

/// .avp/ 디렉토리를 git repo로 초기화
pub fn init_repo() {
    let dir = avp_dir();
    if dir.join(".git").exists() {
        return; // 이미 초기화됨
    }
    Repository::init(&dir).expect(".avp/ git 초기화 실패");

    // .gitignore 생성 — 대용량 영상 파일 제외
    let gitignore = dir.join(".gitignore");
    if !gitignore.exists() {
        std::fs::write(&gitignore, "*.mp4\n*.mov\n*.avi\n*.mkv\n").unwrap();
    }

    // 초기 커밋
    auto_commit("init: 프로젝트 저장소 초기화");
}

/// 모든 변경사항을 스테이지하고 커밋
pub fn auto_commit(message: &str) {
    let dir = avp_dir();
    let repo = match Repository::open(&dir) {
        Ok(r) => r,
        Err(_) => return, // repo 없으면 무시
    };

    let mut index = repo.index().expect("index 열기 실패");

    // 모든 파일 추가
    index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
        .expect("파일 스테이징 실패");
    index.write().expect("index 쓰기 실패");

    let tree_oid = index.write_tree().expect("tree 쓰기 실패");
    let tree = repo.find_tree(tree_oid).expect("tree 찾기 실패");

    let sig = Signature::now("avp", "avp@local").expect("서명 생성 실패");

    // HEAD가 있으면 parent 커밋, 없으면 초기 커밋
    let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());

    match parent {
        Some(ref p) => {
            repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[p])
                .expect("커밋 실패");
        }
        None => {
            repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[])
                .expect("초기 커밋 실패");
        }
    }
}

/// git log 요약 출력
pub fn log_summary(count: usize) {
    let dir = avp_dir();
    let repo = match Repository::open(&dir) {
        Ok(r) => r,
        Err(_) => {
            println!("  git 이력 없음");
            return;
        }
    };

    let mut revwalk = match repo.revwalk() {
        Ok(r) => r,
        Err(_) => return,
    };
    revwalk.push_head().ok();

    println!("📜 최근 이력:");
    for (i, oid) in revwalk.enumerate() {
        if i >= count { break; }
        if let Ok(oid) = oid {
            if let Ok(commit) = repo.find_commit(oid) {
                let msg = commit.message().unwrap_or("(no message)");
                let time = commit.time();
                let ts = chrono::DateTime::from_timestamp(time.seconds(), 0)
                    .map(|dt| dt.format("%m-%d %H:%M").to_string())
                    .unwrap_or_default();
                println!("  {} {}", ts, msg.trim());
            }
        }
    }
}
