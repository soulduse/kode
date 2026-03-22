# Kode — Product Requirements Document

## 비전

터미널(tmux)의 속도와 가벼움, IntelliJ의 코드 시각화와 분석 능력을 결합한 **Rust 네이티브 IDE**. Kotlin/Spring Boot 개발에 최적화된 차세대 개발 도구.

## 타겟 사용자

- tmux + vim 워크플로우를 좋아하지만 IntelliJ의 코드 탐색/리팩토링이 그리운 개발자
- Kotlin/Spring Boot 백엔드 개발자
- 빠르고 가벼운 IDE를 원하는 개발자

## 핵심 가치 제안

| 기존 문제 | Kode의 해결 |
|----------|------------|
| IntelliJ는 무겁고 느림 (JVM + Swing) | Rust + GPU 렌더링으로 네이티브 속도 |
| tmux는 코드 시각화가 불편 | GPU 가속 구문 강조, 인라인 힌트, 미니맵 |
| Zed/Lapce는 Kotlin 지원 부족 | Kotlin 컴파일러 기반 LSP로 깊은 분석 |
| Spring 통합은 IntelliJ만 가능 | 커스텀 Spring 인덱서로 빈/엔드포인트 탐색 |

## 기능 요구사항

### Phase 1: 에디터 코어
- [ ] GPU 가속 텍스트 렌더링 (wgpu)
- [ ] Rope 기반 텍스트 버퍼 (대용량 파일 지원)
- [ ] Vim 모달 키바인딩 (Normal, Insert, Visual, Command)
- [ ] Tree-sitter 기반 구문 강조 (Kotlin, YAML, Groovy)
- [ ] 파일 열기/저장
- [ ] Undo/Redo (트랜잭션 기반)
- [ ] 검색/치환

### Phase 2: tmux 기능
- [ ] 패인 분할 (수평/수직)
- [ ] 내장 터미널 (PTY 통합)
- [ ] 탭 관리
- [ ] 세션 저장/복원
- [ ] 키보드 중심 패인 제어 (Ctrl-a 프리픽스)

### Phase 3: 언어 통합
- [ ] LSP 클라이언트 (JSON-RPC over stdio)
- [ ] Kotlin LSP 서버 (Kotlin 컴파일러 API 기반)
- [ ] 자동완성, 정의 점프, 참조 찾기
- [ ] 인라인 진단 (에러/경고)
- [ ] 호버 정보 (타입, 문서)
- [ ] 인레이 힌트 (타입 추론 표시)

### Phase 4: Spring 통합
- [ ] Spring 어노테이션 인덱싱 (@Service, @Repository 등)
- [ ] 빈 의존성 그래프 시각화
- [ ] application.yml 자동완성
- [ ] REST 엔드포인트 탐색기
- [ ] Gradle 태스크 통합

### Phase 5: 플러그인 시스템
- [ ] WASM 샌드박스 (wasmtime)
- [ ] 플러그인 API (WIT 인터페이스)
- [ ] 플러그인 매니페스트 (plugin.toml)
- [ ] 플러그인 레지스트리/디스커버리

## 비기능 요구사항

### 성능 목표
| 지표 | 목표 |
|------|------|
| 시작 시간 | < 200ms (에디터만) |
| 10만 줄 파일 스크롤 | 60fps 유지 |
| 키 입력 → 화면 반영 | < 16ms (1프레임 이내) |
| Tree-sitter 재파싱 | < 5ms (증분) |
| 메모리 사용 | < 200MB (일반 프로젝트) |

### 플랫폼 지원
| 플랫폼 | 우선순위 |
|--------|---------|
| macOS (ARM) | P0 — 최우선 |
| Linux (x86_64) | P1 |
| Windows | P2 — 추후 지원 |

## 기술 스택 선정 근거

| 선택 | 이유 | 대안 및 기각 사유 |
|------|------|-----------------|
| Rust | GC 없음, 메모리 안전, 네이티브 성능 | C++ (메모리 안전성 부족), Go (GC 있음) |
| wgpu | 크로스플랫폼 GPU API, Rust 네이티브 | OpenGL (레거시), Vulkan 직접 (복잡) |
| ropey | Rope 자료구조, 대용량 파일 최적 | gap buffer (대용량 시 성능 저하) |
| cosmic-text | 텍스트 셰이핑/레이아웃, Rust 네이티브 | glyphon (유연성 부족) |
| tree-sitter | 증분 파싱, 검증된 생태계 | 자체 파서 (비현실적) |
| alacritty_terminal | 검증된 터미널 에뮬레이션 | 자체 구현 (수개월 소요) |
| wasmtime | 성숙한 WASM 런타임, 샌드박싱 | wasmer (안정성 이슈) |

## 경쟁 제품 비교

| 기능 | IntelliJ | Zed | Lapce | Helix | **Kode** |
|------|----------|-----|-------|-------|----------|
| 속도 | 느림 | 매우 빠름 | 빠름 | 매우 빠름 | **매우 빠름** |
| Kotlin 지원 | 최상 | 기본 | 기본 | 기본 | **깊은 분석** |
| Spring 통합 | 완벽 | 없음 | 없음 | 없음 | **있음** |
| tmux 워크플로우 | 없음 | 부분 | 부분 | 터미널 | **완전** |
| GPU 렌더링 | 없음 | 있음 | 있음 | 없음 | **있음** |
| 플러그인 | JVM | WASM | WASM | 없음 | **WASM** |
| 원격/SSH | 부분 | 있음 | 있음 | 있음 | **TUI 폴백** |
