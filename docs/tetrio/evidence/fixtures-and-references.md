# 표본 리플레이와 비교 구현

기준일: 2026-09-12. 표본의 SHA·구조·사건 수와 로컬 CTK3 패키지를 문서 작성 때 다시 확인했다. 숫자 정밀도를 보존하는 TTRX 파서나 게임 실행 검증을 수행한 것은 아니다.

## 제공 표본

| 항목 | 단일 표본 | 대전 표본 |
|---|---|---|
| 파일명 | a0d49c30cc1e.ttr | versus pulsar_.ttrm |
| 원본 위치 | C:/Users/강민수/Downloads/a0d49c30cc1e.ttr | C:/Users/강민수/Downloads/versus pulsar_.ttrm |
| 크기 | 137,704 bytes | 458,602 bytes |
| SHA-256 | 4a6ccf1c0247a5abd9d7190e6bdf64b3f20767f7e82c7a683b34a31b1d1c9876 | a8526dab6b28e43564a1376a9126ecf9a67a125ab2183cba13e42dbc5d1cd05f |
| 컨테이너 version | 1 | 1 |
| 게임 규칙 version | 19 | 19 |
| 이벤트 경로 | replay.events | replay.rounds[r][p].replay.events |
| 라운드·스트림 | 1스트림 | 6라운드·12스트림 |
| 이벤트 합계 | 1,907 | 5,632 |
| start / end | 1 / 1 | 12 / 12 |
| keydown / keyup | 953 / 952 | 2,669 / 2,662 |
| IGE | 0 | 277 |
| full | 0 | 0 |

합계 크기는 596,306 bytes다. 대전 IGE는 target 13, allow_targeting 13, interaction 125, interaction_confirm 125, kev 1이다. 바깥 event.frame과 envelope.frame이 다른 사건은 253개다. payload.frame이 존재하는 251개는 모두 바깥 frame과 다르며, 나머지 26개에는 payload.frame이 없다. 없는 필드와 값이 다른 필드를 같은 범주로 세지 않는다.

단일 표본의 frames는 7200, 마지막 event.frame은 7199다. 두 표본에서 관측한 key subframe은 0,0.1,...,0.9다. 이전 정적 표본 분석에서는 같은 tick에 복수 입력과 hoisted가 관측됐고, 라운드 간 플레이어 배열 순서도 달랐다. 따라서 ID로 스트림을 연결하고 원본 배열 순서는 별도로 보존한다. 이 표본 관측을 모든 입력의 시간·순서 제약으로 확대하지 않는다.

기본 옵션 명시 합집합 57개, 선언 173개 중 미명시 116개, 선언 밖 키 0개다. [기계 판독 결과](fixture-observations.json)에 파일별 frame·옵션 키·사건 수가 있다. 옵션 미명시는 기본값 경로의 실행이 없었다는 뜻이 아니다. 두 표본만으로 커스텀 규칙·Zenith·동적 상수의 지원을 검증할 수 없다.

## CTK3

| 항목 | 확인한 기준 |
|---|---|
| 저장소 | C:/Users/강민수/Desktop/프로젝트/Clearra/Clearra |
| Git HEAD | 701454b27851e088521ac7b9c3291c404ac5a414 |
| 패키지 | packages/ctk3, version 0.1.1 |
| 확인 범위 | scoped git status 깨끗함, package.json, 추적된 codec.ts |
| 현재 패키지 의존성 | tetris-fumen ^1.1.3 |
| codec 기본 크기 한도 | width/height 각각 31 |

기존 대화의 소스 비교에서는 npm gitHead 7e0e27644c7445c98ea368f49cb525eacd2ec3de와 로컬 패키지 소스의 차이가 없다고 관측했다. 이번 문서 작성에서는 npm 배포물을 다시 내려받아 비교하지 않았으므로 이 문장은 이전 비교 시점의 기록이다.

재사용 후보는 bit reader/writer의 원리, 좁은 값 범위·차분·공유·반복 표현, 실제 비용에 따른 후보 선택이다. CTK3의 크기 한도·빈 행 축약·회전 정규화·미노/팔레트 ID를 TTRX 원본 의미에 그대로 적용하지 않는다. Rust 코어에서 원리를 재구현하는 것과 TypeScript 패키지·tetris-fumen을 런타임 의존성으로 쓰는 것은 다른 선택이다.

## Triangle

이전 비교 대상은 halp1/triangle의 ff2e1d78d3bb1b90c62a837d0844a264ed370af8 스냅샷이다. 보조 구현의 테스트 성공은 공식 TETR.IO와의 실행 동치 증거가 아니다. 당시 확인한 조기 topout 방지 수준의 테스트와 converter의 boardwidth/boardheight 교환 문제 때문에 원본 어댑터를 그대로 복사하는 근거로 사용하지 않는다.

Triangle은 이번 문서 작성에서 다시 다운로드하거나 실행하지 않았다. 구현 단계에서 이를 사용할 경우 해당 커밋과 관련 코드를 재검증한다. 공식 소스와 표본 관측을 우선하며, 비교 구현끼리의 일치만으로 호환 완료를 표시하지 않는다.
