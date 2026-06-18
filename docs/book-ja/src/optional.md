# `optional` による省略可能なエントリの宣言

`optional: true` を付けると、そのエントリは存在しなくても許可される。存在した場合は `::` で指定した内容に従って検証される。

`optional:` を使わない場合、`file:` や `dir:` エントリはすべて必須扱いになる。

## ディレクトリ構成

```text
<!-- cmdrun cd ../../../ && tree tutorials/optional --noreport -->
```

## ルール定義

```yaml
{{#include ../../../tutorials/optional/.type-dir.yaml}}
```

`required.txt` には `optional:` がないため、存在しなければ `LT002 required name not found` が報告される。一方、`extra.txt` と `opt.txt` には `optional: true` が付いているため、存在しなくてもエラーにならない。

このチュートリアルのディレクトリには `extra.txt` は存在するが、`opt.txt` は存在しない。`opt.txt` はルールに宣言されているにもかかわらず、`type-dir check` は成功する。これが `optional: true` の意味を端的に示している。存在すれば `::` で指定した内容に従って検証され、存在しなければ何も起きない。

`optional: true` を付けたエントリは「存在したらそのエントリとして受理する」という意味を持つ。宣言されていない名前のファイルがディレクトリに存在した場合は、`optional:` の有無にかかわらず `LT001 undeclared path` が報告される。「宣言されているが存在しなくてもよい」という意味であり、「宣言なしでも存在してよい」という意味ではない。
