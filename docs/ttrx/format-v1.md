# TTRX 1.0 source-semantic 바이너리 형식

기준일: 2026-09-12. 이 문서는 현재 Rust 구현이 읽고 쓰는 `source-semantic` profile의 상호운용 명세다. TETR.IO 게임 규칙을 실행해 행동이나 보드를 다시 계산하는 형식은 이 profile의 범위가 아니다.

## 기본 규칙

- 모든 고정폭 정수는 unsigned little-endian이다.
- 가변폭 unsigned 정수는 canonical unsigned LEB128이다. 같은 값을 더 긴 byte열로 표현한 non-canonical varint는 거부한다.
- signed 정수는 `0, -1, 1, -2, 2, ...` 순서가 되는 zigzag 값을 unsigned LEB128로 기록한다.
- 문자열은 Unicode scalar 값으로 검증된 UTF-8이다. 사전 entry에는 NUL 종결자를 붙이지 않는다.
- checksum은 CRC-32C Castagnoli다. 표준 check value `CRC32C("123456789") = 0xe3069283`을 사용한다.
- 하나의 파일에는 하나의 header와 하나의 payload만 존재한다. 선언한 payload 뒤의 byte도 오류다.

## 72-byte header

| Offset | 크기 | 필드 | 1.0 값 또는 의미 |
|---:|---:|---|---|
| 0 | 4 | magic | ASCII `TTRX` (`54 54 52 58`) |
| 4 | 1 | format major | `1` |
| 5 | 1 | format minor | `0` |
| 6 | 1 | profile | `1`: `source-semantic` |
| 7 | 1 | source kind | `0`: unknown, `1`: `.ttr`, `2`: `.ttrm` |
| 8 | 2 | flags | `0`; 다른 bit가 있으면 거부 |
| 10 | 2 | header length | `72` |
| 12 | 4 | reserved | `0` |
| 16 | 8 | original JSON length | 입력 JSON의 byte 수. 복원 JSON 길이와 같을 필요는 없으며 64 MiB 이하여야 한다. |
| 24 | 8 | payload length | header 뒤 payload의 정확한 byte 수 |
| 32 | 4 | payload CRC-32C | payload 전체에 대한 checksum |
| 36 | 4 | reserved | `0` |
| 40 | 8 | dictionary entry count | payload 앞부분에서 읽을 문자열 수 |
| 48 | 8 | shape entry count | 문자열 사전 뒤에서 읽을 객체 shape 수 |
| 56 | 8 | value count | root를 포함한 JSON 값 node 수 |
| 64 | 4 | maximum depth | root 깊이를 `1`로 한 최대 node 깊이 |
| 68 | 4 | header CRC-32C | offset `0..68`에 대한 checksum |

major 또는 minor가 구현값과 다르거나, profile/source kind가 알려지지 않았거나, reserved/flags/header length가 예상값과 다르면 디코더는 파일을 거부한다. Header CRC를 확인한 뒤 길이와 자원 한도를 확인하고, payload CRC를 확인한 뒤 테이블과 값을 해석한다.

## Payload

Payload는 다음 세 구역을 구분 byte 없이 이어 붙인다. 각 구역의 entry 수는 header에 있으므로 경계를 계산할 수 있다.

1. 문자열 사전 `dictionary[0..dictionary_entry_count]`
2. 반복 객체 key shape 표 `shapes[0..shape_entry_count]`
3. 하나의 root JSON 값

### 문자열 사전

각 entry는 `byte_length: varuint` 다음에 그 길이만큼의 UTF-8 byte가 온다. Encoder가 JSON tree를 depth-first로 방문하면서 처음 만난 순서대로 다음 문자열을 한 번씩 등록한다.

- 객체 member key
- JSON string 값
- 정수로 정규화할 수 없는 JSON number의 정확한 lexeme

문자열 내용이 같으면 용도가 달라도 같은 사전 entry를 참조한다. 따라서 number tag가 사전 문자열을 참조할 때 디코더는 그 문자열이 유효한 JSON number이며 정수 전용 tag로 표현할 수 없는 lexeme인지 다시 확인해야 한다.

### 객체 shape 표

객체 shape는 member key의 순서 있는 사전 ID 배열이다. 중복 key도 배열에 그대로 남는다. 전체 tree에서 같은 shape가 두 번 이상 나타난 경우에만 표에 넣으며, 해당 shape를 처음 만난 객체 순으로 ID를 부여한다.

각 shape entry는 `member_count: varuint`와 이어지는 `member_count`개의 `key_dictionary_id: varuint`로 구성한다.

