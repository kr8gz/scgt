# SCGT Syntax overview

## Guide

### Closing characters
* ``[`]`` - only strings
* ``[;]`` - closing character for everything else
* a newline closes **everything**

### Implicit printing at statement level
* denoted by ``[$]``

### Code placeholders
* ``b`` - block
    * ( ``s`` )*
* ``d`` - dictionary
    * ``m`` ( [ { ( ``i`` | ``v`` ) ``:`` }+ ``v`` | { ``i`` | ``v`` } ``v`` | ``v`` { ``,`` }? ] )*
    * with single character modifier: ``m`` ( [ { ``j`` | ``w`` }+ ``:v`` | ``w`` { ``,`` }? ] )*
    * modifiers only apply to values, not keys
    * chain like ``v:v:v`` or ``cc:v`` to assign all keys in between to the value after the last ``:``
    * only idents in dictionaries and only numbers in objects (no value keys)
    * ``,`` after a key to add it as a key-value pair (``v,`` → ``v: v``)
        * should also work with values in ``Od``
        * also works like this if the identifier is the last one in the dict
        * also works like this for all keys with no value specified in for character key list
* ``i`` - identifier
    * matches ``[a-z]+``
    * identifiers may have one trailing space to separate them from the next one
* ``j`` - identifier limited to one character - see ``i`` above
* ``l`` - list
    * ``m`` ( ``v`` )*
* ``m`` - optional modifiers - see [**Modifiers**](#modifiers)
* ``s`` - statement - see [**Statements**](#statements)
* ``v`` - expression
    * expressions are just values chained with operators
    * see [**Values and Prefixes**](#values-and-prefixes) and [**Postfixes**](#postfixes)
    * expressions may have one trailing space to separate them from the next one
        * *TODO check_with_state to check if outermost expression idfk*
* ``w`` - value limited to one character - see ``v`` above
* ``x`` - any character
* ``…`` - any sequence of characters (until closed accordingly)
* ``z`` - see notes for the entries that use this

## Statements
no statements everything is just a value either implicitly printable or not

## Modifiers
Only in this order
* ``$`` debug printing
* any sequence of digits ``[0-9]+`` to be added to the end of something
* any type(s) (except consecutive duplicates) - see also [**Built-in types**](#built-in-types)
    * defaults to ``N`` if a digit modifier is used
* ``:`` single character list - limits any following identifiers and/or values to one character
* ``␣`` optional end of modifier list indicator if following value could be a modifier too

## Values and prefixes
* ``[ ] [$]`` - ``!v`` "inverts" using helper function ``_scgt_inv(v)``
* ``[ ] [$]`` - ``@i`` same as SPWN's type indicators
* ``[ ] [ ]`` - ``$v`` for explicit printing
    * returns the unmodified value
* ``[ ] [$]`` - ``'x`` represents a string containing the following character only
* ``[`] [$]`` - `` `…`` starts a regular string
* ``[`] [$]`` - ``\…`` starts a string that may contain newlines
    * this means that it *must* be closed before anything else
* ``[;] [$]`` - ``(b`` evaluates a block as a value
* ``[;] [$]`` - ``)z;b`` defines a macro
    * ``z``: ``m`` ( ``imv`` [ ``,`` ]? )*
    * ``,`` needed to separate macro argument definitions because of possible default values
        * ``␣`` or no delimiter also works when unambiguous
* ``[;] [$]`` - ``[l`` defines array
* ``[;] [$]`` - ``{d`` defines dictionary
* ``[;] [$]`` - ``}b`` defines trigger function
* ``[ ] [$]`` - ``A`` equivalent to SPWN ``[]``
* ``[ ] [$]`` - ``B`` equivalent to SPWN ``?b``
* ``[ ] [$]`` - ``C`` equivalent to SPWN ``?c``
* ``[ ] [$]`` - ``D`` equivalent to SPWN ``?i``
* ``[ ] [ ]`` - ``Ev`` roughly equivalent to SPWN ``on(touch(), v)`` - see ``vEv`` entry [below](#postfixes)
* ``[ ] [$]`` - ``F`` equivalent to SPWN ``false``
* ``[ ] [$]`` - ``G`` equivalent to SPWN ``?g``
* ``[ ] [$]`` - ``I`` / ``J`` / ``K`` for loop variables
* ``[;] [ ]`` - ``Lb`` for infinite loop
* ``[;] [$]`` - ``Mb`` for macro def with no args
* ``[ ] [$]`` - ``N`` equivalent to SPWN ``null`` / ``()``
* ``[;] [ ]`` - ``Od`` adds an object
* ``[ ] [$]`` - ``S`` equivalent to SPWN ``""``
* ``[ ] [$]`` - ``T`` equivalent to SPWN ``true``
* ``[;] [ ]`` - ``Wb`` starts a runtime infinite loop
* ``[;] [$]`` - ``Xb`` equivalent to ``)x;b``

## Postfixes
* ``[;] [ ]`` - ``i!v`` assigns a value and returns it
* ``[;] [ ]`` - ``iTd`` declares a type ``@i`` with the members in ``d``
* ``[ ] [ ]`` - ``v?vv`` for ternary operator
* ``[ ] [ ]`` - ``vXv`` equivalent to ``v?vN``
* ``[ ] [$]`` - ``v.i`` for accessing children
* ``[;] [ ]`` - ``v(z`` calls a macro
    * ``z``: ``m`` ( ( ``i:`` )? ``v`` )*
    * ``i:v`` when calling to specify the arg that gets the value
* ``[;] [ ]`` - ``v)z;b`` shortcut for named macro definition
    * see macro definition entry [above](#values-and-prefixes)
* ``[;] [$]`` - ``v]z`` for indexing/slicing
    * *TODO think of syntax for slice*
* ``[;] [$]`` - ``v}v`` dictionarize / (multi-)zip?
    * *TODO for like matrix stuff? idk look at more stuff*
* ``[;] [ ]`` - ``vEv`` roughly equivalent to SPWN ``on(v, v)``
    * *TODO find out how event system works and which conversions should happen*
* ``[;] [ ]`` - ``vIb`` / ``vJb`` / ``vKb`` to start for loop with corresponding variable using helper function ``_scgt_iter(v)``
* ``[;] [ ]`` - ``vLb`` starts a while loop using helper function ``_scgt_bool(v)``
* ``[ ] [ ]`` - ``vM`` calls stuff using ``_scgt_call(v)``
    * macro with no args, trigger functions, treat number or group as trigger function etc
* ``[;] [ ]`` - ``vWb`` starts a runtime while loop
* ``[ ] [$]`` - any type(s) (except consecutive duplicates) - converts to types in order
    * see also [**Built-in types**](#built-in-types)
* operators
    * ``v`` ( op ``v`` )*

## Built-in types
* ``A``: ``@array``
* ``B``: ``@block``
* ``C``: ``@color``
* ``D``: ``@item``
* ``G``: ``@group``
* ``N``: ``@number``
* ``O``: ``@object`` (automatically adds to the level)
* ``S``: ``@string``
* ``T``: ``@bool``
