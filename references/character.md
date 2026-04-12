# 캐릭터 디자이너 (Character Designer)

새 캐릭터의 레퍼런스 이미지와 캐릭터 시트를 미드저니로 생성한다.

---

## 캐릭터 생성 플로우

### 1단계: 정보 수집
사용자에게 AskUserQuestion으로 확인한다:
- **역할**: 주인공 / 아군 / NPC / 조력자 / 기타
- **성별/체형**: 남성/여성, 체형 키워드
- **복장 컨셉**: 전투복, 일상복, 제복, 판타지 갑옷 등
- **무기/능력**: 전투 스타일 (검술, 마법, 총기, 격투, 비전투 등)
- **성격 키워드**: 차가운/열정적/유머러스/과묵 등 (표정에 반영)
- **참고 이미지나 작품**: 있으면 스타일 참고

### 2단계: 캐릭터 시트 생성

**출력 구조:**
```
[캐릭터 시트 — 캐릭터 이름/코드명]

■ 역할: (주인공/아군/NPC 등)
■ 질감 단계: Phase X

캐릭터 설명 (한글):
(외형, 복장, 장비, 성격이 드러나는 표정/자세를 상세 서술)

Midjourney Prompt (영문):
"(1,000자 이내, --v 7)"

📏 글자수: 약 X자

고정 외형 키워드:
(이후 모든 컷에서 이 캐릭터를 묘사할 때 재사용할 핵심 키워드 리스트)
```

### 3단계: 고정 키워드 확정

캐릭터 이미지가 확정되면 **고정 외형 키워드**를 정리한다.
이 키워드는 이후 모든 영상 컷, 키컷 이미지, 다른 에셋에서 해당 캐릭터를 묘사할 때 일관되게 삽입된다.

예시:
```
sharp black short hair, dark eyes, worn brown leather aviator jacket, 
white scarf, vintage flight goggles on forehead, brown leather gloves
```

## 프롬프트 유형

**캐릭터 시트 (터너라운드 시트) — 기본**
- 정면/측면/후면 또는 정면/3/4뷰 터너라운드. 미드저니가 가장 잘 잡는 형식.
- 프롬프트 맨 앞에 **시트 타입을 짧게 선언**한다. 길게 쓰면 오히려 안 먹힌다:

```
anime character sheet, front view and side view and back view
```

또는 화풍에 따라:
```
flat vector character sheet, front view and 3/4 view
```

- **복장/장비는 괄호 묶기로 나열**한다. 문장형 서술 금지:

```
❌ Wearing a tattered olive-green travel cloak with frayed edges, brown leather buckle strap across shoulder
✅ post-apocalyptic survivor outfit (tattered olive-green cloak, frayed edges, brown leather buckle strap, dark inner clothing, worn brown boots)
```

- **스타일 키워드는 구체적으로, 금지 키워드는 최소한으로:**

```
clean line art with uniform stroke, cel shading, solid color fills, minimal shading, blank background, no text
```

- `no text` 하나면 충분. `no labels, no watermark, no annotations` 다 넣으면 오히려 노이즈.

- **프롬프트 구성 순서:**
  1. 시트 타입 선언 (짧게)
  2. 캐릭터 기본 정보 (나이, 성별, 외형 키워드)
  3. 복장/장비 (괄호 묶기)
  4. 표정/포즈 지정
  5. 스타일 키워드
  6. 배경/금지 키워드 (최소한)
  7. 파라미터 (--ar, --v 7)

- **핵심 원칙: 미드저니 프롬프트는 짧고 키워드 중심일수록 잘 먹힌다.** 설명문을 쓰지 않는다.

**단일 포즈 레퍼런스**
- 특정 상황에서의 캐릭터 모습 (전투 자세, 일상, 감정 표현 등)
- 배경 포함 가능 — 세계관 분위기 반영

**Phase 변화 비교 시트**
- 동일 캐릭터의 Phase 1~4 변화를 한 장에
- `character evolution sheet, Phase 1 to Phase 4, left to right progression`

## 주의사항

- **배경은 흰색/빈 배경** — `blank background` 또는 `white background`. 짧게.
- **텍스트 금지** — `no text` 하나면 충분. 과잉 금지 키워드는 노이즈.
- 미드저니는 텍스트(이름, 글자)를 잘 못 넣는다. 이름표 같은 건 프롬프트에 넣지 않는다.
- 캐릭터 시트에서 너무 많은 포즈(6개 이상)를 요구하면 퀄리티가 떨어진다. 2~3개가 적정.
- 무기를 들고 있는 모습이 필요하면, 무기 디자인을 먼저 확정하고 캐릭터에 반영하는 게 일관성에 좋다.
- **프롬프트가 길어질수록 미드저니가 혼란스러워한다.** 핵심만 넣고 나머지는 빼라.
