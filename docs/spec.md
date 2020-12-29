# Harald - WIP specification / introduction

## What is Harald?

Harald is a domain-specific language for randomly assembling text from hand made patterns.

The language is:

- Expression based
- Mostly lazily evaluated
  - Generally speaking expressions are only evaluated when necessary.
  - Some (future) language features can force an expression to be evaluated earlier.
- Mostly pure
  - No IO or explicit mutation.
  - Language constructs like bags and tables use randomness so their output is unpredictable by design.
- Mostly dynamically typed
  - There is no compile-time type checking as of now, but the interpreter performs run-time type checks as necessary.
  - Some language features (such as bag and table syntaxes) perform parse and/or compile time checks which are essentially type checks in all but name.

## Concepts

### String literal

A string literal is a sequence of UTF-8 characters delimited by double quotes (`"hello!"`). Some characters can only be written using standard C-like escape sequences (`\n`, `\"`, `\\`).

### Variable

A variable declaration starts with an identifier, followed by an equals sign, an expression and a semicolon.
At the time of writing a variable declaration is the only kind of statement in the language.

An identifier must start with a letter, followed by any sequence of letters and numbers. A letter is defined to be any character in the Unicode Alphabetic group.

```
thisIsAVariable = "123";
thïsÏsÄlsoVälid = thisIsAVariable;
```

A variable declaration does not evaluate the expression; the expression is evaluated every time the variable is referenced.

Variable references are resolved by name every time they are evaluated. This means variable declaration order does not matter, except in cases when the same variable is declared multiple times (the last definition wins). This also means that variable definitions can be self-recursive. The current Harald interpreter does not yet solve the halting problem, so be careful when using recursion.

### Program

A Harald program is a sequence of statements separated by semicolons. One of the statements must define the magic variable `result`, which is evaluated to get the output of the program.

```
result = "This is the output.";
```

### Pattern

A pattern is used to concatenate a sequence of expressions into a single string.

```
world = "world!";
result = { "Hello " world };
```

Evaluation:

1. Evaluate all expressions from left to right and coerce them into strings
   - Throw a type error if any value cannot be coerced into a string
2. Concatenate the strings into a single string
   - If there are no strings no concatenate, return an empty string (`""`)

### Bag

A bag is a unordered container of 1 to N expressions which can be sampled randomly. A bag must always contain at least one item.

```
oneEntry = bag [ "a" ];
twoEntries = bag ["a", "b" ];
threeEntries = bag [
    "a",
    "b",
    "c",
];
```

Bag items are separated by commas. A trailing comma after the last item is allowed but not required.

Each bag item can optionally have a _weight_ which affects how likely the item is to be selected from the bag. The weight is a positive number, either an integer or a decimal number. The default weight is 1.0. At least one item must have a non-zero weight.

```
example = bag [ 0 "never", "always" ];
example = bag [ 9.0 "9 out of 10", "1 out of 10" ];
```

Coercion to string:

1. Select a random item the bag according to the weights
2. Evaluate the item
3. Coerce the item to string or throw an error

NOTE: While a bag can contain expressions that resolve to any type, a bag can't currently be sampled without coercing the value of the chosen expression to string.

### Table

A table is 2-dimensional container consisting of rows, each with a fixed number of named columns. It is very useful for implementing dictionaries for words which have different forms and inflections. Just like `bag` it is designed to be randomly sampled.

Rows are delimited by square brackets and like bag entries are separated by commas. Values within rows are also separated by commas. The table syntax allows optional trailing commas just like the `bag` syntax.

The first row defines the names of the columns. The names are prefixed with `.` and follow the same rules for identifiers as declared variables.

```
example = table [
    [.firstColumn, .secondColumn],
    ["a",          "b"          ],
    ["c",          "d"          ],
];
```

#### Column extraction

A column can be extracted into a `bag` using the property access syntax. Using `example` from the previous code block, the following two statements produce the same results

```
result = example.firstColumn;
result = bag [ "a", "c" ];
```

#### Weights

Like bags, tables supports an optional weight for each row (default 1.0). When a bag is extracted from a table column, each value inherits its weight from its row.

```
example2 = table [
       [.firstColumn, .secondColumn],
    10 ["a",          "b"          ],
    3  ["c",          "d"          ],
];

firstColumn = example2.firstColumn;
firstColumn = bag [ 10 "a", 3 "c" ];

secondColumn = example2.secondColumn;
secondColumn = bag [ 10 "b", 3 "d" ];
```

#### Holes

Tables also supports _holes_, which enables leaving out some values for some columns of a row. Holes are marked with an underscore (`_`). This is useful when some words in a table don't have suitable forms for all inflictions, but the words are still similar enough that they can be used together in the same dictionary.

When a bag is extracted from a table column, the holes are ignored.

```
example3 = table [
    [.firstColumn, .secondColumn],
    [_,            "b"          ],
    ["c",          _            ],
];

firstColumn = example3.firstColumn;
firstColumn = bag [ "c" ];

secondColumn = example3.secondColumn;
secondColumn = bag [ "b" ];
```

#### Append

In addition to normal values and holes, table entries can also be written as suffixes to the base form (the first column). These are known as _append entries_. This is merely syntax sugar for saving keystrokes when writing dictionaries by hand, but it is very useful when different forms are almost but not quite identical.

Append entries are typically used with string literals, but they technically support any expression. Syntax-wise they differ from normal entries by having a `+` prefix.

```
animals = table [
    [.singular, .plural  ],
    ["dog",     +"s"     ],
    ["puppy",   "puppies"],
    ["cat",     +"s"     ],
    ["kitten",  +"s"     ],
];

withoutAppend = table [
    [.singular, .plural  ],
    ["dog",     "dogs"   ],
    ["puppy",   "puppies"],
    ["cat",     "cats"   ],
    ["kitten",  "kittens"],
];
```

One common use case is to repeat the first form by appending an empty string (`+""`).

Note that even with more than 2 columns, append entries always append to the first column, _not_ the previous column.

If the first column is a hole, append entries treat it like it was an empty string (`""`).

#### Remarks

Each row must have exactly as many entries as there are named columns, however any number of these can be holes. Having a row with a differing number of entries is a compilation error.

It is technically legal to have a row consisting only of holes, but it is also practically useless.

Each column in a table must have at least one (non-hole) value, because otherwise property access could result in bags with zero values. Having columns with no values is a compilation error.

### Built-in functions

Harald contains a few built-in functions which are implemented by the interpreter. At the time of writing the language has no facilities for creating user defined functions.

#### `capitalise(expr)`

Coerces the expression to string and converts the first character to upper case.

#### `maybePrepend(prefix, condition)`

Prepends `prefix` to `condition` if `condition` when evaluated and coerced to string is not empty. Useful for conditionally adding whitespace.

#### `maybeAppend(condition, suffix)`

Appends `suffix` to `condition` if `condition` when evaluated and coerced to string is not empty.
