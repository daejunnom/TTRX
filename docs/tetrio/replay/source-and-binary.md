# 리플레이 원본·입출력·기존 바이너리 코덱

기준: [공식 스냅샷](../evidence/sources.md). B @794751, StripBloat @795381, FormatOptions @795569, importer S @800745, event codec Ee @1091400 부근. JSON 표본 관측과 정적 소스 분석이며 전체 codec 왕복 실행은 미수행이다.

## 원본 구조와 버전

관측한 단일 구조는 replay.events, 대전 구조는 replay.rounds[r][p].replay.events다. 두 표본의 컨테이너 version은 1, options.version은 19다. 엔진 템플릿의 version 기본값 11과 세 값을 구분한다.

핸드오프의 data.events, data[r].replays[p].events는 구형 계열의 탐색 후보다. 이번 두 표본으로 지원을 입증한 경로가 아니다. 확장자만 보고 구조를 확정하거나 배열 위치로 여러 라운드의 같은 플레이어를 연결하지 않는다.

`frames`와 마지막 event.frame은 같지 않을 수 있다. full/start 필수 존재, 모든 timestamp의 단조성, 원본 numeric 값이 네트워크 비트 폭에 들어간다는 가정을 두지 않는다. [표본 근거](../evidence/fixtures-and-references.md)를 참조한다.

## 기록과 JSON 내보내기

B.Export는 frames/events/options/results를 내보낸다. B.SetOptions는 gameOptions를 복사하고 seed_random=false로 설정한다. 초기 room.constants를 저장하는 별도 필드는 기본 외형에 없다.

StripBloat는 모든 full을 제거하고 start.data를 빈 객체로, end.data를 reason 중심의 작은 객체로 바꾼다. results는 별도로 유지한다. full 없는 표본은 공식 export 경량화 경로의 결과일 수 있다.

noreplay는 기록 여부와 관련 flush를 바꾸고, fulloffset/fullinterval은 프레임 진행 후 full 생성 주기에 쓰인다. latencymode=zero/low/medium/high/xhigh는 입력 전송 provisioning 속도를 바꾼다. 전송 설정만으로 서버 지연 모델을 재생할 수는 없다.

## importer와 뷰어 변환

구버전 Upgrade는 infinitemovement→infinite_movement, infinitestock→infinite_stock, gbase→levelgbase, gspeed→levelgspeed, latencypreference→latencymode, x_resulttype→objective_result 등의 이름을 바꾸고 타입을 보정한다. objective 중첩을 평탄화하고 ghostskin을 minoskin.ghost로 이동한다. physical/presets/constants_overrides 삭제와 구 display→counter/bar 변경도 있다.

root version 없는 계열의 업그레이드와 0→1 변환, 일부 구형 단일 경기의 재실행·StripBloat 경로가 있다. 구형 league multiplayer 갱신에는 오류 경로도 있다. 업그레이드 결과만 보존하면 원본 필드·이름·존재 여부를 잃으므로 원본 복원 모델은 변환 이전 정보를 유지해야 한다.

FormatOptions는 뷰어용이다. single/multi에서 countdown·prestart·mission·zoom·display_replay를 바꾸며 single은 현재 사용자 pro/counter 설정도 적용한다. [클라이언트·뷰어](../options/client-and-viewer.md)에 상세 경계를 기록했다.

## 이벤트와 부분 snapshot

전용 codec의 8개 분기는 keydown/keyup/start/full/end/ige/strategy/manual_target이다. 미지 type에 일반 pack/unpack fallback도 존재한다. undo/redo/retry/exit는 keydown.data.key다.

full에 사용되는 Snapshot은 보드·bag·현재 미노·handling·일부 controlling·stats·diyusi를 포함한다. RNG·공급 내부·actors·대기 큐·상수·전체 모드 상태는 빠져 있다. engine의 직접 listener에도 full/start/end 상태 주입은 확인되지 않았다. full을 state_patch나 완전 checkpoint로 해석하지 않는다. [상태 인터페이스](../runtime/events-and-state.md)를 별도로 사용한다.

## 기존 binary codec의 관측

| 대상 | 확인한 표현 |
|---|---|
| key subframe | read/writeFloat(4,10) |
| strategy | unsigned 3bit |
| manual target | unsigned 13bit |
| snapshot bag 길이 | unsigned 12bit |
| g, shift ARR/DAS 누적, falling.y/locking | binary64 double |
| falling | signed x, unsigned hy, rotation/IRS 2bit, kick 5bit, keys 16bit, flags 15bit |
| safelock/lockresets/rotresets | 3/5/6bit |
| skip | 7bit index+1, 0 sentinel |
| handling | ARR 6bit/10, DAS·DCD 8bit/10, SDF 6bit와 flags·enum |
| end APM/PPS/VS | double |

이 폭은 해당 codec의 선택이다. permissive JSON·custom constants·임의 미노의 원본 값까지 같은 범위를 만족한다는 보증이 아니다. 이 표를 `.ttrx`의 최종 비트 형식으로 채택하지 않았다.

## 원본 의미 보존용으로 그대로 재사용할 수 없는 이유

- stats codec We는 zenlevel/zenprogress를 저장하지 않고 decode 때 1/0을 생성한다.
- 전용 stats/handling/falling은 알려진 필드 위주이며 미지 필드 보존을 보장하지 않는다.
- custom queue codec은 배열을 toString으로 쓰고 split(',')로 읽는다.
- tetrominoes codec은 orientation별 고정 셀 수와 제한된 키·비트표를 가정한다. matrix 셀의 세 번째 edge 값, disallow_kick 등 일부 속성을 기록하지 않는다.
- binary end Qe는 successful/gameoverreason/killer/options/aggregatestats와 Snapshot을 요구한다. StripBloat된 JSON end와 다른 필수 필드 계약이다.

따라서 기존 binary는 표현·상태 관측의 참고 자료이고, TTRX lossless source 모델의 대체물이 아니다. 더 작은 기존 codec roundtrip을 원본 데이터 의미 보존 성공으로 표시하지 않는다.

## 원본 복원과 실행 검증

원본 필드·정밀 수치·입력·미지 사건을 복원하는 경로와 결정론적 실행을 분리한다. 실제 외부 사건을 실행에 반영하고, source_residual은 출력 차이를 보존한다. 재계산 통계를 원본 값으로 바꾸기 위해 엔진을 임의 보정하지 않는다.

source semantic roundtrip, bit codec roundtrip, 공식 기준 실행과의 state trace 비교를 별도 결과로 보고한다. JSON.parse 기반 표본 집계는 숫자 lexeme·중복 키·lone surrogate·부호 있는 0의 완전 보존 검사가 아니다. 그런 검증은 정밀한 source parser와 별도 자료가 필요하다.

현재 미확인 범위는 모든 구형 원본 스키마, binary obfuscation의 모든 분기, 모든 custom payload 왕복, 초기 constants가 빠진 파일의 실행 계약이다. 두 표본의 동작이나 기존 binary 규격에 맞추어 이 범위를 조용히 삭제하지 않는다.
