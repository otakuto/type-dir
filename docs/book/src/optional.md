# `optional` — allow an entry to be absent

Adding `optional: true` to an entry allows it to be absent without error. When it does exist, it is validated against the contents declared under `::`.

Without `optional:`, every `file:` and `dir:` entry is treated as required. Use `optional:` when you want to declare an entry that may or may not be present.

## Directory layout

```text
<!-- cmdrun cd ../../../ && tree tutorials/optional --noreport -->
```

## Rule definition

```yaml
{{#include ../../../tutorials/optional/.type-dir.yaml}}
```

Because `required.txt` has no `optional:`, its absence causes `LT002 required name not found`. In contrast, `extra.txt` and `opt.txt` both have `optional: true`, so their absence does not cause an error.

In this tutorial directory, `extra.txt` exists on disk, but `opt.txt` does not. Even though `opt.txt` is declared in the rule, `type-dir check` succeeds. This contrast illustrates what `optional: true` means: when the entry exists it is validated against the contents declared under `::`, and when it does not exist nothing happens.

An entry marked `optional: true` means "if this name is present, accept it as this entry." If a file or directory exists in the directory under a name that is not declared at all, it becomes `LT001 undeclared path` regardless of whether `optional:` is used. `optional:` means "declared but not required to exist," not "may exist without being declared."
