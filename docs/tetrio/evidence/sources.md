# 소스·문서 근거

기준일: 2026-09-12. 이 문서는 분석 스냅샷과 근거 위치를 식별한다. 원본 게임 코드를 작업소에 복사해 실행하거나 게임 규칙을 구현한 결과가 아니다.

## 공식 클라이언트

| 항목 | 값 |
|---|---|
| URL | [공식 tetrio.js](https://tetr.io/js/tetrio.js) |
| SHA-256 | `eed14d6f268b2d6088dd82bc869a19f98410097e2c14a1f43cecb5a3d878526f` |
| UTF-8 크기 | 2,523,270 bytes |
| UTF-16 길이 | 2,518,858 code units |
| 클라이언트 빌드 | 1.7.8 |
| producer 규칙 버전 | 19 |
| 옵션 템플릿 version 기본값 | 11 |
| 파서 | Acorn 8.15.0, ecmaVersion=latest, sourceType=script |
| 전체 AST 노드 수 | 629,233 |
| 기본 옵션 정의 수 | 173 |

문서 작성 때 HTTPS로 다시 읽고 SHA 일치를 확인했다. 소스는 메모리에서 파싱했으며 클라이언트 자체는 실행하지 않았다. 파서 라이브러리 사용은 감사 도구에 해당하고 TTRX 코어의 의존성 결정과 무관하다. 이 URL은 이후 변경될 수 있으므로 위치와 의미를 재검증할 때 반드시 해시부터 맞춘다.

## 위치 색인

아래 위치는 모두 **0 기반 UTF-16 오프셋**이다. 클래스 전체 구간의 시작과 메서드 위치를 함께 제공하며, minify 경계에서 몇 글자 차이의 주변 위치는 `부근`으로 표기한다.

| 범위 | 식별자·위치 |
|---|---|
| 빌드 추가 설정 | `_` @690378 부근; Zenith 추가 설정 @690979 |
| PRNG | `C` @743012 |
| 공개 게임 설정 | `setGame` @745863 부근 |
| 기본 게임 상수 | `w` @751562 부근 |
| 점수·공격 상수 | @770548 부근 |
| 게임 모드 생성 상수 | `w.gameModes` @791549 부근 |
| 리플레이 관리자 | `B` @794751; FormatOptions @795569; StripBloat @795381 |
| 구버전 importer | `S` @800745; Upgrade @800834 |
| actors | `L` @804940 부근 |
| 공격·쓰레기 | `z` @810407 부근 |
| 공급 | `F` @827605 |
| 보드 | `I` @830353; LoadMap @837049; 줄 삭제 RNG @840498 부근 |
| 상수 관리자 | `M` @843684 |
| 입력 스트림 | `P`, raw key 처리 @853700 부근 |
| 미노 물리 | `R` @858363; Fall @864175; Hold @867757 |
| IGE·custom | `$` @876402 부근; DoCustomEvents @878936 부근 |
| 입력 해석 | `j` @905523; ProcessSubframe @909736 |
| 레벨·공격 분류 | LevelLines @914927 부근; AnnounceLines @917278 부근 |
| 프레임 루프 | `p` @938038 부근 |
| 목표 검사 | CheckObjectiveCleared @951004 |
| 상태 입출력 | Snapshot @960300 부근; EjectState @961033 |
| stock·survival·타기팅 | @964017 / @965905 / @967621 부근 |
| 대기 사건 | WaitFrames @975688; ExecuteWaitingFrames @975781 |
| Zen | `_e` @976304; UpdateFromZenConfig @981828 |
| Zenith | `me` @984632; Update @985612 |
| 옵션 | OptionsList @1016979; ValidateOptions @1023459; SetOptions @1024655 |
| 시작·카운트다운 | StartGame @1029428; StartCountdown @1030735 |
| 전용 event binary codec | `Ee` @1091400 부근 |
| stats·custom·end codec | `We` @1106000 부근; `Ve` @1144000 부근; `Qe` @1156900 부근 |
| 사용자 설정 기본 객체 | `ot` @1204891 부근 |
| 방 프리셋 | `qn` @1256425 |
| 방·매치 UI | `Yn` @1271852; practice @1273836 |
| 룸 상수 실제 전달 | local @2088801; opponent @2090041 |
| 서버 경기 결과 소비 | @2137000 부근 |
| Practice 종료·내보내기 | @2147547 / @2147793 / @2147967 |
| 자유 설정 전달 | `/SET` @2166095 부근; preset 확장 @2170170 부근 |

정확한 옵션별 선언과 동일 property 참조 위치는 [options.json](options.json)에 있다. 위치 일치만으로 의미 검증을 대신하지 않는다.

## 사용자 핸드오프

- 위치: `C:/Users/강민수/Downloads/TTRX_Codex_Handoff.md`.
- SHA-256: `9868a775f513cd8afb931af26cf9ac4a090955aff1e4497484d59bf866e43cda`.
- 역할: 원본 보존, 정수 행동 선계산, 규칙·공급 계약, 코덱 책임, 검증·벤치마크 요구를 설명한 기획 자료.
- 문서 자체가 미검증 제안을 포함한다고 밝힌다. 소스 증거·실행 성공·현재 기술 선택의 유일한 기준으로 사용하지 않는다.

현재 작업소는 문서 작성 전 `master`에 커밋과 추적 파일이 없는 상태였다. 기존 구현을 수정한 작업이 아니다. [참고 구현과 표본](fixtures-and-references.md)을 별도로 확인한다.
