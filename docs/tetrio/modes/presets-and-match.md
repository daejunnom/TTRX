# 프리셋·모드 생성·매치 구성

기준: [공식 스냅샷](../evidence/sources.md). qn @1256425, 방 UI @1271852, Practice @1273836, preset 적용 UI @1556902, /SET 확장 @2170170 부근. 정적 관측이다.

## 프리셋 10개

각 프리셋이 전달하는 설정 이름과 문자열 값 전체는 [presets.json](../evidence/presets.json)에 원문 순서로 추출했다. 값은 UI·서버 정규화 전 command 값이며 곧바로 engine effective options가 아니다. 예를 들어 문자열 1/0을 boolean으로 해석하는 경로를 별도로 확인해야 한다.

| 이름 | 확인한 대표 조합 |
|---|---|
| default | 7-bag/SRS+, all-mini+, g=.02, gincrease=.0025, gmargin=3600, B2B charge, AC garbage=5 |
| tetra league | 2인·FT7, all-mini+, gincrease=.0035, gmargin=7200, B2B charge |
| tetra league (season 1) | 2인·FT7, T-spins, B2B chaining, charging 비활성, AC garbage=10 |
| classic | classic bag/NRS, 홀드·하드드롭·180 비활성, NEXT1, ARE12/lineclear18, ARR5/DAS16, locktime5 |
| enforced delays | SRS, 180 비활성, ARE7/lineclear35, DAS9/SDF10, limited blocking, classic guideline combo, cap100 |
| arcade | ARS, 180 비활성, NEXT3, ARE27/lineclear25, ARR1/SDF20, locktime18 |
| quickplay | legacy Quick Play, g=.05·gmargin0, T-spins, B2B chaining. Zenith와 구분 |
| 4wide | 4×26, SRS-X, handheld spin, cap 증가 |
| 100 battle royale | 100인 royale, NEXT6, ARE6/lineclear25, defensive bonus, delayed garbage, garbage ARE7/bump12, attack cap20·absolute cap12 |
| bombs | 7+2-bag, usebombs, garbage multiplier=.8/increase=.005, inner messiness=.3, AC garbage3 |

이 표는 각 프리셋의 전체 설정을 생략한 요약이다. 누락값은 JSON을 참조한다. 적용 명령 수는 각각 51/53/53/53/54/51/51/51/61/52개다. 173개 엔진 기본값과 프리셋 default는 다른 객체다.

프리셋에는 options.presets도 등장하지만 이 키는 173개 OptionsList의 실행 옵션이 아니다. 방 설정·프리셋 표시와 엔진 옵션을 같은 스키마로 취급하지 않는다. preset 명칭이 없거나 이후 값을 수정한 커스텀 방도 [독립 규칙 조합](../options/custom-rooms.md)으로 다룬다.

## 방·경기 설정

방 설정은 name, userLimit, autoStart, public, allowAnonymous, allowQueued, allowUnranked, userRankLimit, useBestRankAsLimit, gamebgm을 확인했다. autoStart=0은 비활성 시작 countdown 경로다. 입장·rank 조건은 개별 미노 물리와 별도의 방 메타데이터다.

match에는 gamemode, modename, ft, wb, gp가 있다. gamemode UI 값은 versus/royale/practice이며 practice는 2인 훈련과 undo/board reset을 안내한다. Practice는 10개 프리셋에 없지만 종료·리플레이 다운로드 처리에서도 소비된다.

FT/WB/GP의 UI·표시와 서버 전달은 확인했지만 승자 결정의 완전한 구현은 이 클라이언트에서 확인하지 못했다. game.match.score, game.advance, game.end로 scoreboard/leaderboard를 받는다. 단일 보드의 GameOver를 매치 승리 판정으로 대체하지 않는다.

## 단일 모드 생성 목록의 한계

w.gameModes에는 40l/blitz 두 항목만 있다. 그것을 전체 모드 목록으로 사용하지 않는다. 이름 표시 테이블에는 40l, blitz, 5mblast, zen, custom, league, zenith, zenithex가 있으며 5mblast는 실행 생성 계약을 확인하지 못한 잔존 이름이다.

40L·Blitz 고정 설정, Custom Solo, Zen과 서버가 공급하는 대전/Zenith 설정의 생성 경로를 구분한다. 룸 프리셋 quickplay가 현재 Zenith 초기화의 완전한 대체물이 아니다.

## 빌드·서버 추가 설정

현재 스냅샷의 league_additional_settings는 빈 객체이며 league season current=2, previous=1이다. league_mm_roundtime_min/max는 25/50이다. 이는 이 빌드의 설정 관측이며 서버 전체 규칙이나 미래 기본값이 아니다.

Zenith 추가 설정은 TEMP_zenith_grace와 messiness_timeout=0을 공급한다. 엔진 생성자 기본 grace와 값이 다르다. 카드·Practice·룸 상수를 서버가 확장하는 전체 규칙은 미확인이다.

## 검증 범위

10개 프리셋은 회귀 자료의 시작점이다. 이름 없는 커스텀 조합, 프리셋 수정, 동일 표시 이름의 서로 다른 actual values, 별도 room.constants, Practice와 다인전, FT/WB/GP·결과 사건 변화도 포함해야 한다. 서버가 허용한 실제 경기 자료와 합성 엔진 테스트를 구분한다.
