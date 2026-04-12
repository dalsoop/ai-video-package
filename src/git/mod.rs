use git2::{Repository, Signature, IndexAddOption};
use crate::PROJECT_DIR;

/// GitHub org/user 이름 — 변경 시 여기만 수정
const GITHUB_OWNER: &str = "dalsoop";
/// 프로젝트 레포 접두사
const REPO_PREFIX: &str = "avp-";

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

/// GitHub 레포 생성 + remote 설정 + 첫 push
pub fn setup_remote(project_name: &str) {
    let dir = avp_dir();
    let repo_name = format!("{}{}", REPO_PREFIX, project_name);

    // gh CLI로 레포 생성
    let create = std::process::Command::new("gh")
        .args(["repo", "create", &repo_name, "--private",
               "--description", &format!("avp 프로젝트: {}", project_name)])
        .output();

    match create {
        Ok(output) if output.status.success() => {
            println!("✅ GitHub 레포 생성: {}/{}", GITHUB_OWNER, repo_name);
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("already exists") {
                println!("ℹ️  GitHub 레포 이미 존재: {}/{}", GITHUB_OWNER, repo_name);
            } else {
                eprintln!("⚠️  GitHub 레포 생성 실패: {}", stderr.trim());
                return;
            }
        }
        Err(e) => {
            eprintln!("⚠️  gh CLI 실행 실패: {} (gh 설치 필요)", e);
            return;
        }
    }

    // remote 추가
    let remote_url = format!("https://github.com/{}/{}.git", GITHUB_OWNER, repo_name);
    let repo = Repository::open(&dir).unwrap();

    // 이미 origin이 있으면 스킵
    if repo.find_remote("origin").is_err() {
        repo.remote("origin", &remote_url).unwrap();
        println!("✅ remote 설정: {}", remote_url);
    }

    // push
    push_to_remote();
}

/// 원격으로 push (git CLI 사용 — libgit2의 push는 인증이 복잡)
pub fn push_to_remote() {
    let dir = avp_dir();
    if !dir.join(".git").exists() {
        return;
    }

    // remote가 설정되어 있는지 확인
    let repo = match Repository::open(&dir) {
        Ok(r) => r,
        Err(_) => return,
    };
    if repo.find_remote("origin").is_err() {
        return; // remote 없으면 push 안 함
    }

    let result = std::process::Command::new("git")
        .args(["push", "-u", "origin", "main"])
        .current_dir(&dir)
        .stderr(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .output();

    match result {
        Ok(output) if output.status.success() => {
            // 조용히 성공
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("up-to-date") {
                // main 브랜치가 없을 수 있음 — 첫 push 시
                let _ = std::process::Command::new("git")
                    .args(["branch", "-M", "main"])
                    .current_dir(&dir)
                    .output();
                let _ = std::process::Command::new("git")
                    .args(["push", "-u", "origin", "main"])
                    .current_dir(&dir)
                    .output();
            }
        }
        Err(_) => {} // git CLI 없으면 무시
    }
}

/// 모든 변경사항을 스테이지하고 커밋 + push
pub fn auto_commit(message: &str) {
    let dir = avp_dir();
    let repo = match Repository::open(&dir) {
        Ok(r) => r,
        Err(_) => return,
    };

    let mut index = repo.index().expect("index 열기 실패");

    index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
        .expect("파일 스테이징 실패");
    index.write().expect("index 쓰기 실패");

    let tree_oid = index.write_tree().expect("tree 쓰기 실패");
    let tree = repo.find_tree(tree_oid).expect("tree 찾기 실패");

    let sig = Signature::now("avp", "avp@local").expect("서명 생성 실패");

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

    // 자동 push
    push_to_remote();
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
