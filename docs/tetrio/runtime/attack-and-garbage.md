# 공격·쓰레기·ACK·타깃

기준: [공식 스냅샷](../evidence/sources.md). z @810407 부근, FightLines @820845, IncomingAttackHit @822924, TakeAllDamage @823930, garbage ARE @842237, AnnounceLines @917278, 타기팅 pe @967621. 정적 관측이며 서버 또는 실제 경기 실행 검증은 아니다.

## 옵션을 적용 단계별로 구분한다

기본값과 전체 34개 쓰레기·타기팅 옵션 및 13개 공격 정책은 [catalog](../options/catalog.md)에 있다.

| 단계 | 옵션 | 확인한 의미 |
|---|---|---|
| 활성·보호 | hasgarbage, shielded | shielded는 단순 boolean이 아니라 frame과 비교하는 보호 종료 값 |
| 공격·수신·상쇄 | garbagemultiplier, receivemultiplier, cancelmultiplier | 서로 다른 계산 단계의 배율 |
| 시간 변화 | garbagemargin/increase, garbagecapmargin/increase | margin보다 큰 frame부터 /60씩 누적 |
| 정수화 | roundmode | down/rng. 소수 부분이 있을 때만 rngex를 쓰는 AutoRound |
| 일반 공격 상한 | garbageattackcap | 별도 AC/surge를 포함한 모든 공격에 일괄 적용되는 상한이 아님 |
| 주입 상한 | garbagecap, garbagecapmax | floor(min(max,cap)), 한 번의 TakeAllDamage에서 처리할 행 수 |
| 대기량 상한 | garbageabsolutecap | pending 생성 시 제한·shielded 처리 |
| 확인 후 도착 | garbagespeed | incoming-attack-hit 예약 지연 |
| 경고·직렬 대기 | garbagephase, garbagequeue | sleeping/caution/danger/spawn 및 active/firstcycle과 결합 |
| 주입 | garbageentry, garbageare, garbagearebump | instant/continuous/delayed와 간격·잠금 연장 |
| 상쇄 | garbageblocking, passthrough, openerphase | 줄 삭제 시 유입 차단, ACK, 초반 방어 |
| 타깃 보너스 | garbagetargetbonus, garbagespecialbonus | enemies 수, downstack·quad/spin 등의 조건 |
| 구멍 | garbageholesize, garbagefavor, messiness_* | 패킷 내부·경계·timeout·중앙·동일 열 정책 |
| 사용자 타깃 | manual_allowed, new_payback | UI 수동 선택 허용과 payback 갱신 |

선택값은 garbageentry=instant/continuous/delayed, garbageblocking=none/combo blocking/limited blocking, passthrough=zero/limited/consistent/full, roundmode=down/rng, garbagetargetbonus=none/defensive/offensive다. 선언 enum을 validator가 강제하지 않는 점은 별도다.

## 공격 계산 순서

spinbonuses의 10개 값은 none, T-spins, T-spins+, all, all+, all-mini, all-mini+, mini-only, handheld, stupid다. combo는 none, multiplier, classic guideline, modern guideline의 네 가지다.

기본 일반 공격 single/double/triple/quad/penta는 0/1/2/4/5, 일반 spin은 2/4/6/10/12, mini는 0/1/2/4다. 5줄을 넘는 확장 계산도 있으므로 custom piece를 4줄 이하 모델로 처리하지 않는다. constants가 이 표를 바꿀 수 있다.

1. 실제 줄·spin, combo/B2B와 all-clear B2B 증가량을 계산한다.
2. B2B가 끊어지면 charging surge를 별도로 처리한다. 세 부분으로 나눠 순서대로 상쇄·발송하므로 하나로 합치지 않는다.
3. 기본 공격과 handheld 보정, B2B 배율·chaining·extras를 적용한다.
4. combo multiplier 또는 선택 표, 자신을 공격하는 enemies 수에 따른 보너스를 적용한다.
5. garbage 배율·AutoRound·special bonus·일반 공격 cap·Zenith 분기를 적용한다.
6. 공격·상쇄·유입 차단을 처리하고 별도의 all-clear 공격을 처리한다.

B2B chaining은 Math.log1p와 나머지 연산을 쓴다. allclear_b2b_sends, allclear_b2b_dupes, allclear_charges는 증가량뿐 아니라 기존 B2B와 중복되는지, 공격 보너스를 주는지, charge 문턱까지 채우는지를 바꾼다. 일반 공격 cap을 뒤의 AC/surge까지 무조건 적용하지 않는다.

## pending과 ACK

