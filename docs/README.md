# TTRX 문서

기준일: 2026-09-12. 이 디렉터리는 기획 결정, 정적 소스 분석, 표본 관측, 현재 `source-semantic` 코덱의 명세·검증과 미해결 사항을 보관한다. 게임 규칙을 실행하는 `action-compiled` 엔진의 완료 보고서는 아니다.

## 읽는 순서

1. [요구사항과 대화에서 확정한 선택](ttrx/requirements.md)
2. [TETR.IO 분석 목차와 근거 수준](tetrio/README.md)
3. [커스텀 방: 프리셋 밖의 규칙과 변경](tetrio/options/custom-rooms.md)
4. [분석이 구현 설계에 미치는 영향](ttrx/implementation-implications.md)
5. [검증 범위와 미해결 항목](tetrio/verification/coverage-and-open-items.md)
6. [TTRX 1.0 source-semantic 바이너리 형식](ttrx/format-v1.md)
7. [현재 변환기 구현 상태](ttrx/implementation-status.md)
8. [변환기 검증 기록](ttrx/validation.md)

문서 자체의 목록·링크·배치 검사는 [문서 검증 기록](tetrio/verification/documentation-check.md)에 있다. 게임 실행 검증과 구분한다.

## 문서 구조

| 디렉터리 | 내용 |
|---|---|
| `ttrx/` | 사용자 요구사항, 채택한 기술 선택, 현재 binary 형식·구현·검증, 기존 계획에서 수정할 가정 |
| `tetrio/evidence/` | 공식 소스 식별, 표본 관측, 기계 판독용 옵션 목록 |
| `tetrio/options/` | 173개 옵션, 초기화, 커스텀 규칙, 사용자·뷰어 설정 |
| `tetrio/runtime/` | 입력·시간·물리·공급·공격·외부 사건·상태 |
| `tetrio/modes/` | 프리셋·매치·레벨·Survival·Zen·Zenith |
| `tetrio/replay/` | JSON 구조, importer/exporter, 기존 binary codec |
| `tetrio/verification/` | 게임 실행 구현 전 확인할 계약, 차등 검증 사례, 근거 한계 |

프리셋은 규칙 조합의 예시다. TTRX의 분석·지원 단위를 프리셋 이름으로 제한하지 않는다. 현재 문서는 실제 옵션 값, 별도 상수, 초기화 경로, 실행 중 변경과 외부 입력을 기준으로 정리한다.
