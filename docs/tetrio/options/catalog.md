# 기본 게임 옵션 전수 목록

기준일: 2026-09-12. 상태: 정적 소스 관측. 구현 또는 실행 동치 검증 결과가 아니다.

[분석 목차](../README.md) · [초기화 의미](initialization.md) · [커스텀 방](custom-rooms.md) · [기계 판독 목록](../evidence/options.json)

공식 tetrio.js SHA-256: `eed14d6f268b2d6088dd82bc869a19f98410097e2c14a1f43cecb5a3d878526f`. 모든 위치는 원본 문자열의 0 기반 UTF-16 오프셋이다.

173개 선언을 13개 주분류로 배정했다. 누락 0, 중복 0. 표시 분류는 실행 영향이 없다는 보증이 아니다.

## 읽는 법

- 기본값은 엔진 OptionsList 선언값이다. UI 기본값·프리셋·서버 전달값·리플레이 뷰어 값과 다르다.
- 선언 제약은 원문 메타데이터다. allowed/possibles는 초기 validator가 강제하지 않는다. min=0만 있는 경우 범위 처리 조건도 주의한다.
- 참조 수는 전체 파일의 동일 property 이름 MemberExpression 수다. 같은 이름의 다른 객체가 포함될 수 있으며 동적 접근·별칭의 완전한 데이터 흐름 분석이 아니다.
- options.json의 implemented/runtimeVerified는 모두 false다. noextrawidth는 정적 member 참조를 찾지 못한 선언 항목이다.
- 중간 setoptions는 초기 SetOptions를 다시 호출하지 않는다. 값만 변경되는 것과 상태에 반영되는 것을 구분한다.

## 식별·시드 — 5개

실행 의미: [식별·시드](initialization.md)

| 키 | 선언 기본값 | 선언 제약·선택값 식 | 정의 위치 | 정적 이름 참조 수 |
|---|---|---|---:|---:|
| `version` | `11` | `static=!0` | 1016999 | 34 |
| `gameid` | `0` | `min=0; max=8192; strict=!0` | 1017029 | 48 |
| `seed` | `0` | — | 1017073 | 21 |
| `seed_random` | `false` | — | 1017090 | 4 |
| `username` | `""` | — | 1022323 | 300 |

## 입력·물리 — 16개

실행 의미: [입력·물리](../runtime/input-and-time.md)

| 키 | 선언 기본값 | 선언 제약·선택값 식 | 정의 위치 | 정적 이름 참조 수 |
|---|---|---|---:|---:|
| `are` | `0` | — | 1017133 | 2 |
| `lineclear_are` | `0` | — | 1017149 | 6 |
| `g` | `0.02` | — | 1017175 | 35 |
| `gincrease` | `0` | — | 1017191 | 3 |
| `gmargin` | `0` | — | 1017213 | 1 |
| `gravitymay20g` | `true` | — | 1017233 | 1 |
| `allow_harddrop` | `true` | — | 1019115 | 3 |
| `allow180` | `false` | — | 1019143 | 1 |
| `infinite_hold` | `false` | — | 1019165 | 4 |
| `infinite_movement` | `false` | — | 1019192 | 10 |
| `clutch` | `true` | — | 1019268 | 3 |
| `nolockout` | `false` | — | 1019308 | 3 |
| `locktime` | `30` | — | 1019581 | 12 |
| `lockresets` | `15` | `min=0; max=30` | 1019603 | 15 |
| `inverted` | `false` | — | 1019770 | 3 |
| `display_hold` | `true` | — | 1020961 | 7 |

## 보드·회전·공급 — 7개

실행 의미: [보드·회전·공급](../runtime/board-and-supply.md)

| 키 | 선언 기본값 | 선언 제약·선택값 식 | 정의 위치 | 정적 이름 참조 수 |
|---|---|---|---:|---:|
| `kickset` | `"SRS+"` | `possibles=["none","SRS","SRS+","TETRA-X","NRS","ARS","ASC"]` | 1018460 | 12 |
| `bagtype` | `"7-bag"` | `allowed=F.BagList` | 1018545 | 2 |
| `no_szo` | `false` | — | 1019288 | 1 |
| `boardwidth` | `10` | `min=4; max=100` | 1019406 | 10 |
| `boardheight` | `20` | `min=1; max=100` | 1019444 | 10 |
| `boardbuffer` | `20` | `min=0; max=100` | 1019483 | 6 |
| `map` | `""` | — | 1021405 | 97 |

## 핸들링 — 5개

실행 의미: [핸들링](../runtime/input-and-time.md)

