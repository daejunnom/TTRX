# 검증 범위와 미해결 계약

기준일: 2026-09-12. 문서·소스 정적 감사·표본 구조 관측에 더해 원본 JSON tree를 보존하는 `source-semantic` Rust 코덱·CLI·WASM을 구현하고 실제 두 표본으로 검증했다. 아래 V01-V14와 O01-O11은 주로 후속 `action-compiled` 게임 실행에 필요한 작업이며 통과 목록이 아니다. 현재 코덱 결과는 [별도 검증 기록](../../ttrx/validation.md)에 있다.

## 현재 근거로 주장할 수 있는 범위

| 대상 | 현재 증거 | 아직 주장할 수 없는 것 |
|---|---|---|
| 공식 소스 | 고정 SHA와 AST 629,233 노드 | 최신 서버·과거 전체 빌드와 동치 |
| OptionsList | 173개·13분류·누락/중복 0 | 모든 값·조합 실행 성공 |
| 초기화·소비 | 구간별 정의·읽기·쓰기·호출 경로 | 모든 동적 별칭과 잘못된 입력의 완전 의미 분석 |
| 프리셋 | 10개 명령 설정 추출 | 서버가 허용하는 모든 커스텀 규칙 |
| 표본 | 두 SHA·7,539 사건·57개 명시 옵션 | 비기본 옵션·모드 전체 지원 |
| 상태·binary | snapshot/codec 분기 차이 확인 | 완전 checkpoint 및 원본 lossless 보증 |
| TTRX source-semantic | Rust 코덱·CLI·WASM, data-model 왕복, 실제 두 표본, checksum·구조 실패 경로 | 게임 규칙 실행 및 독립 placement trace 동치 |
| TTRX action-compiled | 요구·경계·검증 사례 문서화 | Rust 게임 엔진·행동 코덱·전체 옵션 실행 완료 |

원본 보존, 규칙 식별, 초기 정보 충분성, 실행 동치, 자원 안전, 성능을 별도 축으로 보고한다. 이름 없는 커스텀 규칙도 이 판정에 포함하며 프리셋 미일치만으로 미지원 처리하지 않는다.

## 실행 검증 범위

| ID | 범위 | 필요한 사례·관측점 | 관련 문서 |
|---|---|---|---|
| V01 | 옵션 | 173개 모든 처리 경로, 생략/명시 기본값, 타입·범위·enum 밖 값, 초기와 동적 변경 차이 | [초기화](../options/initialization.md) |
| V02 | 입력 | handling 9개, room override, ARR0·hoisted·양방향·sleep·실패 입력·IRS/IHS | [입력](../runtime/input-and-time.md) |
| V03 | 시간·수치 | event partition, lock 경계, 역순 예약, fractional 값, JS round/mod/pow/log1p | [입력](../runtime/input-and-time.md) |
| V04 | 공급·RNG | 14종 bag, refill·map·no_szo·fractional seed, 두 RNG, Zenith 공급 | [공급](../runtime/board-and-supply.md) |
| V05 | 보드·회전 | 8종 킥·13종 piece, custom matrix·colors·skip, 크기·map 실패·resize | [보드](../runtime/board-and-supply.md) |
| V06 | 공격 | spin10·combo4, B2B chain/charge/extras, 분할 surge·AC·cap 순서 | [공격](../runtime/attack-and-garbage.md) |
| V07 | 쓰레기·ACK | entry3×blocking3, passthrough4, 확인 전 취소, 큐 우선순위·cap·messiness RNG | [쓰레기](../runtime/attack-and-garbage.md) |
| V08 | 외부·custom | 11종 custom, 같은 frame 사건 순서, 상수·미노 교체, 다중 frame·ID | [사건](../runtime/events-and-state.md) |
| V09 | 수명·복원 | stock90frame, undo/redo, retry·forfeit·clear, snapshot/overlay/inject 차이 | [상태](../runtime/events-and-state.md) |
| V10 | 기본 모드 | garbage/timed 목표, Master21/46, Survival layer/timer·Next, tutorial | [레벨·Survival](../modes/levels-survival-zen.md) |
| V11 | Zen | config13, gravity·garbage 모드, 외부값 유무, 실시간/discreet levelup | [Zen](../modes/levels-survival-zen.md) |
| V12 | Zenith | 정상9·반전9·조합, 층·피로·공급 임계, Duo 무력화·12frame 부활·외부5사건 | [Zenith](../modes/zenith.md) |
| V13 | 커스텀 방 | 프리셋 없음·수정, 같은 이름 다른 값, 별도 room.constants, 옵션 간 조합 | [커스텀 방](../options/custom-rooms.md) |
| V14 | 매치 | versus/royale/practice, 다인·다라운드·부분 참가자, FT/WB/GP·서버 결과 | [매치](../modes/presets-and-match.md) |
| V15 | 원본·형식 | 현재 profile은 정밀 수치·부호 있는0·필드 유무·중복 키·미지 사건을 합성 검사했고 두 현대 표본을 왕복했다. 구형 구조·후속 행동 binary 차이는 남음 | [리플레이](../replay/source-and-binary.md) |
| V16 | 자원·측정 | 현재 profile은 절단·길이·참조·중첩·반복·출력 한도와 크기를 검사한다. action tick·replay 성능·더 넓은 적대 입력 측정은 남음 | [TTRX 요구](../../ttrx/requirements.md) |

