# 사용자 설정·표시·리플레이 뷰어

기준: [공식 스냅샷](../evidence/sources.md). 사용자 기본 객체 ot @1204891 부근, handling reset @1411509, replay FormatOptions @795569. 실제 사용자의 localStorage나 계정 설정을 읽은 결과가 아니라 소스의 기본 객체와 소비 경로 분석이다.

## 게임 옵션 밖의 사용자 설정

| 그룹 | 확인한 상위 키 수 | 내용 |
|---|---:|---|
| controls | 4 | style, custom, sensitivity, vibration |
| handling | 9 | arr, das, dcd, sdf, safelock, cancel, may20g, irs, ihs |
| volume | 12 | disable, music, sfx, stereo, others, attacks, zenithrank, next, noreset, oof, scrollable, bgmtweak |
| video | 43 | 아래 목록 |
| gameoptions | 18 | 40L/Blitz 각각 pro·alert·retry·stride 및 counter 5개 |
| electron | 8 | loginskip, frameratelimit, presence, taskbarflash, autoupdate, anglecompat, adblock, devtools |
| notifications | 8 | suppress, forcesound, online, offline, dm, dm_pending, invite, other |

controls.custom의 22개 바인딩은 moveLeft, moveRight, softDrop, hardDrop, rotateCCW, rotateCW, rotate180, hold, exit, retry, chat, target1..4, menuUp/Down/Left/Right/Back/Confirm, openSocial이다. 문서 작성 시 원본 기본 객체의 실제 키를 다시 세어 22개로 확인했다. 컨트롤러 gpdown/gpup도 replay에는 논리 keydown/keyup으로 변환된다.

video 43개: graphics, caching, actiontext, particles, background, bounciness, shakiness, gridopacity, boardopacity, shadowopacity, zoom, alwaystiny, nosuperlobbyanim, nozenithanim, colorshadow, holdlocked, sidebyside, spin, chatfilter, background_url, background_usecustom, nochat, hideroomids, emotes, emotes_anim, siren, powersave, invert, nobg, chatbg, replaytoolsnocollapse, kos, fire, focuswarning, hidenetwork, guide, lowrescounters, desktopnotifications, lowres, webgl, bloom, chroma, flashwave.

이 설정들이 모두 replay JSON 필드라는 뜻은 아니다. 입력 생성, 엔진 초기화 fallback, 뷰어 표시, 소리, 플랫폼 기능을 구분한다. 원본에 있는 필드는 보존하되 현재 사용자 설정으로 원본을 덮어쓰지 않는다. video/electron/notifications의 모든 렌더링·플랫폼 동작을 TTRX 재생 엔진으로 이식하는 범위는 확정하지 않았다.

## 표시 이름으로 실행 영향을 판단하지 않는다

| 항목 | 확인한 소비 |
|---|---|
| display_hold | Hold에서 실제 동작을 허용/차단 |
| pro_retry | local/fullmode/can_retry 조건에서 finesse 실패 후 GameOver retry 가능 |
| pro_alert | finesse 알림·효과음 |
| nextcount | 주로 preview 렌더링. 공급 refill 임계값을 정하지 않음 |
| display_next | preview와 Next 효과음 경로 |
| invisible/master_invisible | 확인한 렌더러에서 alpha 변경, 그 지점의 shared RNG 소비 없음 |
| lineclear_are 관련 연출 | OnClient 밖에서 rngex를 소비해 예약을 만듦 |
| Zen counters=versus | 외부 Zen 설정이 hasgarbage도 true로 변경 |
| kick color override | 실제 보드 셀 값으로 반영 |
| noextrawidth | 이 스냅샷에서 선언 외 정적 소비를 찾지 못함 |

따라서 source-only로 분류하기 전 상태·RNG·종료·통계 소비 경로를 확인해야 한다. 반대로 렌더러에서만 사용된 값까지 임의의 상태 보정 사건으로 만들지도 않는다.

## 뷰어 FormatOptions

multi 뷰어는 display_replay, countdown, precountdown/prestart, mission, zoominto를 수정한다. single 뷰어는 추가로 stride, song, username/display_username과 현재 사용자 pro·counter 설정을 적용한다.

이 변환은 원본 경기 옵션의 canonicalization이 아니다. raw source, 실제 경기의 effective 설정, 뷰어가 만든 설정을 따로 기록한다. 원본 복원에는 현재 사용자의 pro/slot 설정을 섞지 않는다.

## 재생용 최소 구현에 미치는 영향

렌더러·소리 시스템을 생략하는 것은 가능하지만 shared RNG 호출과 실행 종료를 바꾸는 코드까지 함께 제거하면 동치가 깨진다. 외부 사용자 설정을 참조하는 경로는 캡처 또는 계약 입력이 필요한지 판정한다. 실제 사용자 저장소를 읽어 임의 기본값으로 맞추는 것은 원본 증거를 대신하지 않는다.