| 키 | 선언 기본값 | 선언 제약·선택값 식 | 정의 위치 | 정적 이름 참조 수 |
|---|---|---|---:|---:|
| `handling` | `{}` | — | 1021422 | 121 |
| `room_handling` | `false` | — | 1021444 | 1 |
| `room_handling_arr` | `2` | — | 1021471 | 1 |
| `room_handling_das` | `10` | — | 1021501 | 1 |
| `room_handling_sdf` | `6` | — | 1021532 | 1 |

## 공격·스핀·B2B·All Clear — 13개

실행 의미: [공격·스핀·B2B·All Clear](../runtime/attack-and-garbage.md)

| 키 | 선언 기본값 | 선언 제약·선택값 식 | 정의 위치 | 정적 이름 참조 수 |
|---|---|---|---:|---:|
| `spinbonuses` | `"T-spins"` | `allowed=ne.SpinRules` | 1018304 | 12 |
| `combotable` | `"multiplier"` | `allowed=["none","multiplier","classic guideline","modern guideline"]` | 1018357 | 7 |
| `b2bchaining` | `false` | — | 1018736 | 7 |
| `b2bcharging` | `false` | — | 1018761 | 19 |
| `b2bextras` | `false` | — | 1018786 | 1 |
| `b2bcharge_at` | `4` | `integer=!0; min=0; max=10` | 1018809 | 26 |
| `b2bcharge_base` | `0` | `integer=!0; min=0; max=10` | 1018858 | 8 |
| `allclears` | `true` | — | 1018909 | 2 |
| `allclear_garbage` | `10` | `integer=!0; min=0` | 1018932 | 1 |
| `allclear_b2b` | `0` | `integer=!0; min=0` | 1018979 | 4 |
| `allclear_b2b_sends` | `false` | — | 1019021 | 1 |
| `allclear_b2b_dupes` | `true` | — | 1019053 | 1 |
| `allclear_charges` | `false` | — | 1019085 | 1 |

## 쓰레기·타기팅 — 34개

실행 의미: [쓰레기·타기팅](../runtime/attack-and-garbage.md)

| 키 | 선언 기본값 | 선언 제약·선택값 식 | 정의 위치 | 정적 이름 참조 수 |
|---|---|---|---:|---:|
| `shielded` | `0` | — | 1017260 | 8 |
| `hasgarbage` | `false` | — | 1017281 | 10 |
| `garbagespeed` | `20` | `integer=!0; min=1` | 1017327 | 4 |
| `garbagefavor` | `0` | — | 1017370 | 5 |
| `garbagemultiplier` | `1` | — | 1017395 | 8 |
| `receivemultiplier` | `1` | — | 1017425 | 3 |
| `cancelmultiplier` | `1` | — | 1017455 | 1 |
| `garbagemargin` | `0` | — | 1017484 | 1 |
| `garbageincrease` | `0` | — | 1017510 | 2 |
| `garbageholesize` | `1` | — | 1017538 | 3 |
| `garbagephase` | `0` | — | 1017566 | 4 |
| `garbagequeue` | `false` | — | 1017591 | 2 |
| `garbageentry` | `"instant"` | `allowed=["instant","continuous","delayed"]` | 1017617 | 4 |
| `garbageare` | `5` | `integer=!0; min=1` | 1017693 | 4 |
| `garbagearebump` | `12` | `integer=!0; min=0` | 1017733 | 1 |
| `garbagecap` | `8` | — | 1017778 | 2 |
| `garbagecapincrease` | `0` | — | 1017801 | 2 |
| `garbagecapmargin` | `0` | — | 1017832 | 1 |
| `garbagecapmax` | `40` | — | 1017861 | 1 |
| `garbageabsolutecap` | `0` | — | 1017888 | 5 |
| `garbageattackcap` | `0` | — | 1017919 | 2 |
| `garbagetargetbonus` | `"none"` | `allowed=["none","defensive","offensive"]` | 1017948 | 3 |
| `garbageblocking` | `"combo blocking"` | `allowed=["none","combo blocking","limited blocking"]` | 1018025 | 3 |
| `passthrough` | `"zero"` | `allowed=["zero","limited","consistent","full"]` | 1018121 | 3 |
| `openerphase` | `0` | — | 1018197 | 2 |
| `roundmode` | `"down"` | `allowed=["down","rng"]` | 1018221 | 1 |
| `garbagespecialbonus` | `false` | — | 1018271 | 1 |
| `messiness_change` | `1` | — | 1018589 | 6 |
| `messiness_inner` | `0` | — | 1018618 | 8 |
| `messiness_nosame` | `false` | — | 1018646 | 2 |
| `messiness_center` | `false` | — | 1018676 | 4 |
| `messiness_timeout` | `0` | — | 1018706 | 4 |
| `manual_allowed` | `true` | — | 1019331 | 1 |
| `new_payback` | `false` | — | 1019359 | 3 |

## 목표·레벨·점수 — 14개

