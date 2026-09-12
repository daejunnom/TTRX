# 커스텀 방: 프리셋 밖의 규칙과 변경

기준일: 2026-09-12. 사용자 추가 요구: 커스텀 방에서는 프리셋으로 고정되지 않은 커스텀 규칙이 가능하므로 문서와 구현 검토에 포함한다. 아래는 [고정 소스](../evidence/sources.md)의 정적 관측과 그에 따른 설계 반영이다.

## 프리셋은 규칙의 시작값이다

10개 프리셋을 전체 규칙 종류의 열거형으로 취급하지 않는다. 프리셋을 사용하지 않고 개별 설정을 지정하거나, 프리셋을 적용한 뒤 값을 바꾼 방도 분석 대상이다. 같은 프리셋 이름이라도 최종 설정이 다를 수 있고, 이름이 없어도 실제 규칙이 같을 수 있다.

TTRX는 원본의 프리셋 이름을 메타데이터로 보존할 수 있지만, 그 이름만 저장하고 현재 프리셋 표로 실행 옵션을 재생성하지 않는다. 규칙 버전과 당시 실제 값·상수·적용 순서를 기준으로 실행을 판정한다. 프리셋을 찾을 수 없다는 이유만으로 알려진 규칙 조합을 미지원으로 분류하지 않는다.

## 커스텀 규칙이 들어오는 경로

| 경로 | 소스에서 확인한 내용 | 판정 경계 |
|---|---|---|
| 개별 방 설정 | `options.*`와 match/room 설정을 UI 또는 명령으로 전송 | 클라이언트 UI 목록이 서버 허용 목록 전체는 아님 |
| 프리셋 적용 후 변경 | `qn` 프리셋이 여러 설정을 채우며 개별 설정 경로와 연결 | 프리셋 최종 이름보다 실행 시 실제 값을 보존 |
| `/SET` | `;`로 항목, `=`로 이름/값을 분리하고 escape를 복원해 `room.setconfig`로 전송 | UI에 없는 index도 전송. 서버 승인·정규화 규칙은 별도 |
| 초기 룸 상수 | `setGame(options, room.constants)` | 옵션과 다른 입력이며 SetOptions보다 먼저 적용 |
| 경기 중 custom 사건 | setoptions, constants, tetrominoes 및 보드·공급 조작 | 초기화 함수와 다른 상태 전이 |
| 모드 내부 변화 | 레벨·Zen·Zenith가 옵션과 파생 상태를 갱신 | 외부 설정 사건이 없어도 규칙 상태가 시간에 따라 변화 |

실제 초기 상수 전달은 local multiplayer @2088801, opponent attach @2090041에서 확인했다. `/SET`은 @2166095 부근이다. 모든 일반 방에서 모든 custom 사건이나 임의 상수 수정 권한이 제공된다는 뜻은 아니다. **실행기가 처리하는 능력, 일반 사용자에게 노출된 설정, 서버가 승인하는 범위를 구분한다.**

## 표현해야 할 커스텀 규칙 축

| 축 | 예시 | 함께 필요한 정보 |
|---|---|---|
| 보드 | width/height/buffer, map, 영구·폭탄 셀 | 좌표·색상·숨은 행·초기 queue/hold |
| 공급 | 14개 bag, no_szo, seed | RNG와 bag 내부 상태, Zenith 상태 의존 |
| 미노·회전 | 13개 기본 미노, 8개 킥셋, custom tetrominoes | matrix, minotypes, kick/spin/color overrides |
| 입력·물리 | handling, room handling, ARE, g, locktime, 180/hold | 초기 정규화와 실제 적용값, 호출 순서 |
| 공격·쓰레기 | spin/combo/B2B/AC, 배율·cap·blocking·entry·messiness | 상쇄 큐·ACK·타깃·rngex |
| 목표·생명 | lines/garbage/timed, levels/master, stock, 종료 옵션 | 목표 frame, 진행 상태, 예약 복구 |
| 특수 모드 | Survival·actors·Zenith의 개별 규칙 | 외부 설정과 모드 상태, 상대 사건 |
| 매치 | versus/royale/practice, FT/WB/GP, 인원 | 개별 보드 종료와 별도인 서버 경기 결과 |

