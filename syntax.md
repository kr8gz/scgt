# SCGT Syntax overview

## Legend

### Closing characters
* `"` - only strings
* `;` - closing character for everything else

### Code placeholders
* `b` - block
* `d` - dictionary
    * *TODO deez notes*
    * chain `v:v` to assign all keys in between to the value after the last `:`
    * `,` after a key to add it as a key-value pair (`v,` → `v: v`)
        * should also work with values because of `Od`
        * also works like this if the identifier is the last one in the dict
* `i` - identifier
* `l` - list
    * *TODO deez notes*
* `s` - statement
* `v` - value/expression
* `x` - any character
* `…` - any sequence of characters (until closed accordingly)

## Statements
* `v!` calls a trigger function using helper function `_scgt_trg_fn(v)`
    * *TODO how to syntax for multiple assignment `a = b = c`? since inline assignment is also `!`*
* `;` - `i@d` declares a type `i` with the members in `d`

## Modifiers
TODO this entire section sucks
* `:` single char list
* `$` debug printing
* `0` any sequence of numbers to be added to the end of something
* any type(s) (except consecutive duplicates) - see also [**Built-in types**](#built-in-types)

## Values and prefixes
* `!v` "inverts" using helper function `_scgt_inv(v)`
* `@i` same as SPWN's type indicators
* `$v` for explicit printing
    * returns the unmodified value
* `'x` represents a string containing the following character only
* `"` - `\…` starts a string starting with an escape char
* `"` - `"…` starts a regular string
* `;` - `(l;b` defines a macro
    * *TODO deez notes*
    * `,` needed to separate macro argument definitions because of possible default values
        * `␣` or no delimiter also works when unambiguous
* `;` - `[l` defines array
* `;` - `{d` defines dictionary
* `;` - `}b` defines trigger function
* `A` equivalent to SPWN `[]`
* `B` equivalent to SPWN `?b`
* `C` equivalent to SPWN `?c`
* `D` equivalent to SPWN `?i`
* `F` equivalent to SPWN `false`
* `G` equivalent to SPWN `?g`
* `I` / `J` / `K` for loop variables
* `;` - `Lb` for infinite loop
* `;` - `Mb` for macro def with no args
* `N` equivalent to SPWN `null` / `()`
* `;` - `Od` adds an object
* `S` equivalent to SPWN `""`
* `T` equivalent to SPWN `true`

## Postfixes
* `;` - `i!v` for inline assignment
* `v?vv` for ternary operator
* `v.i` for accessing children
* `;` - `v)l` calls a macro
    * `i:v` when calling to specify the arg that gets the value
* `;` - `v]z` for indexing/slicing
    * *TODO think of syntax for slice*
* `;` - `v}v` dictionarize / (multi-)zip?
    * *TODO for like matrix stuff? idk look at more stuff*
* `;` - `vIb` / `vJb` / `vKb` to start for loop with corresponding variable using helper function `_scgt_iter(v)`
* `;` - `vLb` starts a while loop using helper function `_scgt_bool(v)`
* `vM` for macro call with no args
* any type(s) (except consecutive duplicates) - converts to types in order
    * see also [**Built-in types**](#built-in-types)

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

## Misc
* `␣` needed to separate values when greedy value parsing is unwanted
