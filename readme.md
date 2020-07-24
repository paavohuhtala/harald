# Harald: Text generation DSL

## Concept

```
adjective = bag("Awesome","Epic", "Legendary", "Worn");
sword = prefix(bag("broad", "bastard", "long", "short", 2 ""), "sword");
weaponType = bag(0.3 ...sword, "spear", "hammer", "bow", "staff", "dagger");
ofSuffix = bag(2 "", pat("of " bag("awesomeness", "epicness", "evilness")));
result = pat(adjective " " weaponType);
```
