# 옵션 초기화와 경기 중 변경

기준: [공식 스냅샷](../evidence/sources.md). 주요 위치: OptionsList @1016979, ValidateOptions @1023459, SetOptions @1024655, SetConstants @1029349, StartGame @1029428. 모든 결과는 정적 관측이다.

## 구분할 설정 계층

1. 원본 replay options: 필드 존재 여부와 원래 값을 보존하는 입력.
2. 엔진 OptionsTemplate: 173개 선언 기본값. version 기본값은 11이다.
3. 사용자·모드·서버가 공급한 값: handling fallback, 룸 상수, 추가 모드 설정 등.
4. 초기 effective options와 gameOptions: 검증·정규화 후 실행 옵션과 replay에 보낼 비기본값 중심 객체.
5. derived state: S.g, S.handling, S.stock, RNG, 보드 치수 등 별도로 복사·계산한 상태.
6. 경기 중 옵션과 상태: custom 사건 및 모드 로직이 바꾼 결과.
7. 뷰어용 FormatOptions: 현재 뷰어 환경이 원본 옵션에서 만드는 표시·재생 설정.

이 계층을 하나의 정규화 객체로 합치면 생략된 원본 값이나 초기화 순서를 잃는다. 전체 값 목록은 [catalog](catalog.md), 프리셋 밖 조합은 [custom rooms](custom-rooms.md)를 참조한다.

## 초기 validator의 실제 동작

- 키가 선언에 없으면 deprecated 키는 건너뛴다. 그 외 키는 production에서 경고 후 건너뛰고 비production에서는 오류를 발생시킨다.
- deprecated 키는 garbagequadbonus, garbagespinbonus, garbageaspinbonus다.
- 값의 타입은 선언 기본값과 `typeof`로 비교한다. 배열·null을 세밀하게 검증하는 스키마가 아니다.
- strict 범위 위반은 오류다. gameid에는 0..8192 strict 범위가 있다.
- 일반 범위 보정은 `min || max`가 참일 때만 적용한다. 따라서 min=0만 있고 max가 없는 선언의 하한 보정이 실행되지 않는 경우가 있다.
- integer 플래그가 있으면 Math.floor를 사용한다. 모든 number 옵션에 정수화를 적용하는 것은 아니다.
- allowed/possibles는 강제하지 않는다. 선언 목록과 실행 가능한 값이 다를 수 있다.
- static 항목 또는 기본값과 다른 값만 generator에서 내보낸다. gameOptions는 전체 setoptions와 다르다.

원본보다 강한 검증이나 버그 수정은 호환 처리와 분리해야 한다. 코덱의 길이·자원 검증을 하면서 게임 옵션을 임의로 바꾸지는 않는다. 선언 수치와 실행 검증이 같다고 가정하지 않는다.

## 초기 적용 순서

공개 setGame은 상수 override를 먼저 적용하고 SetOptions를 호출한 뒤, 생성된 gameOptions를 replay 관리자에 전달한다. SetOptions 안에서는 다음 순서가 관측됐다.

1. TEMP_zenith_rng의 비결정적 파생값, TEMP_zenith_grace의 targeting grace를 준비한다.
2. handling 입력을 선택하고 버전별 정규화를 수행한다.
3. seed_random, 생략 gameid 등 초기 값을 처리한다. objective_count=null과 shielded=false에는 별도 호환 처리가 있다.
4. ValidateOptions 결과를 gameOptions와 S.setoptions에 반영한다.
5. 표시·모드 관련 초기 부수 효과를 수행한다.
6. S.g, S.stock, 초기 score를 복사하고 rng와 rngex를 같은 seed에서 각각 만든다.
7. Zen 상태, Resize, SetupBoard, LoadMap을 적용한다.
8. room_handling으로 ARR·DAS·SDF를 다시 덮어쓴다.
9. 초기 레벨·중력을 계산하고 PopulateBag 및 no_szo를 처리한다.
10. voidhole actors, Survival 초기 쓰레기, latency·undo, 반전 모드 초기 보드 등을 처리한다.

map 로딩이 공급과 RNG를 소비할 수 있으므로 공급 생성 순서를 편의상 앞으로 옮기면 안 된다. [board and supply](../runtime/board-and-supply.md)에 세부 상태가 있다.

## handling 정규화

원본 handling 객체에 das 키가 있으면 그 객체를 선택한다. 없으면 사용자 전역 ot.handling, 해당 환경에서 없다면 빈 객체 경로를 사용한다. 부분 객체를 모든 사용자 기본값과 일반적으로 병합하는 함수가 아니다.

| 필드 | 초기 정규화 | 추가 의미 |
|---|---|---|
| arr | 0..5, 0.1 단위 내림 | 0은 즉시 반복 이동 경로 |
| das | 1..20, 0.1 단위 내림 | hoisted와 DCD에 연결 |
| dcd | 누락/falsy는 0, 0..20, 0.1 단위 내림 | 사용자 기본 객체 2, UI reset 1과 구분 |
| sdf | 5..41, 정수 내림 | 버전별 무한 SDF와 최소 낙하량 |
| safelock | version>=12이고 명시 false가 아니면 활성 | 호출 횟수와 harddrop 허용 |
| cancel | boolean 강제 변환 | 양방향 입력 반복 |
| may20g | version<18이면 true, 그 이상 명시 false 여부 | gravitymay20g와 별도 조건 |
| irs/ihs | off/hold/tap, 그 외 tap | 스폰 버퍼·held 상태 |

room handling은 위 처리 뒤 arr/das/sdf 세 값만 변경한다. 입력 사건 선계산에는 최종 S.handling과 초기 선택 경로가 필요하다.

## 경기 중 setoptions의 의미

DoCustomEvents의 setoptions는 key별 직접 할당이다. 초기 validator, RNG 생성, 전체 보드 초기화가 재실행되지 않는다.

| 변경 키 | 확인한 차이 |
|---|---|
| seed, seed_random | 옵션 값 변경만으로 RNG 재초기화 안 함 |
| handling | S.handling을 다시 만들지 않음 |
| room_handling 및 하위 값 | 초기 room override 재실행 안 함 |
| boardwidth/height/buffer | 값 할당과 보드 Resize 사건을 구분 |
| g | S.g에도 반영하는 경로 |
| stock | S.stock에도 반영하는 경로 |
| inverted | held 좌우와 DAS 상태를 초기화하는 부수 효과 |
| shielded·표시 별칭 등 | 각 키의 명시된 부수 처리 추적 필요 |

모드 내부 쓰기도 존재한다. Master는 locktime/lockresets, Zenith는 여러 garbage/messiness 설정을 진행 중 바꾼다. 모든 옵션을 영구 불변으로 공유할 수 없다.

## 시작 문맥과 남은 근거

StartGame의 server/client 경로, countdown과 stride, 입력 hoisting, OnClient 내부 pro_retry는 실행 문맥에 따라 다르다. 단순히 headless/server 플래그를 바꿔 기준 실행을 만들면 원본과 달라질 수 있다.

seed_random과 TEMP_zenith_rng가 실제 선택한 값을 export가 충분히 남기는지, user handling fallback과 Zen config가 파일에 있는지, 룸 상수가 누락됐는지를 원본 완전성 검사에서 다룬다. 이 감사는 모든 잘못된 타입·범위 조합의 실행 결과를 검증하지 않았다.
