# Kode — 진행 상황 추적

> 상태: ⬜ 미시작 | 🔄 진행중 | ✅ 완료 | ❌ 차단됨

## Phase 0: 프로젝트 초기화

- [x] PRD.md 작성
- [x] PROGRESS.md 작성
- [x] Git 저장소 초기화
- [x] Cargo 워크스페이스 초기화
- [x] 크레이트 스텁 생성 (12개)
- [x] 기본 설정 파일 생성 (config/)

## Phase 1: 에디터 코어

### Step 2: kode-core 기본 타입
- [x] config.rs — TOML 설정 구조체
- [x] error.rs — KodeError 통합 에러
- [x] event.rs — KodeEvent 이벤트 enum
- [x] geometry.rs — Rect, Point, Size
- [x] color.rs — 테마/색상 정의

### Step 3: kode-editor 텍스트 버퍼
- [x] buffer.rs — Rope 기반 텍스트 버퍼
- [x] cursor.rs — 멀티 커서
- [x] selection.rs — Selection 범위
- [x] history.rs — Undo/Redo 트리
- [x] command.rs — EditorCommand enum
- [x] document.rs — Document 통합 구조체
- [x] search.rs — Find/Replace

### Step 4: kode-renderer GPU 렌더링
- [x] Stage A: winit 윈도우 + wgpu 서피스 (스텁 구조)
- [x] Stage B: cosmic-text 텍스트 셰이핑 + 글리프 아틀라스 (스텁 구조)
- [x] Stage C: wgpu 렌더 파이프라인 (스텁 구조)
- [ ] Stage D: WGSL 셰이더 (text.wgsl, rect.wgsl)
- [x] Stage E: 줄번호, 커서, 스크롤바 (렌더 커맨드 생성 로직)
- [x] Stage F: 컴포지터

### Step 5: kode-keymap Vim 키바인딩
- [x] mode.rs — 모드 enum
- [x] parser.rs — 키 시퀀스 파서
- [x] motion.rs — 모션 (word, line 등)
- [x] operator.rs — 오퍼레이터 (delete, yank 등)
- [x] text_object.rs — 텍스트 오브젝트
- [x] bindings.rs — 키맵 테이블

### Step 6: kode-treesitter 구문 강조
- [x] parser.rs — Parser 관리, 증분 파싱
- [x] highlight.rs — 하이라이트 쿼리
- [ ] languages/kotlin.rs — Kotlin 문법 (tree-sitter-kotlin 통합 필요)

### Step 7: kode-app 와이어링
- [x] cli.rs — clap CLI
- [x] app.rs — App 상태 머신
- [ ] event_loop.rs — tokio 이벤트 루프 (GPU 렌더링 연동 시 구현)
- [x] main.rs — 엔트리포인트

### Phase 1 마일스톤 검증
- [x] `cargo build --workspace` 성공
- [x] `cargo test --workspace` 통과 (41개 테스트)
- [ ] Kotlin 파일 열기 + 구문 강조 + vim 편집 + 저장 (GPU 렌더링 구현 후)

---

## Phase 2: tmux 기능

### Step 8: kode-workspace 패인 레이아웃
- [x] layout.rs — 이진 트리 레이아웃
- [x] pane.rs — PaneContent enum
- [x] resize.rs — 리사이즈 로직 (resize_pane, equalize_panes)
- [x] tmux 키바인딩 (Ctrl-a 프리픽스) — workspace_keys.rs + parser.rs 통합
- [x] layout.rs — 포커스 탐색 (find_pane_in_direction, find_pane_at)

### Step 9: kode-terminal 내장 터미널
- [x] pty.rs — PTY 관리 (spawn, read, write, resize)
- [x] emulator.rs — alacritty_terminal Term 래핑
- [x] grid.rs — 터미널 셀 → TerminalCell 변환 + ANSI 색상 매핑
- [x] input.rs — 키 → 이스케이프 시퀀스 (문자, Ctrl, Alt, 화살표, F키)

### Step 10: 탭/세션 관리
- [x] tab.rs — Tab 구조체
- [x] session.rs — Session 관리
- [x] persistence.rs — 세션 JSON 직렬화/역직렬화

### Step 11: kode-app 통합
- [x] app.rs — handle_workspace_action() + handle_key_event()
- [x] app.rs — 터미널 패인 생성/관리
- [x] app.rs — 패인 분할/닫기/포커스/줌
- [x] app.rs — 커맨드라인 (:w/:q/:wq)
- [x] app.rs — 세션 저장+종료 (Detach)

---

## Phase 3: 언어 통합

### Step 11: kode-lsp 클라이언트
- [ ] transport.rs — JSON-RPC 전송
- [ ] client.rs — LSP 클라이언트
- [ ] completion.rs — 자동완성
- [ ] diagnostics.rs — 진단
- [ ] hover.rs — 호버 정보
- [ ] goto.rs — 정의 점프
- [ ] code_action.rs — 코드 액션

### Step 12: Kotlin LSP 서버 (JVM)
- [ ] Gradle 프로젝트 설정
- [ ] KotlinLspServer.kt — 서버 메인
- [ ] KotlinAnalyzer.kt — 컴파일러 API
- [ ] CompletionProvider.kt
- [ ] DiagnosticProvider.kt
- [ ] NavigationProvider.kt
- [ ] HoverProvider.kt

---

## Phase 4: Spring 통합

### Step 13: Spring 인텔리전스
- [ ] SpringIndexer.kt — 어노테이션 인덱싱
- [ ] BeanGraphBuilder.kt — 빈 그래프
- [ ] YamlCompleter.kt — YAML 자동완성
- [ ] 엔드포인트 인덱싱
- [ ] graph.rs — 빈 그래프 시각화 (Rust)
- [ ] GradleConnector.kt — Gradle 연결
- [ ] TaskRunner.kt — 태스크 실행

---

## Phase 5: 플러그인 시스템

### Step 14: WASM 플러그인
- [ ] kode.wit — WIT 인터페이스
- [ ] host.rs — wasmtime 호스트
- [ ] manifest.rs — plugin.toml
- [ ] registry.rs — 플러그인 디스커버리
- [ ] 예시: Git blame 플러그인
- [ ] 예시: Bracket colorizer 플러그인
- [ ] 예시: TODO highlighter 플러그인

---

## 결정 로그

| 날짜 | 결정 | 사유 |
|------|------|------|
| 2026-03-22 | Strategy A 채택 (Rust 셸 + JVM LSP) | Kotlin 분석을 직접 구현하는 것은 비현실적, 컴파일러 API 재사용 |
| 2026-03-22 | fwcd/kotlin-language-server 포크 | 처음부터 만들기보다 기존 구현 확장이 효율적 |
| 2026-03-22 | cosmic-text 선택 (glyphon 대신) | 에디터 특화 텍스트 레이아웃에 더 유연 |
| 2026-03-22 | alacritty_terminal 사용 | 터미널 에뮬레이터 직접 구현은 수개월 소요 |
