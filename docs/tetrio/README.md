# TETR.IO 소스 감사

기준일: 2026-09-12. 공식 `tetrio.js`의 고정 스냅샷을 정적으로 분석하고, 제공된 두 리플레이의 구조를 관측했다. 선언과 실제 소비를 함께 읽었지만 모든 옵션 조합을 실행한 결과는 아니다.

## 기준과 상태

- 공식 소스 SHA-256: `eed14d6f268b2d6088dd82bc869a19f98410097e2c14a1f43cecb5a3d878526f`.
- 클라이언트 1.7.8, 현재 producer 규칙 버전 19, OptionsList의 version 기본값 11.
- UTF-8 2,523,270 bytes, UTF-16 길이 2,518,858. 문서의 `@숫자`는 원본 문자열의 **0 기반 UTF-16 오프셋**이며 줄 번호가 아니다.
- 173개 기본 옵션을 13개 분류에 중복 없이 배정했다. 미지 property 접근과 서버 설정까지 완전히 열거했다는 주장은 아니다.
- 표본에서 명시된 기본 옵션의 합집합은 57개. 나머지 116개는 명시되지 않았다. 생략된 기본값의 실행 여부와 비기본값 검증 여부는 다르다.

| 표기 | 의미 |
|---|---|
| 정적 관측 | 해당 소스에서 정의·읽기·쓰기·호출 경로를 확인 |
| 표본 관측 | 특정 SHA의 파일 구조나 사건을 확인 |
| 설계 반영 | 관측을 바탕으로 TTRX가 고려할 경계·요구 |
| 미확인 | 소스 밖 입력, 서버 동작, 실행 증거 등이 아직 부족 |
| 구현·실행 검증 | 이번 문서 작성에서 수행하지 않음 |

## 범위별 문서

| 범위 | 문서 |
|---|---|
| 근거 식별 | [소스·핸드오프·원본 위치](evidence/sources.md), [표본·CTK3·비교 구현](evidence/fixtures-and-references.md) |
| 옵션 | [173개 전수 목록](options/catalog.md), [초기화와 동적 변경](options/initialization.md) |
| 커스텀 방 | [프리셋 밖 규칙·상수·사건](options/custom-rooms.md) |
| 클라이언트 설정 | [입력 생성·표시·리플레이 뷰어](options/client-and-viewer.md) |
| 입력·시간 | [handling·subframe·스케줄러](runtime/input-and-time.md) |
| 보드·공급 | [미노·킥·map·RNG·특수 셀](runtime/board-and-supply.md) |
| 대전 | [공격·쓰레기·ACK·타깃](runtime/attack-and-garbage.md) |
| 사건·상태 | [custom 사건·snapshot·undo·종료](runtime/events-and-state.md) |
| 프리셋·매치 | [10개 프리셋·Practice·매치 결과](modes/presets-and-match.md) |
| 모드 | [목표·레벨·Survival·Zen](modes/levels-survival-zen.md), [Zenith·반전 카드·Duo](modes/zenith.md) |
| 리플레이 | [JSON 입출력·기존 바이너리 코덱](replay/source-and-binary.md) |
| 검증 | [범위별 사례·미확인 계약](verification/coverage-and-open-items.md) |

## 기계 판독 근거

- [source-manifest.json](evidence/source-manifest.json): 소스 해시·길이·파서·분류 개수.
- [options.json](evidence/options.json): 173개 정의의 기본값·원문 메타데이터·정의 위치·동일 property 이름 정적 참조 위치.
- [fixture-observations.json](evidence/fixture-observations.json): 표본 SHA·크기·구조·사건 수·옵션 합집합.
- [presets.json](evidence/presets.json): 10개 프리셋의 설정 이름·문자열 값·전달 순서. 자유 커스텀 규칙의 전체 목록이 아니다.

`options.json`의 참조 목록은 이름 기반 정적 색인이다. 별칭·동적 키의 완전한 데이터 흐름 추적이나 모든 참조가 게임 옵션이라는 보증이 아니다. 구현·실행 검증 플래그는 모두 false다.

## 분석 경계

옵션 선언, 초기화, 입력·물리·공급, 대전·쓰레기, 모드·Zenith, 상태·리플레이 직렬화를 구간별로 확인했다. 전용 binary codec 전체의 모든 custom payload 왕복과 모든 브라우저 경로를 실행하지 않았다. 서버에서 가능한 설정 전체, 서버의 match 판정, 룸 상수 생성과 카드 확장도 이 클라이언트만으로 완결하지 못한다.

프리셋 이름은 지원 판정의 필수 조건이 아니다. 원본 값과 실제 실행 규칙을 기준으로 [커스텀 규칙](options/custom-rooms.md)을 다룬다.
