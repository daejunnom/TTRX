# 변환기 검증 기록

기준일: 2026-09-12. 이 문서는 `source-semantic` 구현에 실제로 수행한 검증과 그 증명 범위를 기록한다.

## 자동화 검증

| 검증 | 결과 |
|---|---|
| `cargo test --workspace --all-targets --locked` | 55 passed, 0 failed |
| `cargo test --workspace --doc --locked` | 성공; doctest 0개 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 성공; warning 0개 |
| `cargo build --release -p ttrx-cli --locked` | Windows release CLI 빌드 성공 |
| `cargo build --release -p ttrx-wasm --target wasm32-unknown-unknown --locked` | WASM release 빌드 성공 |
| `wasm-bindgen ... --target nodejs` | 고정한 schema 0.2.126으로 JavaScript glue 생성 성공 |
| `scripts/wasm-smoke.cjs` | Node.js 24.16.0에서 두 표본 encode/decode/inspect/verify 성공 |
| `cargo +1.85.0 check --workspace --all-targets --locked` | 선언한 최소 Rust 버전에서 전체 workspace check 성공 |
| `npm run build && npm run test:package` | Node.js CommonJS/ESM에서 내장 TTR/TTRM 왕복·손상 거부 성공 |
| `node scripts/pack-npm.mjs work/npm-pack` | `tetr-ttrx@0.1.0` tarball 생성 및 파일 allowlist 검사 성공 |
| 검증 tarball clean install | 별도 npm consumer에서 CommonJS/ESM import와 버전 호출 성공 |
| 검증 tarball `npm publish --dry-run --access public` | 공개 배포 dry run 성공; 15개 파일, 91,662 byte tarball |
| `npm publish work/npm-publish-tetr/tetr-ttrx-0.1.0.tgz --access public` | `tetr-ttrx@0.1.0`을 public `latest`로 최초 배포 성공 |
| 공개 registry clean install | `tetr-ttrx@0.1.0`을 새 consumer에 설치해 CommonJS 왕복과 ESM import 성공 |
| `npm trust list tetr-ttrx` | `daejunnom/TTRX`의 `publish.yml`, environment `npm`에 publish/stage publish OIDC 권한 등록 확인 |

자동화 테스트는 strict JSON parsing, 모든 JSON number 형식, surrogate와 UTF-8 오류, 객체 순서와 중복 key, TTR/TTRM shape 판별과 부분 손상 거부, 결정적 encoding, varint 경계와 non-canonical 표현 거부, CRC-32C known value, payload 손상·절단·trailing data·unknown version/tag 거부, 사전 반복·shape/table count의 메모리 증폭 예산, canonical 출력 길이 계산, CLI 입력 읽기 한도·인자와 Windows `--force` 파일 교체·디렉터리 거부, WASM source 확장자 보존을 포함한다. npm package 검사는 공개 함수 11개, 두 replay 종류의 encode/decode/inspect/verify, canonical JSON data model 일치, checksum 손상 거부와 두 Node.js module loader를 확인한다.

최초 npm 배포물의 registry integrity는 로컬 검증 tarball과 같은 `sha512-QwOhOhWYPRXYqiDTnNJgusGzLMgDUOvs+2uzx4pwawZmATHlMCNTnhRA/8Z6H2mqV9cndxsOhZ/+zpyDU4EDcw==`이다. 최초 버전은 패키지가 존재해야 Trusted Publisher를 등록할 수 있는 npm 제약 때문에 대화형 2FA로 배포했다. 후속 GitHub Release는 `.github/workflows/publish.yml`이 검증 tarball을 별도 job에서 만들고, `id-token: write`를 가진 publish job이 장기 npm token 없이 배포하도록 구성되어 있다.

## 실제 표본 왕복 변환

표본 SHA-256은 [표본·근거](../tetrio/evidence/fixtures-and-references.md)에 기록한 기준과 다시 대조했다.

| 항목 | Single-player `.ttr` | Multiplayer `.ttrm` |
|---|---:|---:|
| 원본 파일 | `a0d49c30cc1e.ttr` | `versus pulsar_.ttrm` |
| 원본 byte | 137,704 | 458,602 |
| 원본 SHA-256 | `4a6ccf1c0247a5abd9d7190e6bdf64b3f20767f7e82c7a683b34a31b1d1c9876` | `a8526dab6b28e43564a1376a9126ecf9a67a125ab2183cba13e42dbc5d1cd05f` |
| replay stream | 1 | 12 |
| event | 1,907 | 5,632 |
| `.ttrx` byte | 26,708 | 87,377 |
| 원본 대비 | 19.40% | 19.05% |
| `.ttrx` SHA-256 | `ae0ad19c4e3d7eb24681ebbf4c9eae9ace70c7368ff260233cea6bf8661df8f1` | `441a867579c727d20377e93a2df8e822a9dd01915d571e788eecc2b44c88170c` |
| decoded JSON data model | 원본과 일치 | 원본과 일치 |
| decoded 파일 SHA-256 | 원본과 일치 | 원본과 일치 |
| decode 후 재-encode `.ttrx` | 최초 `.ttrx`와 byte 일치 | 최초 `.ttrx`와 byte 일치 |

두 표본의 decoded 파일이 원본과 byte까지 같았던 것은 원본이 현재 canonical serializer와 같은 표현을 사용했기 때문이다. 형식 계약은 JSON 공백과 동등한 escape spelling을 보존하지 않으므로 모든 입력에서 byte 동일성을 약속하지 않는다.

## 실패 경로

실제 single-player `.ttrx`의 payload 한 byte를 바꾼 뒤 CLI `verify`를 실행했다. CLI는 exit code `1`과 `payload checksum mismatch`를 반환했다. `.ttrx`를 `.bin`으로 복사한 입력은 확장자 대신 `TTRX` magic으로 container를 판별하고 정상 검증했다.

## 이 검증이 증명하는 것

- 지원하는 두 replay 외형을 strict JSON으로 읽고 전용 binary tree에 기록할 수 있다.
- 현재 parser의 전체 JSON data model을 `.ttrx` 왕복에서 보존한다.
- 실제 두 표본에서 CLI와 WASM 경로가 동일하게 동작한다.
- 형식이 선언한 checksum, 통계와 구조 제약을 decoder가 검사한다.

이 결과는 TETR.IO runtime을 별도로 실행한 placement trace 비교가 아니다. `source-semantic` profile은 원본 event와 값을 그대로 보존하기 때문에 변환 과정에서 입력 timing이나 미노 배치를 다시 만들지 않는다. 행동 compilation과 계산 가능한 상태 생략은 [후속 검증 항목](../tetrio/verification/coverage-and-open-items.md)이 충족된 별도 profile에서 다룬다.
