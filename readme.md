# Harald: Text generation DSL

## Principles

- Declarativity
  - Harald is a declarative language. Programs should be easy to refactor and their behaviour should be easy to read from the structure of the code.
- Focused
  - Harald is designed for only a single purpose: generating random text based on handmade patterns. The goal is not to be a general purpose language; traditional programming constructs are only implemented when they are deemed absolutely necessary or at the very least useful for this task.

## Concepts

### Bag

A bag is an unordered container, which can be randomly sampled with a weighted distribution.