실행 의미: [목표·레벨·점수](../modes/levels-survival-zen.md)

| 키 | 선언 기본값 | 선언 제약·선택값 식 | 정의 위치 | 정적 이름 참조 수 |
|---|---|---|---:|---:|
| `score` | `0` | — | 1017115 | 30 |
| `objective_type` | `"none"` | `allowed=["none","lines","timed"]` | 1019945 | 9 |
| `objective_count` | `0` | — | 1020010 | 13 |
| `objective_time` | `0` | — | 1020038 | 4 |
| `objective_result` | `""` | `allowed=["","score","time","lines"]` | 1020065 | 1 |
| `absolute_lines` | `false` | — | 1020798 | 3 |
| `levels` | `false` | — | 1021015 | 11 |
| `masterlevels` | `false` | — | 1021035 | 6 |
| `startinglevel` | `1` | — | 1021061 | 2 |
| `levelspeed` | `1` | — | 1021087 | 2 |
| `levelstatic` | `false` | — | 1021110 | 2 |
| `levelstaticspeed` | `10` | — | 1021135 | 2 |
| `levelgbase` | `0.8` | — | 1021165 | 2 |
| `levelgspeed` | `0.007` | — | 1021189 | 2 |

## 특수 보드·Survival — 10개

실행 의미: [특수 보드·Survival](../modes/levels-survival-zen.md)

| 키 | 선언 기본값 | 선언 제약·선택값 식 | 정의 위치 | 정적 이름 참조 수 |
|---|---|---|---:|---:|
| `usebombs` | `false` | — | 1017305 | 4 |
| `survivalmode` | `"none"` | `allowed=["none","layer","timer"]` | 1021739 | 8 |
| `survival_messiness` | `0` | — | 1021802 | 4 |
| `survival_layer_amt` | `10` | — | 1021833 | 2 |
| `survival_layer_non` | `false` | — | 1021865 | 1 |
| `survival_layer_min` | `0` | — | 1021897 | 1 |
| `survival_timer_itv` | `1` | — | 1021928 | 2 |
| `survival_cap` | `0` | — | 1021959 | 4 |
| `void_holes` | `0` | `integer=!0; min=0; max=15` | 1022037 | 2 |
| `void_holes_hungryness` | `12` | `integer=!0; min=1` | 1022084 | 1 |

## 생명·재시작·진입 — 18개

실행 의미: [생명·재시작·진입](../runtime/events-and-state.md)

| 키 | 선언 기본값 | 선언 제약·선택값 식 | 정의 위치 | 정적 이름 참조 수 |
|---|---|---|---:|---:|
| `can_undo` | `false` | — | 1019384 | 6 |
| `stock` | `0` | `min=0; max=10` | 1019522 | 23 |
| `infinite_stock` | `false` | — | 1019553 | 4 |
| `prestart` | `0` | — | 1019640 | 4 |
| `precountdown` | `0` | — | 1019661 | 3 |
| `countdown` | `false` | — | 1019686 | 7 |
| `countdown_count` | `3` | — | 1019709 | 3 |
| `countdown_interval` | `1000` | — | 1019737 | 2 |
| `stride` | `false` | — | 1020249 | 18 |
| `pro` | `false` | — | 1020269 | 6 |
| `pro_alert` | `false` | — | 1020286 | 2 |
| `pro_retry` | `false` | — | 1020309 | 2 |
| `can_retry` | `false` | — | 1020332 | 4 |
| `anchorseed` | `false` | — | 1022273 | 1 |
| `forfeit_time` | `60` | — | 1022297 | 1 |
| `fromretry` | `false` | — | 1022475 | 1 |
| `retryisclear` | `false` | — | 1022498 | 3 |
| `topoutisclear` | `false` | — | 1022524 | 7 |

## Zen·튜토리얼 — 5개

실행 의미: [Zen·튜토리얼](../modes/levels-survival-zen.md)

| 키 | 선언 기본값 | 선언 제약·선택값 식 | 정의 위치 | 정적 이름 참조 수 |
|---|---|---|---:|---:|
| `tutorial` | `false` | — | 1022136 | 74 |
| `usezenconfig` | `false` | — | 1022158 | 8 |
| `zenlevels` | `false` | — | 1022184 | 11 |
| `zenlevel` | `1` | — | 1022207 | 16 |
| `zenprogress` | `0` | — | 1022228 | 18 |

## Zenith — 14개

실행 의미: [Zenith](../modes/zenith.md)

