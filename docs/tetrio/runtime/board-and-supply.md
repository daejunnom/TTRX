# 보드·미노·회전·공급·RNG

기준: [공식 스냅샷](../evidence/sources.md). PRNG C @743012, 상수 w @751562, 공급 F @827605, 보드 I @830353, LoadMap @837049, 상수 M @843684, 줄 삭제 RNG @840498 부근. 정적 관측이다.

## 난수 계약

PRNG 생성자는 seed를 2147483647로 나눈 나머지를 사용하고, 0 이하의 값에는 2147483646을 더한 뒤 falsy 보정을 수행한다. 갱신은 16807을 곱한 나머지다. 생성자에는 parseInt가 없고 seed 옵션도 integer 제한이 없다. 정상 정수 seed의 알고리즘을 모든 fractional seed에 대한 정수 연산으로 일반화하지 않는다.

rng와 rngex는 같은 effective seed에서 각각 생성되는 독립 상태다. 전자는 주로 공급, 후자는 공격 반올림·구멍·Survival·actors·Zenith 등에 쓰인다. 줄 삭제 연출 예약도 rngex를 소비한다. Math.random 기반 비결정적 초기화·시각 효과와 구분한다.

seed_random은 초기 seed 선택을 바꾼다. custom.setoptions.seed는 RNG를 다시 초기화하지 않는다. 룸 상수, map, 공급 보충, no_szo, actor 생성 순서를 함께 보존해야 한다.

## 공급 알고리즘 14종

| bagtype | 확인한 생성 동작 |
|---|---|
| 7-bag | minotypes 한 벌 shuffle. switch의 기본 경로도 여기에 연결 |
| 14-bag | 두 벌 shuffle |
| total mayhem | minotypes에서 독립적으로 7번 선택 |
| classic | 7번 생성. 이전 선택과 동일하거나 sentinel이면 한 번 재추출; lastGenerated 유지 |
| pairs | minotypes를 shuffle해 처음 두 종류를 각각 3개씩 넣고 다시 shuffle |
| 7+1-bag | 한 벌에 무작위 1개 추가 후 shuffle |
| 7+2-bag | 한 벌에 무작위 2개 추가 후 shuffle |
| 7+x-bag | bagid에 따라 3,2,1,1,0…개 추가; 별도 bagex pool |
| 7-bag+oo | 정상 한 벌과 O를 OO로 바꾼 한 벌을 각각 shuffle해 연결 |
| 7+1-lone-bag | 한 벌과 minotypes 밖 piece 1개 |
| 7+2-lone-bag | 한 벌과 minotypes 밖 piece 2개 |
| 14+1-lone-bag | 두 벌과 minotypes 밖 piece 1개 |
| 14+2-lone-bag | 두 벌과 minotypes 밖 piece 2개 |
| zenith | cancelstreak·volatile·I5 등장 이력에 따라 추가 공급 |

기본 minotypes 순서는 z,l,o,s,i,j,t다. PullFromBag는 소비 직전 queue 길이가 14 미만이면 보충하며 Zenith는 임계값 6을 사용한다. nextcount가 이 임계값을 정하지 않는다. no_szo는 초기 queue의 선두가 S/Z/O가 아니도록 회전시키는 처리이며 매 bag의 금지 규칙이 아니다.

재생 상태에는 bag, bagex, bagid, lastGenerated, rng가 필요하다. Zenith는 cancelstreak/hasseenI5 등 모드 상태도 필요하다. queue만 보존하고 공급 내부 상태를 버릴 수 없다.

## 미노와 킥

