mod asset;
mod cut;
mod git;
mod project;
mod prompt;

use clap::{Parser, Subcommand};

/// 바이너리 이름 상수 — 변경 시 여기만 수정
pub const BIN_NAME: &str = "avp";
/// 프로젝트 데이터 디렉토리 이름
pub const PROJECT_DIR: &str = ".avp";

#[derive(Parser)]
#[command(name = BIN_NAME)]
#[command(about = "액션 애니메이션 제작 파이프라인 CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 프로젝트 관리
    Project {
        #[command(subcommand)]
        cmd: ProjectCmd,
    },
    /// 에셋 관리 (캐릭터, 몬스터, 배경, 키컷)
    Asset {
        #[command(subcommand)]
        cmd: AssetCmd,
    },
    /// 컷 관리 (15초 영상 단위)
    Cut {
        #[command(subcommand)]
        cmd: CutCmd,
    },
    /// 프롬프트 관리 (Midjourney, Seedance)
    Prompt {
        #[command(subcommand)]
        cmd: PromptCmd,
    },
    /// 전체 상태 확인
    Status,
}

// === PROJECT ===
#[derive(Subcommand)]
enum ProjectCmd {
    /// 새 프로젝트 생성
    Init {
        /// 프로젝트 이름
        name: String,
        /// 세계관/장르
        #[arg(long)]
        genre: Option<String>,
        /// 프로젝트 유형 (숏폼/단편/극장판)
        #[arg(long)]
        r#type: Option<String>,
        /// 시작 Phase (1~4)
        #[arg(long, default_value = "2")]
        phase: u8,
    },
    /// 프로젝트 목록
    List,
    /// 프로젝트 상태
    Status {
        /// 프로젝트 이름 (생략 시 현재)
        name: Option<String>,
    },
    /// 현재 프로젝트 전환
    Use {
        /// 프로젝트 이름
        name: String,
    },
    /// Phase 변경
    Phase {
        /// Phase 번호 (1~4)
        level: u8,
    },
    /// 고정 스타일 접두사 설정/확인
    Style {
        /// 스타일 키워드 (생략 시 현재 값 표시)
        keywords: Option<String>,
    },
}

// === ASSET ===
#[derive(Subcommand)]
enum AssetCmd {
    /// 에셋 추가 (이미지 파일 등록)
    Add {
        /// 에셋 유형 (character/monster/background/keycut)
        r#type: String,
        /// 에셋 이름
        #[arg(long)]
        name: String,
        /// 이미지 파일 경로
        #[arg(long)]
        image: Option<String>,
        /// 미드저니 프롬프트
        #[arg(long)]
        prompt: Option<String>,
        /// 고정 외형 키워드
        #[arg(long)]
        keywords: Option<String>,
        /// 위협 등급 (몬스터용: fodder/medium/elite/boss)
        #[arg(long)]
        threat: Option<String>,
    },
    /// 에셋 목록
    List {
        /// 유형 필터
        r#type: Option<String>,
    },
    /// 에셋 상세 보기
    Show {
        /// 에셋 이름
        name: String,
    },
    /// 에셋 삭제
    Remove {
        /// 에셋 이름
        name: String,
    },
}

// === CUT ===
#[derive(Subcommand)]
enum CutCmd {
    /// 컷 추가
    Add {
        /// 컷 제목
        #[arg(long)]
        title: String,
        /// Phase 단계
        #[arg(long)]
        phase: Option<u8>,
        /// 장면 설명 (한글)
        #[arg(long)]
        desc: Option<String>,
    },
    /// 컷 목록
    List,
    /// 컷 상세 보기
    Show {
        /// 컷 번호
        number: u32,
    },
    /// 컷 상태 업데이트
    Done {
        /// 컷 번호
        number: u32,
        /// 완성된 영상 파일 경로
        #[arg(long)]
        video: Option<String>,
        /// 마지막 프레임 이미지 경로
        #[arg(long)]
        last_frame: Option<String>,
    },
    /// 파이프라인 단계 전진
    Advance {
        /// 컷 번호
        number: u32,
        /// 목표 단계 (keycut_done/seedance_done/frame_extracted/complete)
        #[arg(long)]
        to: Option<String>,
    },
    /// 영상에서 프레임 추출 (ffmpeg)
    Frame {
        /// 영상 파일 경로
        video: String,
        /// 컷 번호 (프레임을 저장할 컷)
        #[arg(long)]
        cut: u32,
        /// 추출 위치 (first/last, 기본: last)
        #[arg(long, default_value = "last")]
        pos: String,
    },
    /// 컷별 에셋 연결
    Link {
        /// 컷 번호
        number: u32,
        /// 에셋 이름
        asset: String,
    },
}

// === PROMPT ===
#[derive(Subcommand)]
enum PromptCmd {
    /// 프롬프트 저장
    Save {
        /// 대상 (컷 번호 또는 에셋 이름)
        target: String,
        /// 프롬프트 유형 (seedance/midjourney)
        #[arg(long)]
        r#type: String,
        /// 프롬프트 텍스트
        #[arg(long)]
        text: String,
    },
    /// 프롬프트 보기
    Show {
        /// 대상
        target: String,
    },
    /// 프롬프트 글자수 체크
    Check {
        /// 프롬프트 유형 (seedance/midjourney)
        #[arg(long)]
        r#type: String,
        /// 프롬프트 텍스트
        text: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Project { cmd } => project::run(cmd),
        Commands::Asset { cmd } => asset::run(cmd),
        Commands::Cut { cmd } => cut::run(cmd),
        Commands::Prompt { cmd } => prompt::run(cmd),
        Commands::Status => status(),
    }
}

fn status() {
    let cwd = std::env::current_dir().unwrap();
    let avp_dir = cwd.join(PROJECT_DIR);

    if !avp_dir.exists() {
        eprintln!("이 디렉토리에 {} 프로젝트가 없습니다.", BIN_NAME);
        eprintln!("  {} project init <이름> 으로 생성하세요.", BIN_NAME);
        std::process::exit(1);
    }

    project::status_current();
}