| 키 | 선언 기본값 | 선언 제약·선택값 식 | 정의 위치 | 정적 이름 참조 수 |
|---|---|---|---:|---:|
| `zenith` | `false` | — | 1022551 | 541 |
| `zenith_expert` | `false` | — | 1022571 | 15 |
| `zenith_doublehole` | `false` | — | 1022598 | 2 |
| `zenith_volatile` | `false` | — | 1022629 | 9 |
| `zenith_gravity` | `false` | — | 1022658 | 1 |
| `zenith_messy` | `false` | — | 1022686 | 5 |
| `zenith_allspin` | `false` | — | 1022712 | 7 |
| `zenith_duo` | `false` | — | 1022740 | 32 |
| `zenith_mods` | `[]` | `allowed=me.Mods` | 1022764 | 83 |
| `zenith_ally` | `[]` | — | 1022805 | 3 |
| `zenith_allyexpert` | `false` | — | 1022830 | 1 |
| `zenith_isshadowedside` | `false` | — | 1022861 | 2 |
| `TEMP_zenith_rng` | `false` | — | 1022896 | 2 |
| `TEMP_zenith_grace` | `""` | — | 1022925 | 2 |

## 화면·소리·안내 — 28개

실행 의미: [화면·소리·안내](client-and-viewer.md)

| 키 | 선언 기본값 | 선언 제약·선택값 식 | 정의 위치 | 정적 이름 참조 수 |
|---|---|---|---:|---:|
| `nextcount` | `5` | `integer=!0; min=1; max=6` | 1019223 | 5 |
| `mission` | `""` | — | 1019792 | 5 |
| `mission_type` | `"mission"` | `allowed=["mission","mission_free","mission_versus","mission_league"]` | 1019813 | 4 |
| `no_mission_sound` | `false` | — | 1019915 | 1 |
| `zoominto` | `"none"` | `allowed=["none","slow","fast","cinematic","fade","zenithduoleft"]` | 1020131 | 4 |
| `noextrawidth` | `false` | — | 1020223 | 0 |
| `slot_counter1` | `""` | `allowed=["",...O.DisplayCountersList]` | 1020355 | 9 |
| `slot_counter2` | `""` | `allowed=["",...O.DisplayCountersList]` | 1020420 | 9 |
| `slot_counter3` | `""` | `allowed=["",...O.DisplayCountersList]` | 1020485 | 6 |
| `slot_counter4` | `""` | `allowed=["",...O.DisplayCountersList]` | 1020550 | 5 |
| `slot_counter5` | `""` | `allowed=["",...O.DisplayCountersList]` | 1020615 | 9 |
| `slot_bar1` | `""` | `allowed=["","impending","progress"]` | 1020680 | 12 |
| `slot_bar2` | `""` | `allowed=["","impending","progress"]` | 1020739 | 7 |
| `display_zen` | `false` | — | 1020826 | 2 |
| `display_username` | `false` | — | 1020851 | 8 |
| `display_fire` | `false` | — | 1020881 | 6 |
| `display_replay` | `false` | — | 1020907 | 8 |
| `display_next` | `true` | — | 1020935 | 4 |
| `display_shadow` | `true` | — | 1020987 | 1 |
| `minoskin` | `{"z":"tetrio","l":"tetrio","o":"tetrio","s":"tetrio","i":"tetrio","j":"tetrio","t":"tetrio","other":"tetrio","ghost":"tetrio"}` | — | 1021216 | 20 |
| `boardskin` | `"generic"` | `possibles=["generic","tetrio"]` | 1021344 | 5 |
| `nosound` | `false` | — | 1021584 | 3 |
| `bgmnoreset` | `false` | — | 1021605 | 3 |
| `neverstopbgm` | `false` | — | 1021629 | 1 |
| `song` | `"RANDOMcalm"` | `possibles=["none","RANDOM","RANDOMcalm","RANDOMbattle"]` | 1021655 | 10 |
| `invisible` | `false` | — | 1021984 | 1 |
| `master_invisible` | `false` | — | 1022007 | 1 |
| `nosiren` | `false` | — | 1022252 | 2 |

## 리플레이·전송 — 4개

실행 의미: [리플레이·전송](../replay/source-and-binary.md)

| 키 | 선언 기본값 | 선언 제약·선택값 식 | 정의 위치 | 정적 이름 참조 수 |
|---|---|---|---:|---:|
| `noreplay` | `false` | — | 1021562 | 9 |
| `latencymode` | `"medium"` | `allowed=["zero","low","medium","high","xhigh"]` | 1022345 | 2 |
| `fulloffset` | `300` | — | 1022423 | 1 |
| `fullinterval` | `300` | — | 1022448 | 1 |

## 선언 목록 밖의 값과 확장

SRS-X, objective_type=garbage, zoominto=zenithduoright 등은 실행 분기에서 확인됐다. 실제 공급 14종·킥셋 8종·기본 미노 13종, custom constants와 custom events는 각 범위 문서에서 다룬다. 이 표를 서버 허용 값의 화이트리스트나 프리셋 한정 지원표로 사용하지 않는다.