- 기본 미노 정의 13종: i1, i2, i3, l3, i5, z, l, o, s, i, j, t, oo.
- 실제 킥셋 8종: SRS, SRS+, SRS-X, TETRA-X, NRS, ARS, ASC, none.
- SRS-X는 OptionsList.kickset.possibles에서 빠져 있지만 실행 테이블과 4wide 프리셋에 있다.
- 기본 미노는 1~8개 셀을 가질 수 있다. 4셀·7종·4줄 클리어만 가능한 모델로 고정하지 않는다.
- per-piece kickset_override, disallow_kick, kickset_special, root constants.kickset이 있다.
- ASC의 O 킥, ARS/NRS의 스폰 rotation과 추가 offset, ARS I의 버전 분기를 구분한다.
- color_overrides는 GetPieceColor를 거쳐 실제 보드 셀에 저장된다. 원본 셀 의미를 일반 색 ID로 임의 정규화하지 않는다.

custom tetrominoes는 minotypes·matrix·색상 등을 교체할 수 있다. 현재 미노, hold, queue를 조정하고 미노가 제거되거나 형상이 바뀌면 30frame 대기 후 respawn하는 경로가 있다. 소스가 처리하는 기능과 일반 방 사용자의 서버 권한은 [커스텀 방 문서](../options/custom-rooms.md)처럼 구분한다.

## 보드·map

기본 width/height/buffer는 10/20/20이다. 선언 범위는 width 4..100, height 1..100, buffer 0..100이며, 선언의 number 타입을 항상 정수로 검증하는 것은 아니다. 실제 Resize와 잘못된 입력의 결과는 별도로 검증해야 한다. CTK3의 31 한도를 적용하지 않는다.

map 문법은 board?queue?hold다. 공백 제거·소문자화, `_/#/@`를 빈 셀/garbage/unclearable로 변환, 하단 정렬, 미노 정의에 따른 queue/hold 필터, bag 보충을 포함한다. 부적합 크기나 예외 경로에서 board/hold/bag를 초기화하더라도 앞서 소비한 RNG를 자동 복원하는 것은 아니다.

boardsize와 boardresize, map 사건은 보드 보존·현재 미노 처리·대기 동작이 다르다. options.boardwidth 할당만으로 이 동작이 실행되지 않는다. 숨은 행, 셀 색/종류, skip 셀, 낙하 미노의 fractional y를 보존한다.

## 특수 셀과 actors

| 기능 | 상태 변화 |
|---|---|
| bombs | garbage hole의 gbd, 고정 미노 아래의 gbd→gb 전환, abovePerma 삽입 |
| permanent/unclearable | 일반 garbage와 다른 행 유지·삽입·삭제 의미 |
| wound_timer | 시간 또는 clear 횟수로 행의 gb/null 상태 변경 |
| voidhole | stack과 falling 셀을 먹고 falling.skip 변경; 이동에 rngex 사용 |
| 모든 falling 셀 소멸 | sleep 후 30frame 뒤 다음 미노 예약 |

actor ID 증가, 좌표·anchor, 먹이 수와 다음 이동, wound 만료도 상태다. 렌더링 오브젝트처럼 버릴 수 없다. 초기 void_holes 생성 자체가 rngex를 소비한다.

## 렌더링을 생략할 때

RemoveLinesFromStack의 clear 연출 예약은 OnClient 밖에서 rngex를 소비한다. 화면을 그리지 않더라도 난수 소비와 예약 순서는 원본과 맞아야 한다. invisible/master_invisible의 확인한 alpha 변경 경로에는 같은 shared RNG 소비가 없었다. 이름이나 시각적 목적만으로 제거 대상을 결정하지 않는다.

## 검증 사례

- 14종 공급의 refill 경계, bagid/bagex/lastGenerated, map queue/hold 후 초기 보충과 no_szo.
- 정상·0·음수·fractional seed와 두 RNG 상태, 중간 seed 옵션 변경.
- 8종 킥셋과 13종 piece, 추가 offset, O 킥, ARS 충돌, custom matrix·skip.
- 비표준 width/height/buffer, map 실패 후 RNG, boardresize와 boardsize의 차이.
- lineclear_are 0/양수에서 다음 garbage column과 RNG 상태.
- bomb/permanent/wound/voidhole의 보드·현재 미노·예약 사건 결과.
