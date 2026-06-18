# Passing rules with `with` — parameterizing with records

You can declare a rule-typed parameter using the form `type: rule.<rule-name>`. By passing the records that one rule has captured to another rule via `value: ${...}`, you can achieve a pattern where "the structure captured on the source side is used by a separate rule to validate a different location."

## Directory structure

```text
<!-- cmdrun cd ../../../ && tree tutorials/rule-with-rule --noreport -->
```

## Rule definition

```yaml
{{#include ../../../tutorials/rule-with-rule/.type-dir.yaml}}
```

## Explanation

### Capturing components

Inside the `components/` directory, `use: rule.component` is called with `id: comp` attached. The `component` rule captures each subdirectory using the regex `'^(?<name>.+)$'` and groups the records under `id: component`. Adding `id: comp` makes the collected results accessible via the reference path `${dir.components_root.dir.comp.dir.component}`.

The `dir: components` entry is given `id: components_root`. This allows the entire `components/` directory record — including the inner `comp` captures — to be referenced as a single object.

### Passing components_root to the docs rule

When the `root` rule calls `use: rule.docs`, it passes `components_root` via `with:`. The value is `${dir.components_root}` and its type is declared as `type: rule.root.dir.components_root`. This type declaration means "the shape of the `components_root` capture inside the `root` rule."

The `docs` rule iterates over the received `components_root` using `for`. The iteration target is `${with.components_root.dir.comp.dir.component}`, and each element is bound as `component_entry`. Inside the rule body, `${value.component_entry.regex.name}` references the named capture `name` and requires the file `<name>.md`.

For example, if `components/` contains `button/` and `input/`, then both `docs/button.md` and `docs/input.md` become mandatory.

### Summary

This pattern passes the entire structure captured under `components/` to the `docs` rule and uses it to validate the `docs/` directory. Keeping `for` inside the rule rather than at the call site encapsulates the iteration logic within `docs`. It is an effective pattern when you want to express cross-directory correspondence relationships in a type-safe manner.