이 표는 모든 조합이 서버에서 유효하다는 보증이 아니다. 확인한 기능을 지원 평가에서 누락하지 않기 위한 분해다.

## 원본 값·실행 값·변경 이력을 구분한다

초기화에 대해 `raw options → SetConstants → SetOptions → effective options와 derived state`의 관계를 기록한다. 여기서 raw options의 생략/명시, null/false/0과 정밀 수치를 원본 복원 계층이 보존한다. 이름 기반 압축이나 기본값 생략을 하더라도 존재 여부를 잃지 않는다.

경기 중 `custom.setoptions`는 직접 할당 경로다. seed를 바꿔도 RNG는 다시 초기화하지 않고, handling을 바꿔도 S.handling을 다시 만들지 않으며, boardwidth만 바꿔도 보드를 자동 재구성하지 않는다. g/stock/inverted 등 명시적으로 연결된 부수 효과만 추가로 일어난다. [초기화 문서](initialization.md)와 [사건 문서](../runtime/events-and-state.md)를 함께 적용한다.

custom constants와 tetrominoes는 데이터로 된 규칙 변경이다. 프리셋 번호나 고정 7종 미노 ID만으로 대체할 수 없다. 원본 의미를 보존하는 구조와 실행 가능한 상수·미노 정의의 검증을 구분한다.

## 지원 판정에 반영할 사항

- preset 이름이 없어도 실제 규칙이 알려져 있고 필요한 외부 입력이 있으면 실행 검증 후보로 다룬다.
- 원본 필드 보존 가능, 규칙 식별 가능, 필요한 초기 정보 존재, 실행 동치 검증 여부를 각각 판정한다.
- 미지 키를 보존했다고 그 실행 의미도 지원하는 것은 아니다. production client가 미지 초기 옵션을 건너뛴다는 사실도 원본 필드를 삭제할 근거가 아니다.
- 초기 room.constants가 적용됐지만 export에 근거가 빠진 경우, 기본 상수로 재생한 결과를 원본 동치로 표시하지 않는다. 외부 근거가 필요하다고 기록한다.
- typed JSON 값과 선언 enum 목록, 클라이언트가 실제 처리하는 값, 서버가 승인한 값은 같은 집합이 아니다.
- 형식·자원 한도는 악의적 입력에 대한 코덱의 검증으로 명시한다. 게임 옵션의 의미를 몰래 clamp하거나 프리셋으로 바꿔 해결하지 않는다.

## 필요한 검증 사례

아래는 앞으로 만들 합성·실제 자료의 설계이며, 서버 허용 또는 실행 성공 사례가 아니다.

1. 프리셋 없이 옵션을 명시한 경기와, 프리셋 적용 후 같은 effective 값으로 수정한 경기. 원본 차이는 보존하고 실행 동치는 따로 비교한다.
2. 같은 preset 표시를 갖지만 gravity/handling/garbage 규칙이 다른 경기. 이름만으로 잘못 공유하지 않는지 확인한다.
3. 비기본 보드·bag·kickset·room handling·garbageentry의 프리셋 밖 조합. 각 단독 옵션 외 결합 경계를 확인한다.
4. 같은 raw 옵션에 서로 다른 room.constants를 주는 초기화. 기본 상수와 custom 상수를 혼동하지 않는다.
5. 같은 프레임의 key 입력, custom.setoptions, constants, tetrominoes, boardresize 순서 변경.
6. 초기 seed/handling/boardwidth 변경과 경기 중 같은 키 변경의 차이.
7. 미지 키·enum 밖 값·형식 위반 값. 원본 보존 여부와 실행 지원/실패 이유를 구분한다.
8. 개별 보드 상태가 같아도 FT/WB/GP·인원·결과 사건이 다른 매치. 보드 재생으로 매치 결과를 덮어쓰지 않는다.

남은 외부 근거는 서버의 room.setconfig 검증·상수 생성·모드 확장·매치 판정이다. 이를 확인하지 못했다고 커스텀 규칙 전체를 범위에서 제외하지 않으며, 현재 확인한 클라이언트 동작부터 항목별로 추적한다.
