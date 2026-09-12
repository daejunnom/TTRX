# 외부 사건·상태·종료·되돌리기

기준: [공식 스냅샷](../evidence/sources.md). IGE @876402, custom @878936 부근, Snapshot @960300, EjectState @961033, stock @964017, 대기 사건 @975688. 정적 관측이다.

## 사건 계층

전용 replay codec의 8개 분기는 keydown, keyup, start, full, end, ige, strategy, manual_target이다. 미지 type의 일반 pack/unpack 경로도 있다. engine의 직접 esm.On 등록은 keydown/keyup/ige/strategy/manual_target 5종으로 확인했다. start/full/end를 직접 상태 주입으로 처리하는 listener는 찾지 못했다.

IGEAction의 소비는 interaction, interaction_confirm, target, targeted, allow_targeting, kev, custom이다. interaction_confirm에는 garbage 외 Zenith climb_pts/bonus/incapacitated/revive/attack이 있다. 각 계층의 frame과 참가자 ID를 보존한다.

## custom 사건 11종

| 종류 | 상태 전이와 주의점 |
|---|---|
| garbage | pending 공격 생성; hardened/queue/delay/column/size/색·actor 등 |
| map | board?queue?hold 로딩과 공급·RNG·현재 미노 처리 |
| queue | 공급 queue 변경. 공급 내부 상태 전체의 초기화와 구분 |
| piece | 현재 미노 변경. 형상·위치·충돌과 상태 처리 |
| lines | 직접 행 조작; column 배열·positive/negative cell·position·actor |
| boardsize | 크기 설정과 보드 초기화·bag 복귀·ARE 등 |
| boardresize | 보존하며 크기를 바꾸는 별도 경로. boardsize와 합치지 않음 |
| holderstate | 홀드 및 허용 상태 변경 |
| setoptions | 옵션 직접 할당과 일부 키의 부수 효과; 초기 validator 재실행 아님 |
| constants | root 게임 상수 병합·reload |
| tetrominoes | minotypes/matrix/색 교체, hold/queue 필터, 필요시 30frame 대기 후 respawn |

이는 클라이언트 처리 능력의 목록이다. 모든 사용자가 모든 일반 방에서 이 사건을 생성할 수 있는지 또는 임의 payload가 서버에서 승인되는지는 별도다. [커스텀 방](../options/custom-rooms.md)에서 그 경계를 명시했다.

## Snapshot은 완전한 재시작 상태가 아니다

Snapshot의 구성은 game.board, bag, hold(piece/locked), g, controlling(lShift/rShift/lastshift/inputSoftdrop), falling, handling, playing, stats, diyusi다.

rng/rngex, bagid/bagex/lastGenerated, setoptions, constants overrides, actors, waitingqueues, stock, 전체 Zenith 상태, held 회전·hold 입력은 포함되지 않는다. 따라서 full은 기록·색인용 부분 snapshot이며 존재만으로 state_patch나 재시작 지점으로 해석하지 않는다.

| 인터페이스 | 의미 |
|---|---|
| EjectState | S 대부분을 deep copy, RNG를 seed로 표현, unsafe waiting callback 제외, frame·overrides 포함 |
| EjectOverlayState | undo용 game·overrides; otherstates 제거, frame 없음 |
| OverlayState | can_undo일 때 현재 controls/shift/targets/enemies 등 지정 상태를 유지하며 overlay |
| InjectState | checkpoint 주입. public wrapper는 esm.Seek와 replay cursor 이동도 수행 |
| EjectBoardState | 관전용 board 일부·fire/garbage/크기 표현 |

복원에는 RNG 재생성, constants reload, Resize, collision memo 제거, seen IGE 초기화, Duo partner 연결 해제 등 부수 효과가 있다. EjectState도 브라우저와 모듈의 모든 상태를 포괄하는 일반 직렬화 표준은 아니다. TTRX checkpoint는 필요한 상태가 충분한지 독립적으로 검증해야 한다.

## 예약 큐와 상태 순서

WaitFrames는 target을 계산해 저장하며 ExecuteWaitingFrames는 target===현재 frame인 항목을 역순으로 실행한다. 같은 frame의 사건을 FIFO로 정렬하거나 <=로 처리하지 않는다. fractional delay, 새 예약이 실행 중 추가되는 경우, pause/sleep과 frame 기준이 검증 대상이다.

source_residual은 원본 복원을 위한 정보이고 실제 상태를 임의 보정하지 않는다. 서버나 원본의 실제 보정 사건을 state_patch로 표현하려면 적용 시점·단계·범위가 명시돼야 한다. 계산 통계가 다르다는 이유로 엔진 상태를 덮어써 동치처럼 만들지 않는다.

## stock·undo·retry·종료

- stock loss는 pause/sleep 뒤 90frame 예약 복구다. board·actors·pending garbage·combo/B2B·hold 등을 선택적으로 초기화하지만 RNG·공급 전체를 새 게임처럼 초기화하지 않는다.
- Zen이면 통계와 시간 기준의 추가 초기화가 있고, Zenith Duo는 stock 경로 대신 무력화로 들어간다.
- undo/redo는 overlay를 적용하며 현재 입력·타깃 등을 유지하고 active garbage를 남기는 처리, Zen config 재적용을 포함한다.
- retry/exit/undo/redo는 keydown.data.key다. 별도 event type으로 재분류해 원본을 잃지 않는다.
- forfeit_time 기본 60과 stride의 /3 처리, can_retry, retryisclear, topoutisclear, pro_retry 조건을 구분한다.
- gameover에는 disconnect, topout, garbagesmash, forfeit, clear, topout_clear, winner, retry 등의 이유가 있다.

개별 보드의 종료 사유, replay end에 기록된 결과, 서버의 라운드·매치 승자는 같은 데이터가 아니다. 부분 참가자만 있는 replay에서 로컬 topout만으로 모든 승자·순위를 다시 만들지 않는다.

## 검증 사례

custom 11종의 단독·동일 frame 순서 조합, 초기 설정과 setoptions 차이, constants/tetrominoes 전후 공급·충돌·통계, snapshot에 없는 상태가 영향을 주는 재개, overlay와 full inject 차이, stock 90frame·voidhole 30frame·Duo 12frame 경계, retry/forfeit/clear 분기, 배열 순서가 다른 다중 라운드 참가자 연결을 포함한다.
