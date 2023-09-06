# SCGT Syntax overview

## Guide

### Closing characters
* `["]` - only strings
* `[;]` - closing character for everything else
* a newline closes **everything**

### Implicit printing at statement level
* denoted by `[$]`

### Code placeholders
* `b` - block
    * ( `s` )*
* `d` - dictionary
    * `m` ( [ { ( `i` | `v` ) `:` }+ `v` | { `i` | `v` } `v` | `v` { `,` }? ] )*
    * with single character modifier: `m` ( [ { `j` | `w` }+ `:v` | `w,` | `w` ] )*
    * modifiers only apply to values, not keys
    * chain like `v:v:v` or `cc:v` to assign all keys in between to the value after the last `:`
    * `,` after a key to add it as a key-value pair (`v,` → `v: v`)
        * should also work with values because of `Od`
        * also works like this if the identifier is the last one in the dict
* `i` - identifier
    * matches `[a-z]+`
    * identifiers may have one trailing space to separate them from the next one
* `j` - identifier limited to one character - see `i` above
* `l` - list
    * `m` ( `v` )*
* `m` - optional modifiers - see [**Modifiers**](#modifiers)
* `s` - statement - see [**Statements**](#statements)
* `v` - expression
    * expressions are just values chained with operators
    * see [**Values and Prefixes**](#values-and-prefixes) and [**Postfixes**](#postfixes)
    * expressions may have one trailing space to separate them from the next one
* `w` - value limited to one character - see `v` above
* `x` - any character
* `…` - any sequence of characters (until closed accordingly)
* `z` - see notes for the entries that use this

## Statements
* `[ ] [ ]` - `v!` calls a trigger function using helper function `_scgt_trg_fn(v)`
* `[ ] [ ]` - `i@d` declares a type `i` with the members in `d`
* `[ ] [$]` - expression (at least one binary operator)
* `[ ] [?]` - value
    * print behavior depends on the value
    * see [**Values and Prefixes**](#values-and-prefixes) and [**Postfixes**](#postfixes)

## Modifiers
Only in this order
* `$` debug printing
* any sequence of digits `[0-9]+` to be added to the end of something
* any type(s) (except consecutive duplicates) - see also [**Built-in types**](#built-in-types)
    * defaults to `N` if a digit modifier is used
* `:` single character list - limits any following identifiers and/or values to one character
* `␣` optional end of modifier list indicator if following value could be a modifier too

## Values and prefixes
* `[ ] [$]` - `!v` "inverts" using helper function `_scgt_inv(v)`
* `[ ] [$]` - `@i` same as SPWN's type indicators
* `[ ] [ ]` - `$v` for explicit printing
    * returns the unmodified value
* `[ ] [$]` - `'x` represents a string containing the following character only
* `["] [$]` - \…` starts a string starting with an escape char
* `["] [$]` - `"…` starts a regular string
* `[;] [$]` - `)z;b` defines a macro
    * `z`: `m` ( `imv` [ `,` ]? )*
    * `,` needed to separate macro argument definitions because of possible default values
        * `␣` or no delimiter also works when unambiguous
* `[;] [$]` - `[l` defines array
* `[;] [$]` - `{d` defines dictionary
* `[;] [$]` - `}b` defines trigger function
* `[ ] [$]` - `A` equivalent to SPWN `[]`
* `[ ] [$]` - `B` equivalent to SPWN `?b`
* `[ ] [$]` - `C` equivalent to SPWN `?c`
* `[ ] [$]` - `D` equivalent to SPWN `?i`
* `[ ] [ ]` - `Eb` roughly equivalent to SPWN `on(touch(), b)` - see `vEb` entry [below](#postfixes)
* `[ ] [$]` - `F` equivalent to SPWN `false`
* `[ ] [$]` - `G` equivalent to SPWN `?g`
* `[ ] [$]` - `I` / `J` / `K` for loop variables
* `[;] [ ]` - `Lb` for infinite loop
* `[;] [$]` - `Mb` for macro def with no args
* `[ ] [$]` - `N` equivalent to SPWN `null` / `()`
* `[;] [ ]` - `Od` adds an object
* `[ ] [$]` - `S` equivalent to SPWN `""`
* `[ ] [$]` - `T` equivalent to SPWN `true`

## Postfixes
* `[;] [ ]` - `i!v` assigns a value and returns it
* `[ ] [ ]` - `v?v` ( `v` )? for ternary operator
    * *TODO fix the else syntax, this wont work*
* `[ ] [$]` - `v.i` for accessing children
* `[;] [ ]` - `v(z` calls a macro
    * `z`: `m` ( ( `i:` )? `v` )*
    * `i:v` when calling to specify the arg that gets the value
* `[;] [$]` - `v]z` for indexing/slicing
    * *TODO think of syntax for slice*
* `[;] [$]` - `v}v` dictionarize / (multi-)zip?
    * *TODO for like matrix stuff? idk look at more stuff*
* `[;] [ ]` - `vEb` roughly equivalent to SPWN `on(v, b)`
    * *TODO find out how event system works and which conversions should happen*
* `[;] [ ]` - `vIb` / `vJb` / `vKb` to start for loop with corresponding variable using helper function `_scgt_iter(v)`
* `[;] [ ]` - `vLb` starts a while loop using helper function `_scgt_bool(v)`
* `vM` for macro call with no args
* `[;] [ ]` - `vWb` starts a runtime while loop
* `[ ] [$]` - any type(s) (except consecutive duplicates) - converts to types in order
    * see also [**Built-in types**](#built-in-types)
* operators
    * `v` ( op `v` )*
    * *TODO what is replacement for parantheses?*

## Built-in types
* `A`: `@array`
* `B`: `@block`
* `C`: `@color`
* `D`: `@item`
* `G`: `@group`
* `N`: `@number`
* `O`: `@object` (automatically adds to the level)
* `S`: `@string`
* `T`: `@bool`