옵션 단독 테스트만으로 결합을 증명하지 않는다. 모든 조합을 무작정 곱한 전수 실행 대신, 실제 의존 관계별 교차 사례와 경계값을 명시하고 남은 조합의 미검증 상태를 유지한다. 합성 자료와 실제 서버 승인 경기 자료도 구분한다.

## 미해결 항목

| ID | 미해결 계약 | 필요한 추가 근거 |
|---|---|---|
| O01 | 서버 room.setconfig 검증·자유 index·프리셋 외 실제 허용 | 서버 명세 또는 승인된 경기 설정·관측 |
| O02 | room.constants 생성·초기 상수의 export 누락 | 당시 룸 상수·캡처·신뢰 가능한 초기 상태 |
| O03 | 카드·Practice·추가 설정의 서버 확장 순서 | 실제 초기 options/constants bundle |
| O04 | 타깃 선정·매치 승자·서버 passthrough | 기록된 target/result/ACK와 서버 계약 |
| O05 | Zen 외부 config·실시간 setTimeout | 캡처해야 할 입력·시각·상태와 기준 실행 |
| O06 | seed_random/TEMP_zenith_rng 파생값 | 선택된 seed·APM·gravity·flags 등 초기 근거 |
| O07 | 과거 규칙·구형 source schema | 버전 고정 소스·실제 파일·전이 trace |
| O08 | checkpoint 충분성 | omitted module/callback/partner 상태의 영향 검증 |
| O09 | binary custom payload와 permissive 값 | 독립 원본 모델·실행 trace·실제 codec 왕복 |
| O10 | snowman/5mblast 활성 여부, noextrawidth 소비 | 생성·실행 경로 또는 동시대 자료 |
| O11 | Rust/native/WASM 수치 동치 | 같은 입력·단계의 수치 및 상태 비교 |

미확인을 기본 규칙 대입이나 메타데이터 삭제로 해결하지 않는다. 동시에 외부 근거가 부족하다는 이유로 이미 확인한 클라이언트 기능 전체를 범위에서 제외하지 않는다.

## 기준 실행과 완료 판단

기준 실행은 소스 해시·규칙 버전·초기 상수·옵션·실행 문맥·표본 SHA를 기록해야 한다. server/headless/local/replay 문맥을 임의로 교환하지 않는다. board/falling/hold/queue 외 RNG·pending/ACK·waiting·actors·통계·모드 상태를 의미 있는 관측점에서 비교한다.

비트 코덱 왕복, 원본 의미 왕복, 공식 기준 실행 동치를 각각 검사한다. 인코더와 디코더가 같은 버그를 공유하거나 최종 보드만 같은 것으로는 충분하지 않다. 성능은 원본 총합 대비 크기·encode/decode/replay/restore 시간·메모리를 구분하고 미지원·가설·손실 결과를 성공 통계에 섞지 않는다.

## 문서 자체의 검증

문서 작성 완료 때 하위 디렉터리 배치, 로컬 링크, UTF-8/Markdown 구조, JSON 파싱, 173개 분류와 catalog 일치, 프리셋10개 및 표본57/116 집계, 출처 해시 일치, 커스텀 방·초기 상수·동적 변경·미확인 경계의 문서 포함 여부를 검사한다. 이 검사 결과는 게임 실행 검증과 별도인 [문서 검증 기록](documentation-check.md)에 남긴다.
