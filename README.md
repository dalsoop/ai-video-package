# 🎬 Action Animation Director Skill

Seedance 2.0 영상 프롬프트 + Midjourney 이미지 프롬프트를 생성하는 Claude 스킬.

## 구조

```
action-animation-director/
├── SKILL.md                  ← 마스터 (공통 규칙 + 역할 라우팅)
├── README.md
└── references/
    ├── video-cut.md          ← 영상 컷 디렉터 (Seedance 15초 프롬프트)
    ├── keycut-image.md       ← 키컷 이미지 디렉터 (썸네일/키비주얼)
    ├── character.md          ← 캐릭터 디자이너 (캐릭터 시트/레퍼런스)
    ├── background.md         ← 배경 디자이너 (로케이션/환경)
    ├── enemy.md              ← 적/몬스터 디자이너 (적/보스 디자인)
    └── storyboard.md         ← 스토리보드 기획 (전체 흐름/컷 배분)
```

## 핵심 규칙

- **영상 프롬프트 (Seedance)**: 3,000자 이내, 통합 프롬프트, `No dialogue, No BGM`
- **이미지 프롬프트 (Midjourney)**: 1,000자 이내, `--v 7` 고정
- **이미지 vs 영상 분리**: 이미지는 한 동작만, 영상은 연쇄 동작 OK
- **Phase 질감 시스템**: Phase 1(동화적) → Phase 4(극장판 보스전)
- **모든 출력에 한글 설명 + 영문 프롬프트 병행**

## 사용법

Claude 플러그인 스킬로 설치하거나, SKILL.md를 시스템 프롬프트에 포함시켜 사용.

### 트리거 예시

- "15초 액션 컷 만들어줘" → video-cut.md
- "썸네일 이미지 만들어줘" → keycut-image.md
- "새 캐릭터 추가하자" → character.md
- "이 장면 배경 필요해" → background.md
- "보스 디자인해줘" → enemy.md
- "전체 흐름 잡아줘" → storyboard.md