IGE interaction은 garbage를 pending에 넣고, interaction_confirm은 cid로 항목을 찾아 수신 통계·원인·도착 예약을 갱신한다. confirm 전에 항목이 모두 취소되면 confirm이 무시될 수 있다. payload의 공통값을 압축해 공유하더라도 사건은 별도로 실행한다.

zero/consistent에서는 incoming/outgoing ACK 기록을 사용한다. 상대 ackiid 이하 outgoing을 제거하고 아직 확인되지 않은 outgoing과 incoming을 상쇄한다. limited/full의 서버 측 차이를 이 클라이언트 로컬 분기만으로 확정하지 못했다.

pending 항목의 확인한 필드: id, cid, iid, ackiid, gameid, username, type, active, status, hardened, shielded, delay, queued, amt, x, y, pos, neg, size, column, position, color, actor_pos, actor_neg. firstcycle도 전이 중 생긴다. 없는 값은 옵션·기본값에서 파생될 수 있으므로 원본 존재 여부와 실행 값이 다르다.

추가 게임 상태에는 interactionid/garbageid, ACK incoming/outgoing, impendingdamage, garbageareentries, garbagearelockeduntil, waitingframes, notyetreceivedattacks, lastatktime/lasttanktime, lastcolumn/haschangedcolumn, lastoffensive/garbagebonus/nextwilltank, enemies/targets/diyusi/laststrategychange가 있다. Zenith는 hesitated attacks, windup, cancelstreak, ally/grace 상태도 읽는다.

## 상쇄와 주입 방식

- FightLines는 continuous garbageareentries를 먼저, impendingdamage를 다음으로 상쇄하고 hardened 항목은 건너뛴다.
- pending이 완전히 상쇄돼도 messiness_change RNG를 소비하거나 다음 column을 바꿀 수 있다.
- TakeAllDamage는 active이고 status=spawn인 항목만 처리한다. 나머지를 분리했다가 뒤로 붙이므로 단순 배열 prefix 처리와 다르다.
- instant는 즉시 삽입한다. delayed는 push-garbage-line 예약으로 전환한다. continuous는 계속 상쇄 가능한 ARE 행 대기열로 전환한다.
- delayed 예약 행은 FightLines의 두 큐와 같은 취소 대상이 아니다.
- ProcessGarbageARE는 pause/sleep·잠금 종료를 확인하고 한 행씩 주입하며 현재 미노 밀어올림·충돌·모드 상태를 갱신한다.
- 구멍 column은 숫자 또는 배열일 수 있다. bombs와 permanent/actor/position/color를 일반 한 구멍 회색 행으로 축소하지 않는다.

## 시간·타깃·외부 권위

local의 deferrable IGE는 상대 game frame을 기다리거나 4초 뒤 처리하고 실제 적용 시 replay에 기록한다. replay event.frame, envelope.frame, payload.frame은 다른 값일 수 있다. 기록된 적용 순서를 다른 시각으로 임의 정렬하지 않는다.

타깃 전략은 EVEN=0, ELIMINATION=1, RANDOM=2, PAYBACK=3, MANUAL=4이며 manual 판정은 >=4다. 변경 간격은 30frame, 서버 재평가 주기는 300frame이다. UpdateStrategy는 서버에 action을 emit하며 실제 선정 알고리즘을 담고 있지 않다. target/targeted/allow_targeting/kev와 매치 결과는 필요한 외부 입력이다.

manual_allowed의 직접 소비는 보드 클릭 UI @2089863이며 코어 SetStrategy가 같은 검사를 반복하지 않는다. new_payback은 enemies 변화에 따른 targets 갱신을 바꾼다. 프리셋이나 1대1 가정으로 enemies/targets 구조를 줄이지 않는다.

## 공유 RNG와 검증

AutoRound의 소수 정수화, favor·tie noise, messiness change/inner/timeout, 완전 상쇄, custom lines, 보드·Survival·actors·Zenith·줄 삭제 예약이 rngex를 공유한다. 렌더링 제거 또는 공격 합산이 이 소비 순서를 바꾸면 이후 결과가 달라진다.

필수 검증 축은 entry 3종×blocking 3종, passthrough 4종과 ACK 경계, cap 3계열과 margin 경계, 확인 전 완전 취소, pending/continuous 우선순위, hardened/queued/shielded, 소수 round와 messiness, B2B surge 분할·AC 조합, target/enemy 변경과 kev, 같은 frame의 custom 사건이다. 단독 값뿐 아니라 [프리셋 밖 조합](../options/custom-rooms.md)을 포함한다.

과거 공격 규칙 전체, 서버의 passthrough·타깃·승자 결정, 모든 custom numeric 폭은 미확인이다. 현재 관리자에서 직접 version/date 분기를 찾지 못한 사실은 과거 빌드 동치 증거가 아니다.
