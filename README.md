# dir-lint

A linter that validates directory structure against declarative YAML rules.

Source code has types; directory layout usually doesn't. `dir-lint` lets you
declare the structure a project is supposed to have in a `.dir-lint.yaml` file
and checks the real directory tree against it, reporting any drift with
compiler-style diagnostics. Think of it as a type checker for your file tree.

## Example

Suppose every source file under `src/` must have a
matching doc under `docs/` — `aaa.rs` requires `docs/aaa.md`, `bbb.rs` requires
`docs/bbb.md`, and so on:

```text
.
├── .dir-lint.yaml
├── docs
│   ├── aaa.md
│   ├── bbb.md
│   └── zzz.md
└── src
    ├── aaa.rs
    ├── bbb.rs
    └── zzz.rs
```

```yaml
version: 0

entry: root

rules:
  - rule: root
    ::
      - dir: src
        id: src
        ::
          # capture each <name>.rs under src/
          - file:
              regex: '^(?<name>.+)\.rs$'
            id: modules
      - dir: docs
        ::
          # require a matching <name>.md under docs/ for each one
          - for:
              id: module
              value: ${dir.src.file.modules}
            ::
              - file: '${value.module.regex.name}.md'
      - file: .dir-lint.yaml
```

`src/` collects every `<name>.rs` into `modules`, and `docs/` uses `for` to bind
each as `module` and require a `<name>.md` for it. Add `src/ccc.rs` and
`docs/ccc.md` immediately becomes mandatory; forget it and the check fails.

## Installation

```console
$ git clone https://github.com/otakuto/dir-lint.git
$ cd dir-lint
$ cargo install --path .
```

## Documentation

A complete guide — regex names, counts, `one_of` / `any_of` / `choice`, rule
reuse, recursion, and source/test mirroring — is available as an mdBook:

- English: [`docs/book`](docs/book)
- Japanese: [`docs/book-ja`](docs/book-ja)
