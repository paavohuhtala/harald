# Harald: Text generation DSL

## Principles

- Declarativity
  - Harald is a declarative language. Programs should be easy to refactor and they should
- Focused
  - Harald is designed for only a single purpose: generating random text based on handmade patterns. The goal is not to be a general purpose language; traditional programming constructs are only implemented when they are deemed absolutely necessary or at the very least useful for this task.

## Concepts

### Bag

A bag is an unordered container, which can be randomly sampled with a weighted distribution.

### Pattern

A pattern

```
iceCream = { maybe(0.25 bag["chocolate" | "mint chip"]) " ice cream" }
```

## Example: fantasy RPG weapon generator

```
adjective = bag["Awesome", "Epic", "Legendary", "Worn"];
sword = prefix(bag("broad", "bastard", "long", "short", 2 ""), "sword");
weaponType = bag(0.3 ...sword, "spear", "hammer", "bow", "staff", "dagger");
ofSuffix = bag(2 "", { "of " bag["awesomeness", "epicness", "evilness"] });
result = { adjective " " weaponType)};
```

## Built-ins

### `bag[]`

Creates a new bag from a list of weight expressions. Each expression has a default weight of 1.0, but weights can also be provided for any

Evaluating a `bag` samples it according to the weights.
