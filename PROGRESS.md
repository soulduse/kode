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
- [x] jsonrpc.rs — JSON-RPC 2.0 직렬화/역직렬화, Content-Length 프레이밍
- [x] transport.rs — LSP 서버 프로세스 spawn, tokio 비동기 stdio 통신
- [x] client.rs — LSP 클라이언트 (initialize, shutdown, didOpen/Change/Close/Save)
- [x] capabilities.rs — 서버 기능 확인
- [x] completion.rs — 자동완성 (CompletionList/Array 파싱)
- [x] diagnostics.rs — DiagnosticStore (문서별 진단 관리)
- [x] hover.rs — 호버 정보 (MarkedString/MarkupContent 추출)
- [x] goto.rs — 정의 점프, 구현 점프, 참조 찾기
- [x] code_action.rs — 코드 액션 + WorkspaceEdit 수집
- [x] symbols.rs — 문서/워크스페이스 심볼 (중첩 → 플랫 변환)
- [x] manager.rs — LspManager 다중 서버 관리 (언어별 자동 시작)
- [x] event.rs — KodeEvent에 LspEvent 추가
- [x] app.rs — LspManager 통합

### Step 12: Kotlin LSP 서버 (JVM)
- [x] Gradle 프로젝트 설정 (kotlin-lsp/, shadow jar, JDK 17)
- [x] KodeLspServer.kt — LSP 서버 메인 (lsp4j LanguageServer)
- [x] KodeTextDocumentService.kt — textDocument/* 요청 프록시
- [x] CustomMethodHandler.kt — spring/* 커스텀 메서드 라우팅
- [x] ProcessProxy.kt — fwcd/kotlin-language-server 사이드카 프록시

---

## Phase 4: Spring 통합

### Step 13: Spring 인텔리전스
- [x] SpringIndexer.kt — 어노테이션 인덱싱 (@Service, @Controller, @Bean 등)
- [x] BeanRegistry.kt — 빈 인메모리 저장소
- [x] EndpointIndexer.kt — REST 엔드포인트 스캔 (@GetMapping 등)
- [x] BeanGraphBuilder.kt — 빈 의존성 그래프 (노드/엣지 + 순환 감지)
- [x] YamlCompleter.kt — application.yml 자동완성 (20+ 기본 프로퍼티)
- [x] ConfigMetadataParser.kt — spring-configuration-metadata.json 파서
- [x] graph.rs — 빈 그래프 시각화 (find_dependents, detect_cycles, render_tree)
- [x] GradleConnector.kt — Gradle Tooling API 연결 + 태스크 조회
- [x] TaskRunner.kt — Gradle 태스크 실행 + 출력 스트리밍
- [x] kode-spring 크레이트 — Rust 측 타입(SpringBean, RestEndpoint, BeanGraph) + LSP 클라이언트 래퍼
- [x] SpringEvent — KodeEvent에 Spring 이벤트 (BeansReady, EndpointsReady 등) 추가
- [x] App 통합 — :beans, :endpoints, :gradle 커맨드 + PaneContent 확장

---

## Phase 5: 플러그인 시스템

### Step 14: WASM 플러그인 런타임
- [x] abi.rs — 플러그인 ABI 타입 (PluginEvent, PluginResponse, Decoration)
- [x] manifest.rs — plugin.toml 매니페스트 파싱 (이벤트 구독, 리소스 제한)
- [x] host.rs — wasmtime 호스트 (fuel 메터링, StoreLimits, WASI 샌드박스, 호스트 임포트)
- [x] registry.rs — PluginManager (디스커버리, 디스패치, 데코레이션 캐시)
- [x] event_bridge.rs — KodeEvent → PluginEvent 매핑
- [x] PluginCoreEvent — KodeEvent에 Plugin 이벤트 추가
- [x] App 통합 — :plugins, :plugin-enable, :plugin-disable 커맨드

### Step 15: Plugin SDK + 예시 플러그인
- [x] kode-plugin-sdk — 플러그인 공통 SDK (alloc/dealloc, 호스트 임포트 래퍼)
- [x] todo-highlighter — TODO/FIXME/HACK/XXX 하이라이팅
- [x] bracket-colorizer — 깊이별 무지개 괄호 색상화
- [x] git-blame — 커서 줄 blame 주석 (placeholder)

---

## Phase 6: TUI 렌더링

### Step 16: TUI 기반 렌더링
- [x] event.rs — crossterm → KodeEvent 변환 (키보드, 마우스)
- [x] colors.rs — kode Color → ratatui Color, ThemeStyles 매핑
- [x] editor_view.rs — 에디터 렌더링 (거터, 텍스트, 커서, 스크롤바, 줄 하이라이트)
- [x] terminal_view.rs — alacritty 그리드 → ratatui 셀 렌더링
- [x] chrome.rs — 탭 바, 상태 줄, 패인 테두리
- [x] ui.rs — 프레임 렌더링 오케스트레이션
- [x] event_loop.rs — raw mode + poll 루프 + 터미널 키 전달
- [x] app_bridge — TUI용 앱 상태 (에디터 명령, 모션, 워크스페이스 액션 처리)
- [x] main.rs — --tui 플래그 진입점

---

## 결정 로그

| 날짜 | 결정 | 사유 |
|------|------|------|
| 2026-03-22 | Strategy A 채택 (Rust 셸 + JVM LSP) | Kotlin 분석을 직접 구현하는 것은 비현실적, 컴파일러 API 재사용 |
| 2026-03-22 | fwcd/kotlin-language-server 포크 | 처음부터 만들기보다 기존 구현 확장이 효율적 |
| 2026-03-22 | cosmic-text 선택 (glyphon 대신) | 에디터 특화 텍스트 레이아웃에 더 유연 |
| 2026-03-22 | alacritty_terminal 사용 | 터미널 에뮬레이터 직접 구현은 수개월 소요 |
| 2026-03-22 | 함수 기반 ABI (WIT 대신) | 개인 프로젝트에 WIT Component Model은 과도, JSON 교환이 디버깅 용이 |
| 2026-03-22 | wasmtime 28 + fuel 메터링 | 플러그인 타임아웃/메모리 제한을 위한 샌드박스 |
| 2026-03-22 | ratatui (raw crossterm 대신) | diff 기반 렌더링, Widget 시스템이 패인 레이아웃에 자연스러움 |
| 2026-03-22 | RenderCommand 우회 | GPU용 float 좌표를 TUI 셀로 변환은 비효율적, App 상태에서 직접 렌더링 |
