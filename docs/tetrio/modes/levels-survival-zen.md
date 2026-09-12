# 목표·레벨·Survival·Zen·튜토리얼

기준: [공식 스냅샷](../evidence/sources.md). LevelLines @914927 부근, 목표 @951004, Survival ce @965905, tutorial @970624 부근, Zen _e @976304, UpdateFromZenConfig @981828. 정적 관측이다.

## 목표·점수·레벨

objective_type 선언은 none/lines/timed지만 실제 CheckObjectiveCleared와 Custom UI는 garbage 목표도 처리한다. objective_count/time/result, score, absolute_lines를 함께 확인한다. timed 목표는 esm.frame*(1000/60)을 사용하며 화면 stopwatch나 subframe 포함 final time과 같다고 가정하지 않는다.

levels, masterlevels, startinglevel, levelspeed, levelstatic, levelstaticspeed, levelgbase, levelgspeed가 진행을 정한다. 기본 시작 레벨 1, 속도 1, static 비활성·10줄, gravity base=.8/speed=.007이다. Blitz는 levels=true, levelspeed=.42, levelgbase=.65, gravitymay20g=false 등으로 바꾼다.

LevelLines는 실제 클리어 줄 수를 먼저 반영하고 한 번에 여러 레벨을 올릴 수 있다. 중력은 base-(level-1)*speed의 하한과 pow를 사용한 계산 후 상한을 적용한다. Master는 21부터 locktime=30-min(25,level-20), 46부터 lockresets=15-min(10,level-45)로 옵션 자체를 바꾼다.

AnnounceLines는 실제 줄·레벨 통계 반영 뒤 Zenith/반전 카드의 공격 분류용 줄 수를 바꿀 수 있다. 실제 클리어 통계·점수 분류·공격 분류를 한 값으로 합치지 않는다. 점수 상수 21개는 0~5줄, mini/full spin, B2B, combo, AC, drop을 포함하고 constants override 대상이다.

## Survival

| 항목 | 엔진 기본값 | Custom UI 초기값 |
|---|---|---|
| survivalmode | none | 선택에 따라 설정 |
| survival_messiness | 0 | 100 |
| survival_layer_amt | 10 | 9 |
| survival_layer_non | false | true |
| survival_layer_min | 0 | 3 |
| survival_timer_itv | 1 | 60 |
| survival_cap | 0 | UI 선택값 확인 필요 |

layer는 gb가 있는 행을 세고 target/minimum, cap, lastcolumn, cheesespawned, rngex를 사용한다. 목표보다 행이 많으면 하단 행을 제거하는 경로도 있다. timer는 pause/sleep에서 멈추며 frame%interval===0을 검사한다.

Survival은 프레임 갱신과 Next 양쪽에서 호출된다. 프레임당 한 번으로 통합하지 않는다. stock 복구·board 변화·custom 옵션·모드 변경과의 순서를 검증한다.

## Zen 외부 설정 13개

| 키 | 소스 기본값 |
|---|---|
| leveling | on |
| spins | all-mini+ |
| combotable | multiplier |
| kickset | SRS+ |
| stride | on |
| gravitymode | relaxed |
| gravitystatic | 20 |
| counters | off |
| garbagemode | off |
| cheeselayer_height | 6 |
| cheesetimer_interval | 4 |
| cheesemessiness | 100 |
| infinite_hold | off |

소스의 사용자 설정 구조를 읽었으며 실제 사용자 localStorage를 읽지 않았다. usezenconfig는 이 외부 값을 적용하는 경로이고 zenlevels/zenlevel/zenprogress는 별도 실행 옵션·상태다.

gravitymode는 subzero, off, relaxed, engaging, spicy, static이다. 일부 모드는 cubic Bézier 진행값과 zenprogress로 중력을 계산하며 subzero는 infinite movement와 매우 큰 locktime을 적용한다.

garbagemode는 off, backfire half/full/double, unclear half/full/double, cheeselayer, cheesetimer다. 공격 후처리와 rngex 소비가 추가된다. counters=versus는 화면 카운터만 바꾸지 않고 hasgarbage도 true로 설정한다.

## Zen 세션과 replay의 경계

공식 playZen은 noreplay=true, usezenconfig=true, can_undo=true, infinite_stock=true이며 저장된 map/level/progress/score/B2B 등을 사용한다. `.ttr` 파일 지원과 Zen 세션의 모든 상태·외부 설정 재현은 동일 범위가 아니다.

ZenLevelup의 discreet 경로와 긴 연출 경로가 다르다. 긴 경로는 3100ms setTimeout에서 레벨·보드·통계를 바꾸고 10400ms에 재개한다. 이를 근거 없이 정수 frame 수로 치환하지 않는다. 어떤 외부 사건을 캡처해야 결정론적 재현이 가능한지 아직 확정하지 않았다.

Undo/Redo는 overlay와 active garbage 정리 후 Zen config 재적용을 포함한다. stock 복구는 90frame 뒤 보드·actors·hold·combo/B2B, Zen의 여러 통계·시간 기준을 바꾼다. RNG 전체를 새 게임처럼 초기화하지 않는다.

## 튜토리얼과 특수 상태

tutorial 관리자도 프레임 루프에서 실행되며 입력·진행 상태와 연결된다. tutorial 옵션을 단순 안내문으로 폐기하지 않는다. 전체 tutorial 단계별 동치 실행 자료는 아직 없다. voidhole/wound, bombs, permanent 행은 [보드·공급](../runtime/board-and-supply.md)에 정리했다.

## 검증 사례와 한계

레벨 20→21·45→46, 한 번의 다중 레벨 상승, static/dynamic 레벨, garbage 목표와 timed 경계, 실제 줄/점수/공격 분류 차이를 검증한다. Survival은 frame0·Next 중복 호출·sticky/min/cap·pause/sleep을 포함한다. Zen은 외부 config 유무, 모든 gravity/garbage mode, undo/redo, discreet/실시간 연출 경로가 대상이다.

현재 두 표본으로 위 모드 전체를 검증하지 못한다. source에 있는 동작, 파일이 제공하는 정보, 추가 세션 캡처가 필요한 동작을 별도로 표시한다.
