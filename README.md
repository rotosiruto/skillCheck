# skillCheck

`sysctl.conf(5)` 形式のファイルをネストされた `BTreeMap` として読み込む Rust ライブラリ。
スキーマファイルによる型検証も提供する。

## 課題

### 課題 1

> linux の `sysctl.conf` と同じ文法の任意のファイルをロードし、辞書型・Map 等に格納するプログラムを作成してください。
> プログラムはライブラリとしてファイルを Parse するのに利用する想定。

### 課題 2

> 課題 1 のプログラム実行時に、入力値に不備がないか検証できるようにしてください。
> 入力値に不備がないかは、スキーマファイルによって検証する。

参考: [`sysctl.conf(5)` man page](https://man7.org/linux/man-pages/man5/sysctl.conf.5.html)

## 対応する文法

| 種別 | 仕様 |
|---|---|
| エントリ | `key = value`。最初の `=` がセパレータで、それ以降はすべて値 |
| 余白 | キー / `=` / 値の前後の空白は除去 |
| コメント | trim 後に `#` または `;` で始まる行は無視 |
| 空行 | 無視 |
| `-` プレフィックス | sysctl の "ignore-errors" マーカー。strip して同じキーとして扱う |
| ネストキー | `.` で名前空間を作る。`log.file = x` → `{"log": {"file": "x"}}` |
| 重複キー | 末尾の代入で上書き |
| キー衝突 | `log = x` と `log.file = y` の混在は `ParseError::KeyConflict` |

詳細は `tests/syntax.rs` を参照。

## 使い方

`Cargo.toml`:

```toml
[dependencies]
skillcheck = { git = "https://github.com/rotosiruto/skillCheck" }
```

文字列から:

```rust
use skillcheck::Config;

let cfg: Config = "
    endpoint = localhost:3000
    # debug = true
    log.file = /var/log/console.log
    log.name = default.log
".parse()?;

assert_eq!(cfg.get_str("endpoint"), Some("localhost:3000"));
assert_eq!(cfg.get_str("log.file"), Some("/var/log/console.log"));
assert_eq!(cfg.get_str("log.name"), Some("default.log"));
assert_eq!(cfg.get_str("debug"),    None);
# Ok::<_, skillcheck::ParseError>(())
```

ファイルから:

```rust
use skillcheck::Config;

let cfg = Config::from_file("/etc/sysctl.conf")?;
# Ok::<_, skillcheck::ParseError>(())
```

`Config` を介さず Map を直接欲しい場合:

```rust
use skillcheck::{parse_str, Value};

let map = parse_str("a.b = c")?;
assert!(matches!(map.get("a"), Some(Value::Map(_))));
# Ok::<_, skillcheck::ParseError>(())
```

## 課題の入力例と出力

### 入力例 1

```text
endpoint = localhost:3000
debug = true
log.file = /var/log/console.log
```

```json
{
  "endpoint": "localhost:3000",
  "debug": "true",
  "log": { "file": "/var/log/console.log" }
}
```

### 入力例 2

```text
endpoint = localhost:3000
# debug = true
log.file = /var/log/console.log
log.name = default.log
```

```json
{
  "endpoint": "localhost:3000",
  "log": {
    "file": "/var/log/console.log",
    "name": "default.log"
  }
}
```

両方とも `tests/examples.rs` で `BTreeMap` レベルの完全一致を検証している。

## スキーマによる検証（課題 2）

スキーマファイルは課題 1 と同じ syntax を流用し、値側に型名を書く形式。

```text
# schema.conf
endpoint = string
debug = bool
log.file = string
retry = integer
ratio = float
```

サポートする型: `string` / `bool` / `integer` / `float`。

検証 API:

```rust
use skillcheck::{Config, Schema};

let schema = Schema::from_file("schema.conf")?;
let config = Config::from_file("app.conf")?;

match schema.validate(&config) {
    Ok(()) => println!("ok"),
    Err(errors) => {
        for e in errors {
            eprintln!("{e}");
        }
    }
}
# Ok::<_, Box<dyn std::error::Error>>(())
```

`validate` は最初のエラーで止めず、違反を全件 `Vec<ValidationError>` で返す。
ユーザーが一度のフィードバックで全箇所修正できることを優先した。

検出する違反:

| バリアント | 状況 |
|---|---|
| `MissingKey` | schema にあるが data にない |
| `UnknownKey` | data にあるが schema にない（strict） |
| `TypeMismatch` | 値が宣言された型に変換できない（例: integer に `"abc"`） |
| `ShapeMismatch` | schema が map なのに data はスカラー、または逆 |

スキーマ自体のロード時のエラーは `SchemaParseError`：

- `Parse(_)` — schema ファイルが文法的に壊れている、または I/O エラー
- `UnknownType { type_name }` — 知らない型名（例: `key = somethingweird`）

検証は型チェックのみで、`Value` の中身を `Integer(i64)` のような型付き値に変換しない。
変換が必要なら呼び出し側で `cfg.get_str("retry").unwrap().parse::<i64>()` する。

詳細は `tests/schema.rs` を参照。

## 設計メモ

- 内部表現: `BTreeMap<String, Value>` ＋ `enum Value { String, Map }`。`HashMap` ではなく `BTreeMap` を使うのは出力順を決定的にしてテスト・diff を安定させるため。
- 依存: `thiserror` のみ。文法が小さいので正規表現も parser combinator も不要。
- `#![forbid(unsafe_code)]`。
- エラーは行番号付き（`ParseError::Syntax { line, message }` / `KeyConflict { line, key }`）。
- 公開 API: `parse_str` / `parse_file`（生の Map を返す）と `Config`（`FromStr` 実装、`get("log.file")` でドット区切り検索）の 2 系統。
- `log = a` と `log.file = b` の混在は黙って上書きせずエラーにする。設定の意図が壊れるのを呼び出し側に気づかせるため。
- 出力順は辞書順。挿入順を保ちたいケースは要件発生時に検討。
- スキーマも config と同じパーサーを通すので、コメント・空行・ネストキー・`-` プレフィックスがそのまま使える。

## 実行

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```
