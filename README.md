# Clyde - a tool for querying Rust programs

Supports syntactic and semantic querying of Rust programs and program transformations based on those queries.
Intends to facilitate code exploration, sophisticated search and replace, and refactoring.

Very work in progress.

## Example

```
show (:src/back/mod.rs:10:38).idents.pick.def
```

## Notes on language

Comments are `#` comments.

### Statements

Statements may be terminated with an optional `;`

* Expression: `expr`
* TODO Variable assignment: `name '=' expr`
* TODO Concatenation: `name '+=' expr`
* Meta-command: `'^' command`
* TODO in-place function application: `name '<-' name ['(' args ')']`
* zero-arg function shorthand `name [flags] expr`

### Expressions

* Parens: `(expr)`
* Locations: `'('':'name[:line[:column]]')'` - name is a string, line and column are unsigned ints
* function application: `expr '->' name [flags] ['(' args ')']`
* field projection/sequence indexing: `expr '.' name`
* TODO(named) variables: `name | '$' | '$' n`
* TODO path: `'('['::'name]+')'`

### Commands

`^cmd flags`

* `exit` (`q`)
* `help` (`h`)
* TODO `fmt`
* TODO `build`/`check`
* TODO `type expr` print meta-type info
* TODO `debug expr` print metadata
* TODO `clear [$n]` clear back to line n (or clear all)

### Functions

* `show`: `query -> string` make a short text representation
  - redirect to file
  - long form
  - list form
  - short form
* `select`: `query -> set` evaluate a query
* TODO `eq`: `T, T -> T?` equality
* TODO `match`: `string:T, regex -> T?` regex matching
* TODO `find`: `string|regex|def|ident -> set<ident>` find all refs
* TODO `is`: `item, item-kind -> item?`
* TODO `items`: `item -> set<item>`
* TODO `type`: `ident, type -> ident?`
* TODO `in`: `T, location -> T?`
* TODO `replace`: `expr|item|range, string|<replace block> -> ()`
* TODO `blame`: `location -> list<text>`
* TODO `args`: `ident -> set<expr>`
* TODO `where`: `set<T>, query<T -> U?> -> set<T>`

How to specify?

* select field/var reads vs writes $:expr->is(assign): assign?
* select def vs ref $:ident->is(def): ident?
* fancy replace using parts of the original
  - `(::**::test::foo*)->is(fn).args->replace({first = @[0]; rest = @[1..]; name = @-2.name; "{rest}, {name}: Option<{first.type}>"})`
* generate code based on query; need to specify where to generate the code
* move code
* ident.item: item?, ident.expr: expr?, etc. to get to the enclosing ast node


### Objects

* `location`
  - `idents: set<ident>`
* `position: location`
* `range: location`
* `set`
  - TODO `count: n`
  - `pick`
* `list`
* `identifier`
  - TODO `name: string`
  - TODO `span: range`
  - TODO `type: type`
  - `def: def`
* `def` a chain of definitions
  - TODO `primary: item`
  - TODO `list<item>`
* `item`
  - TODO `kind: def-kind`
  - TODO `span: range`
  - TODO `focus: range`
  - TODO `src: string`
  - TODO `doc: string`
  - TODO `sig: string`
* `type`
  - TODO `def: item`
  - TODO `ident: ident?`

### Coercions

* TODO `expr` -> `query`
* TODO `set<T>*1` -> `T`
* TODO `set<T>*0` <-> `()`

### Variables

TODO

Named variables can be any alphanumeric name + underscores, must start with a letter

Named variables are mutable.

`$` is the result of the previous statement.

`$n` is the result of the nth statement.
`$-n` is the result of the previous nth statement

`$` variables are immutable