### JSON 값

각 값은 1-byte tag로 시작한다.

| Tag | 이름 | tag 뒤 encoding |
|---:|---|---|
| 0 | null | 없음 |
| 1 | false | 없음 |
| 2 | true | 없음 |
| 3 | unsigned integer | `value: varuint` |
| 4 | signed integer | 음수 `value: zigzag varint` |
| 5 | raw number | `number_dictionary_id: varuint` |
| 6 | string | `string_dictionary_id: varuint` |
| 7 | array | `length: varuint`, 이어서 원소 값 |
| 8 | inline object | `member_count: varuint`, key 사전 ID들, 이어서 member 값들 |
| 9 | shaped object | `shape_id: varuint`, 이어서 shape의 key 수만큼 member 값 |

Tag 3은 `0` 또는 leading zero가 없는 양의 십진 정수에만 사용한다. Tag 4는 `-` 뒤에 leading zero가 없는 음의 십진 정수에만 사용하며 `-0`에는 사용하지 않는다. 이 범위에서 `u64` 또는 `i64`에 들어가지 않는 정수, 소수, 지수 표기, `-0`은 tag 5로 원래 number lexeme를 보존한다.

Array와 object의 자식 값은 source 순서대로 재귀 encoding한다. Inline object는 모든 key ID를 먼저 기록한 다음 모든 값을 기록한다. Shaped object의 key ID는 shape 표가 제공한다. 이 구조는 객체 순서와 중복 key를 보존한다.

## Canonical JSON 역변환

역변환 JSON은 공백 없는 UTF-8로 생성한다. Object/array 순서, 중복 key, 문자열 값, boolean/null, number lexeme는 유지한다. 문자열 escape는 JSON에 필요한 문자만 escape하는 한 가지 표현으로 정규화한다. 따라서 원본 JSON과 byte-for-byte 같지 않을 수 있지만, 이 구현이 정의한 JSON data model은 같다.

## Decoder 정책 한도

현재 1.0 구현은 작은 악성 header가 과도한 할당이나 재귀를 유발하지 않도록 다음 한도를 적용한다. 이 값은 TETR.IO 규칙 한도가 아니라 구현의 입력 정책이다.

| 항목 | 한도 |
|---|---:|
| 입력 source JSON | 64 MiB |
| payload | 512 MiB |
| 문자열 한 entry | 16 MiB |
| 문자열 사전 entry | 1,000,000 |
| 문자열 사전의 UTF-8 byte 합계 | 64 MiB |
| shape entry | 1,000,000 |
| 모든 shape의 key 참조 합계 | 2,000,000 |
| array 원소 또는 object member | 1,000,000 |
| 전체 JSON value | 2,000,000 |
| 복원된 key·string·number UTF-8 byte 합계 | 64 MiB |
| canonical JSON 역변환 출력 | 64 MiB |
| binary data-model depth | 513 |

`original JSON length`도 64 MiB 이하인지 확인한다. 문자열 사전 byte 합계는 사전 자체의 메모리 사용을 제한하고, 복원 text byte 합계는 같은 사전 문자열을 반복 참조해 큰 `String` 할당을 유도하는 것을 제한한다. 역변환 전에는 escape와 JSON 구문을 포함한 정확한 canonical 출력 길이를 할당 없이 계산한다. Shape key 참조 합계는 payload의 작은 ID가 메모리에서 다수의 `u64`로 확장되는 양을 제한한다. Header의 dictionary·shape entry 수가 payload에 들어갈 수 있는 최소 byte 수보다 크면 table capacity를 할당하기 전에 거부한다.

디코더는 이 밖에도 잘린 입력, varint overflow, non-canonical varint, UTF-8 오류, 존재하지 않는 사전/shape 참조, 알 수 없는 tag, header의 value/depth 통계와 실제 payload 불일치, payload 뒤의 trailing data를 거부한다.

## 결정성

동일한 `JsonValue`, source kind와 original JSON byte length를 입력하면 encoder는 동일한 TTRX byte열을 만든다. 사전 ID는 최초 방문 순서, shape ID는 최초 객체 출현 순서로 결정한다. Map iteration order에 기대지 않는다.

Decoder는 의미가 같은 비정규 table 배치까지 거부하는 canonical-file validator는 아니다. 중복되거나 사용되지 않은 dictionary/shape entry, encoder라면 shape를 썼을 inline object도 참조·한도·통계가 유효하면 읽을 수 있다. 이 구현으로 다시 encode하면 위의 결정적 table 배치로 정규화된다.
