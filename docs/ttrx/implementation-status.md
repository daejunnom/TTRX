# 구현 상태

기준일: 2026-09-12. 이 문서는 실행 가능한 변환기의 현재 범위와 후속 프로필을 구분한다.

## 현재 구현 프로필: source-semantic

입력 `.ttr`/`.ttrm`을 strict JSON으로 파싱하고 전용 `.ttrx` 바이너리 트리로 기록한다. 역변환은 canonical compact JSON을 만든다. 객체·배열 순서, 중복 객체 키, 숫자의 원본 lexeme, 문자열 값, 미지 필드·미지 event를 보존한다. 공백, 들여쓰기, 동등한 문자열 escape 표현은 보존하지 않는다.

Source 판별은 현대 `replay.events`, `replay.rounds[r][p].replay.events`와 문서에 남긴 구형 `data.events`, `data[r].replays[p]` 후보를 구분한다. Event stream의 각 항목은 string `type`을 가진 객체여야 하며, multi 경로가 존재하면 잘못된 round/player를 조용히 건너뛰지 않는다. 구조상 single인 `.ttrm`은 잘리거나 한 명만 남은 export를 보존하기 위한 명시적 확장자 정책으로 허용한다.

바이너리는 문자열 사전, 반복 객체 shape, 정수 varint와 원본 number fallback을 사용한다. 범용 gzip/Zstandard/Brotli payload나 원본 JSON blob을 사용하지 않는다. format profile ID가 있으므로 후속 action-compiled 형식과 구분할 수 있다. Header, payload와 거부 조건은 [TTRX 1.0 바이너리 형식](format-v1.md)에 고정했다.

이 프로필은 원본 입력 사건도 보존하므로 역변환 후 동작 의미를 임의로 근사하지 않는다. 사용자가 허용한 key timing 차이는 현재 구현에서 필요하지 않다. 대신 독립 게임 엔진이 없으므로 미노 배치 동치를 재실행해 검증했다는 주장은 하지 않는다.

## 명령과 공개 경계

- CLI: encode, decode, inspect, verify.
- Core: byte slice 기반 encode/decode/inspect_source/verify.
- WASM: byte 배열 기반 TTR/TTRM encode, TTRX decode와 inspect/verify wrapper.
- npm: `tetr-ttrx` package가 Node.js CommonJS/ESM과 browser bundler용 WASM binding을 제공한다.
- core 외부 의존성: 0개. WASM wrapper는 wasm-bindgen을 사용한다.

verify는 source→TTRX→decoded JsonValue의 완전 일치를 확인하며 source kind, stream/event 수와 크기를 출력한다. Core 반환값에는 전체 `SourceSummary`가 있고, event 종류와 option key 목록은 CLI/WASM의 inspect에서 출력한다. TETR.IO runtime trace와의 동치는 [별도 검증 범위](../tetrio/verification/coverage-and-open-items.md)다.

현재 자동화 검사와 실제 `.ttr`·`.ttrm` 표본 결과는 [변환기 검증 기록](validation.md)에 분리해 두었다.

## 다음 프로필

action-compiled 프로필은 handling을 실행해 정수 행동을 만들고 계산 가능한 상태를 생략하는 목표다. [입력·시간](../tetrio/runtime/input-and-time.md)의 호출 경계, [공급](../tetrio/runtime/board-and-supply.md)의 두 RNG·14개 bag, [대전](../tetrio/runtime/attack-and-garbage.md)의 ACK·외부 사건, [커스텀 방](../tetrio/options/custom-rooms.md)의 상수·동적 변경을 구현하고 독립 기준 trace를 확보한 뒤 활성화한다.

action-compiled profile이 준비되기 전 converter가 그 동작을 주장하거나 source-semantic 입력을 더 손실적인 형식으로 자동 선택하지 않는다.
