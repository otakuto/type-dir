# `with` によるルール渡しとレコードのパラメータ化

`type: rule.<ルール名>` の形式でルール型のパラメータを宣言できる。あるルールが収集したレコードを `value: ${...}` で別のルールに渡すことで、ソース側で捕捉した構造を別の場所の検証に利用できる。

## ディレクトリ構成

```text
<!-- cmdrun cd ../../../ && tree tutorials/rule-with-rule --noreport -->
```

## ルール定義

```yaml
{{#include ../../../tutorials/rule-with-rule/.type-dir.yaml}}
```

## 解説

### component の捕捉

`components/` ディレクトリ内では `use: rule.component` に `id: comp` を付けて呼び出す。`component` ルールは `'^(?<name>.+)$'` の regex で各サブディレクトリを捕捉し、`id: component` でそのレコードを束ねる。`id: comp` を付けることで、`${dir.components_root.dir.comp.dir.component}` という参照パスで収集結果にアクセスできるようになる。

`dir: components` には `id: components_root` を付与している。これにより、`components/` ディレクトリ全体のレコード(内部の `comp` 捕捉を含む)を1つのオブジェクトとして参照できる。

### components_root を docs ルールへ渡す

`root` ルールは `use: rule.docs` を呼び出す際に、`with:` で `components_root` を渡す。渡す値は `${dir.components_root}` であり、型は `type: rule.root.dir.components_root` として宣言する。この型宣言は「`root` ルールの `components_root` という捕捉の形状」を意味する。

`docs` ルールは受け取った `components_root` を `for` で反復する。反復対象は `${with.components_root.dir.comp.dir.component}` であり、各要素を `component_entry` として取り出す。ルール本体では `${value.component_entry.regex.name}` で named capture `name` を参照し、`<name>.md` というファイルを要求する。

たとえば `components/` に `button/` と `input/` があれば、`docs/button.md` と `docs/input.md` の両方が必須になる。

このパターンにより、`components/` で捕捉した構造を丸ごと `docs` ルールに渡し、`docs/` の検証に使うことができる。`for` を呼び出し元ではなくルール内部に持つことで、反復ロジックを `docs` ルールにカプセル化できる。ディレクトリ間の対応関係を型安全に表現したい場合に有効である。
