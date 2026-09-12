# Zenith·반전 카드·Duo

기준: [공식 스냅샷](../evidence/sources.md). me @984632~1016960, Update @985612, 공격·보드·공급 관리자와 교차 연결. 모든 내용은 정적 관측이며 조합별 공식 실행 trace는 확보하지 않았다.

## 옵션과 모드 상태

기본 옵션 14개는 zenith, zenith_expert, zenith_doublehole, zenith_volatile, zenith_gravity, zenith_messy, zenith_allspin, zenith_duo, zenith_mods, zenith_ally, zenith_allyexpert, zenith_isshadowedside, TEMP_zenith_rng, TEMP_zenith_grace다. nohold/invisible 등의 효과는 다른 기본 옵션에도 분산된다.

정상 카드 9종: invisible, messy, volatile, nohold, doublehole, allspin, gravity, expert, duo. 각 이름의 _reversed 9종도 실제 분기가 있다. snowman/snowman_reversed는 문자열 목록에 있지만 활성 생성·실행 계약을 확인하지 못했다.

카드 이름만 zenith_mods에 넣으면 모든 효과가 자동 확장되는 구조가 아니다. 서버가 공급하는 booleans/handling/garbage/hold 설정 등이 함께 필요하다. 현재 클라이언트만으로 카드→초기 옵션의 완전한 확장을 만들지 않는다.

## 초기 추가 설정

생성자 내부 grace 기본값은 [0,4.8,3.9,2.1,1.4,1.3,.9,.6,.4,.3,.2]다. 현재 빌드 zenith_additional_settings는 [0,3.8,3.0,2.3,1.7,1.2,.8,.5,.5,.5,.2]와 messiness_timeout=0을 공급한다. 초기 생성자 값과 현재 모드 초기 계약을 같은 것으로 보지 않는다.

TEMP_zenith_rng는 Math.random으로 APM·중력·카드 flags·즉사 임계값 등을 선택한다. 이 경로에서는 seed만으로 그 파생값을 재생성할 수 없다. 사용된 값이나 실행 문맥이 충분히 기록됐는지 판정해야 한다.

## 매 프레임·층별 변경

기본 층 경계는 0,50,150,300,450,650,850,1100,1350,1650,Infinity다. rank, climb points, promotion fatigue/lock, bonus drain, speed cap, speedrun 조건, 시간별 targeting factor/grace가 있다.

Update는 messiness_change, messiness_inner, messiness_center, garbagefavor, garbagephase를 다시 계산한다. Gravity/Freefall은 층 전환 때 g/locktime도 바꾼다. 따라서 초기 옵션 snapshot만으로 모든 이후 설정이 고정되는 것이 아니다.

## 카드별 실행 결합

| 기능 | 확인한 효과 |
|---|---|
| Volatile·Zenith 공급 | cancelstreak·volatile·hasseenI5를 통해 미노 공급 변경 |
| Allspin | 최근 clear·piece·줄 수를 추적하고 반복 clear에 wound 행 생성 |
| 반전 Allspin | 최대 20행과 clutch=false까지 반영 |
| 반전 Doublehole | 다중 구멍, 최소 garbage 4행, blighted 배율·해제 조건 |
| 반전 Nohold | garbage 측면 상태와 층별 방향 변경 확률 |
| 반전 Volatile | 미리 생성한 column 두 개, targeting grace 입력량 /3 |
| 반전 Expert | 고도 감소, 장기 체류 수신 배율, 별도 fatigue |
| 반전 Duo | 동료 공격, 홀로 남은 뒤 60frame 동안 속도 감소, 별도 messiness/revive/fatigue |
| Invisible/Messy/Gravity/Expert | 표시·구멍·중력·진행 설정 및 모드 분기에 분산; 카드 이름만으로 초기 옵션 확장을 대체하지 않음 |

실제 board/clear/attack/supply 단계에서 효과가 적용되는 순서를 유지한다. 공급을 seed의 독립 함수로 떼거나, 모든 쓰레기 규칙을 공통 대전 옵션 한 번의 계산으로 합치면 부족하다.

## fatigue와 revive 자료

fatigue 테이블은 각각 47/78/176행이다. 효과에는 unclearable, receivemultiplier, gracestillmessy, maxmessy, messiness, revivelevel, norevive, rerollcolumn과 처리기의 maxrank가 있다. 시각효과 외 실제 상태·수신·부활 제한을 변경한다.

Duo revive 자료는 79개 프롬프트 행, 68개 고유 ID, 18개 덱이다. rngex shuffle, 중복 ID 제외, 카드별 제외, 덱 순서가 있으며 norevive 때도 ospinconsecutive 과제를 만드는 경로가 있다.

무력화·부활은 pause/board blackout, RNG 소비, 상대 상태, 과제 lock·stun, 12frame 예약, hold/bag/B2B 초기화, 360frame glock, 경과 시간에 따른 permanent 행 복원을 포함한다. 일반 stock 90frame 복구와 합치지 않는다.

## 외부 사건과 필요한 상태

외부 confirm 사건은 zenith.climb_pts, zenith.bonus, zenith.incapacitated, zenith.revive, zenith.attack이다. 상대 ID와 도착·적용 순서가 필요하다. 부분 스트림만 가진 파일에서 동료 상태를 임의 생성하지 않는다.

추가 상태의 예시는 cancelstreak, stalepieces/staletime, hasseenI5, garbageahead, reviveprompts, rank_locked_until, bonusremaining, lastclear*다. 엔진 영역의 직접 참조 분석에서는 Zenith 상태 50개와 Zenith stats 14개 필드가 확인됐으나 이 숫자를 완전한 checkpoint 스키마로 사용하지 않는다. 통계처럼 보이는 값도 후속 실행에 사용된다.

## 검증과 미해결 계약

정상 9종·반전 9종 각각과 조합, 층 경계, cancelstreak 공급 임계값, fatigue 시각 전후, Duo 동시 무력화, revive 제외·중복·순서, 12frame 예약과 permanent 행 재구성, 외부 사건 순서가 필요하다. 프리셋 없이 같은 옵션을 조합한 경우도 [커스텀 규칙](../options/custom-rooms.md) 범위에서 평가한다.

서버의 카드 확장·추가 설정 적용 순서, 초기 상수의 누락 여부, 비결정적 TEMP 입력, 부분 동료 스트림, snowman 활성 여부는 미확인이다. 현재 소스의 정적 분기를 구현 완료·운영 모드 지원 완료로 표시하지 않는다.
