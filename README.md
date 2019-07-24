# Clyde - a tool for querying Rust programs

Supports syntactic and semantic querying of Rust programs and program transformations based on those queries.
Intends to facilitate code exploration, sophisticated search and replace, and refactoring.

Very work in progress.

## Notes on language

Comments are `#` comments.

### Statements

Statements may be terminated with an optional `;`

* Expression: `expr`
* TODO Variable assignment: `name '=' expr`
* TODO Concatenation: `name '+=' expr`
* Meta-command: `'^' command`
* TODO in-place function application: `name '<-' name ['(' args ')']`
* TODO zero-arg function shorthand name [flags] expr

TODO remove `show` statement, `select*`

### Expressions

* Parens: `(expr)`
* Locations: `'('':'name[:line[:column]]')'` - name is a string, line and column are unsigned ints
* TODO function application: `expr '->' name [flags] ['(' args ')']`
* TODO field projection: `expr '.' name`
* TODO variables: `name | '$' | '$' n`
* TODO path: `'('['::'name]+')'`

### Commands

* `exit`
* `help`
* `fmt`
* `build`/`check`
* `type expr` print meta-type info
* `debug expr` print metadata
* `clear $n` clear back to line n

### Functions

* TODO `show`: `query -> string` make a short text representation
  - redirect to file
  - long form
  - list form
  - short form
* TODO `select`: `query -> set` evaluate a query
* TODO `eq`: `T, T -> T?` equality
* TODO `match`: `string:T, regex -> T?` regex matching
* TODO `find`: `ident -> set<ident>` find all refs
* TODO `def`: `ident -> def` find definition
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
  - `count: n`
  - `pick`
* `list`
* `identifier`
  - `name: string`
  - `span: range`
  - `type: type`
* `def` a chain of definitions
  - `primary: item`
  - `list<item>`
* `item`
  - `kind: def-kind`
  - `span: range`
  - `focus: range`
  - `src: string`
  - `doc: string`
  - `sig: string`
* `type`
  - `def: item`
  - `ident: ident?`

### Coercions

* `expr` -> `query`
* `set<T>*1` -> `T`
* `set<T>*0` <-> `()`

### Variables

Named variables can be any alphanumeric name + underscores, must start with a letter

Named variables are mutable.

`$` is the result of the previous statement.

`$n` is the result of the nth statement.
`$-n` is the result of the previous nth statement

`$` variables are immutable
