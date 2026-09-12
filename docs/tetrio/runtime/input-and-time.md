# 입력·물리·시간 진행

기준: [공식 스냅샷](../evidence/sources.md). 주요 위치: R @858363, 내부 고정 @861259, 내부 낙하 @861513, Fall @864175, j @905523, ProcessSubframe @909736, 프레임 루프 @938038, 대기 사건 @975688/@975781. 정적 분석이며 차등 실행은 미수행이다.

## 입력 표현과 상태

논리 키는 moveLeft, moveRight, rotateCCW, rotateCW, rotate180, softDrop, hardDrop, hold, retry, exit, undo, redo의 12개다. 기본 keybind에는 앞의 10개가 있고 undo/redo는 별도 Ctrl+Z/Y 경로에서 들어온다. replay에는 keydown/keyup으로 기록하며 키와 이벤트 종류를 혼동하지 않는다.

handling 9개: arr, das, dcd, sdf, safelock, cancel, may20g, irs, ihs. 초기 값 선택·clamp·버전별 처리·room override는 [초기화](../options/initialization.md)에 기록했다.

| 상태·정책 | 실제 소비 |
|---|---|
| arr/das/dcd/cancel | 양쪽 held, DAS/ARR 누적, 마지막 방향, 벽 접촉 상태 |
| hoisted | keydown의 DAS 시작값을 das-dcd와 연결 |
| irs/ihs | sleep 중 tap buffer 또는 held 키를 다음 미노에서 소비 |
| sdf/may20g | 중력·보드 높이·gravitymay20g와 함께 낙하·즉시 slam 판단 |
| safelock | 자동 고정 후 harddrop 억제와 호출 단위 감소 |
| held 좌우/softdrop | 입력 처리 외 Zenith 과제·통계에서도 소비 |
| 키 시도·실패 | 입력·키·finesse 통계와 lock reset 등 부수 효과 |

막힌 이동, 잠긴 hold, 잠든 미노의 회전, safelock에 막힌 harddrop은 성공한 행동과 같은 부수 효과가 아니다. 최종 좌표와 성공한 조작만 남기면 원본 실행 의미가 부족할 수 있다.

## 물리 옵션

- g, gincrease, gmargin, gravitymay20g: 초기 S.g와 이후 중력 진행, 20G 정책.
- are, lineclear_are, locktime, lockresets, infinite_movement: 고정·이동·회전·낙하·대기 큐.
- allow_harddrop, allow180, infinite_hold, display_hold: 실제 행동 허용.
- clutch, nolockout, topoutisclear: 고정·스폰·강제 이동의 종료 조건.
- boardwidth/height/buffer, kickset, inverted: 충돌·스폰·회전·좌우 입력 해석.

Is20G는 고정 높이 20만으로 판단하지 않고 보드 높이·중력·handling을 함께 사용한다. 원본 회전의 추가 offset, ARS/NRS 스폰, O 킥 등은 [보드·공급](board-and-supply.md)을 참조한다.

## subframe과 프레임 단계

ProcessSubframe은 next<=현재 subframe이면 반환한다. 그 외 delta=next-current를 계산하고 ProcessAllShift(delta), Fall(delta), subframe 갱신 순으로 처리한다.

프레임 루프는 subframe을 초기화하고 이벤트를 Pull한 뒤 esm.AdvanceFrame을 수행한다. 그 후 남은 shift/fall, interrupt, 예약 작업, garbage ARE, 목표 검사, 공격 지연 처리, 중력·garbage 증가, Survival, 통계, actors, tutorial, Zenith를 진행한다. replay 기록과 full 생성은 별도 마지막 구간에 있다.

이벤트가 기록된 frame, esm이 진행한 frame, 프레임 종료 단계가 읽는 값의 관계를 보존해야 한다. 입력 timestamp와 최종 game time을 하나로 합치지 않는다.

## 같은 총 시간이어도 분할 방식에 따라 결과가 달라질 수 있다

| 원본 동작 | 잘못 바꾸기 쉬운 가정 |
|---|---|
| Fall은 호출마다 safelock을 1 감소하며 pause/sleep 검사보다 앞서 수행 | delta만큼 비례 감소 |
| v13 이상 일반 SDF 최소 낙하량은 호출별 .05*sdf | 모든 항에 delta를 다시 곱함 |
| 내부 낙하는 1e-6 단위 Math.round와 정수 경계 보정을 사용 | 다른 반올림·격자 반올림으로 대체 |
| 고정 조건은 locking>locktime | >=로 변경 |
| v15 이상 lock timer는 delta를 더함 | 매 호출 1을 더함 |
| 예약 target===현재 frame 조건, 배열 역순 실행 | 기한이 지난 항목 처리 또는 FIFO 정렬 |
| are/lineclear_are 등 모든 number가 integer는 아님 | 모든 예약 시간을 정수 frame으로 clamp |

표본의 0.1 subframe과 기존 key codec의 4bit/10 표현은 관측 사실이다. 그것이 엔진을 프레임당 10개의 균등 physics tick으로 실행할 근거는 아니다. 정수 시간 인코딩, 호출 경계, 원본 사건 순서와 프레임 단계를 따로 계약화해야 한다.

## 확인한 버전 분기

| 경계 | 차이 |
|---|---|
| 12 | keydown ARR 초기화, safelock 활성 조건 |
| 13 | softdrop 최소 속도 |
| 15 | lock timer의 subframe 반영, DAS를 통과한 구간의 ARR 누적, 무한 SDF sentinel 21→41과 낙하량 20→400 |
| 17 | 20G 스폰·이동 slam, ARS I 킥, harddrop 중 softdrop 점수 처리 |
| 18 | gravitymay20g와 handling.may20g 정책 |

이 표는 현재 소스에 남은 분기다. 과거 서버·클라이언트 빌드 전체의 동치 증거는 아니다. v19부터 구현하더라도 생략된 version 기본값 11을 임의로 19로 바꾸면 안 된다.

## TTRX 행동 컴파일에 반영할 점

ARR/DAS/DCD의 반복 발생 시점과 버퍼 실행 시점을 인코더가 선계산하는 요구는 유지한다. 행동 계약에는 이동 시도, 회전·hold의 실제 실행, 입력 시도 통계, 의미 있는 held 상태 효과와 시간 분할을 어떻게 전달할지 명시해야 한다. 이것을 원시 key replay를 그대로 실행하는 디코더로 바꾸거나, 통계를 임의 state_patch로 고치는 것으로 대체하지 않는다.

SDF/20G 같은 물리 정책과 입력에서 컴파일 가능한 상태의 경계는 차등 실행으로 검증해야 한다. 단순 opcode 후보는 아직 최종 규격이 아니다.

## 검증 사례

- ARR=0, 양방향 동시 입력, cancel, hoisted, 벽 접촉 후 DCD, 같은 tick의 반복 입력.
- IRS/IHS off/hold/tap, sleep 중 회전·hold, allow180와 display_hold/infinite_hold 조합.
- 같은 총 시간의 다른 event partition, zero-length remainder, safelock 호출 횟수, SDF 하한.
- locktime의 같음/초과, lockresets·rotresets 포화, infinite_movement, 작은 보드에서 20G.
- 원본 순서를 바꿨을 때 차이가 드러나는 같은 frame의 예약 사건·입력·custom 사건.
- 통계·RNG·waiting queue·입력 상태를 포함한 관측점 비교. 최종 보드만 비교하지 않음.
