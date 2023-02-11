# The Rumm Language

This description of the Rumm language provides syntax where expressions in brackets like `<formula>` are syntactic constructs. It also provides examples, which assume the `set.mm` database is used.

The Rumm language deals with three kinds of expressions:
- [formulas](#formulas),
- [theorems](#theorems),
- [tactics](#tactics).

At top level, Rumm files list tactic scripts and proofs.
Tactic scripts are sub-routine-like proof elements which can be called with variable parameters. The `tactics` keyword allows to declare such a tactics script:
```
tactics <tactics name> (<parameter ID> ... <parameter ID>) <tactics>
```
Tactics scripts allow modularity. Common proof schemes can be described in script tactics, and reused in several proofs, or as sub-tactics.

Proofs simply tell the Rumm program to apply the given tactics to generate a proof for a given theorem statement. They are the starting point for proof elaboration.
```
proof <statement> <tactics>
```
In addition, the `load` keyword tells the program to load the specified MM database file.
```
load <filename>
```

## Tactics

When evaluated, tactics generate proofs. Tactics are evaluated within a *context* with the following properties:
- a **goal**: the goal is the formula to be currently proven. In Rumm, there is always a single goal at any point of the program.
- **hypotheses**: this is a set of formulas known to be proven. When a proof is initiated for a given theorem, this includes all theorem's (`\$e`, essential) hypotheses. Along the proof process, intermediate subgoals are added to this list.
- **local variables**: variables are strongly typed and can be either formula variables (with identifiers prefixed with `+`), theorem variables (prefixed with `≈`), or tactics variables (prefixed with `@`).

Tactics may succeed and return a proof, or fail. Tactics themselves are generally enclosed within brackets `{ ... }`. 

### **The `!` built-in tactics**

This is the simplest possible tactics: it attempts to match the goal with one of the hypotheses or already proven subgoals. Its syntax is a single exclamation mark sign, `!`.

---

### **The `?` built-in tactics**

This is only a dummy proof placeholder. It always fails, but allows to build a syntactically correct Rumm file where a tactics is needed. Its syntax is a single question mark sign, `?`.

---

### **The `apply` built-in tactics**

This is the atomic building block for proofs, applying a single theorem.
```
{ apply <statement> <tactics> ... <tactics> with <statement> <formula> ... <statement> <formula> }
```
The theorem specified by `<statement>` is applied. A list of tactics is provided following the theorem's statement name, which will be used to recursively prove each of the theorem's essential hypotheses. The single goal for these tactics will be the hypothesis, with the theorems variables substitued in the current context. In some cases, there are variables which don't appear in the theorem's statement. 

This example applies the `~a1i` theorem, and attempts to match the hypothesis with already proven statements.
```
{ apply ~a1i ! }
```

This example applies syllogism `~syl` to the current goal:
```
syl.1 $e |- ( ph -> ps ) $.
syl.2 $e |- ( ps -> ch ) $.
$( An inference version of the transitive laws for implication $)
syl $p |- ( ph -> ch ) $= ( wi a1i mpd ) ABCDBCFAEGH $.
```
Since `~syl`'s final statement, `( ph -> ch )`, does not include the `ps` wff variable, one has to provide it manually using the `with` keyword.
```
{ apply ~syl ! ! with ~ps $ A e. V $ }
```

---

### **The `subgoal` built-in tactics**

Rumm generally works top-down, i.e. one starts with a proof for the last step of the final results, and then works one's way to more elementary sub-proofs. This tactics allows to write proofs "bottom up", i.e. first provide a proof for an arbitrary statement, and then the proof for the current goal, whereas the subgoal is already proven.
```
{ subgoal <tactics> <formula> <tactics> }
```
The formula provided is the statement of the subgoal.
The first tactics, listed before that formula, is applied to find a proof for that subgoal.
The second tactics, listed after the formula, is applied to find a proof for the final goal, whereas the formula is a known true statement, and can be found e.g. using the `!` tactics.

---

### **The `try` built-in tactics**

This tactics allows to try a sequence of different tactics.
```
{ try <tactics> ... <tactics> }
```
Each tactics is applied to prove the current goal, and the `try` tactics return the proof provided by first successful sub-tactics. This tactics fails if *all* sub-tactics fail. In the current implementation tactics are tried in the order they are provided.

---

### **The `match` built-in tactics**

This tactics attempts to match a given fixed formula with a list of formulas, and applies different tactics based on the match.
```
{ match <formula> <formula> <tactics> ... <formula> <tactics> }
```
The formula to match is provided first. It is often the current goal.
Then, a list of couples for formula pattern and tactics are provided. 

Example:
```
{ match goal
    $ ( &W1 -> &W2 ) $ @T1
    $ ( &W1 /\ &W2 ) $ @T2
}
```
If the current goal is an implication `->`, this tactics applies `@T1`.
If it is a conjunction `/\`, it applies `@T2`.

---

### **The `find` built-in tactics**

This tactics finds in the loaded database a theorem matching the given formula template, and applies the given tactics.
```
{ find <tactics> <formula> <tactics> }
```
If a match with the given formula is found, the first tactics provided is used to find a proof for the theorem's hypotheses. Note that only one tactics is provided for all hypotheses.
Then the second tactics is applied to prove the given formula itself.

---

### **The `use` built-in tactics**

This tactics allows to use a generic tactics script.
```
{ use <tactics name> <parameter> ... <parameter> }
```
Each parameter is evaluated, and the tactics script is executed with additional context variables corresponding to each of the parameters.
The `<parameter>` tag represents either a formula `<formula>`, a theorem `<statement>` or a tactics `<tactics>` expression.

In the example below, a tactics script named "example" is defined, taking one tactics parameter named `@T`.
It is then used to prove a theorem `~ex1`, whereas the tactics `!` is going to be applied for parameter `@T`.
```
tactics example(@T) {
    apply ~a1i @T
}

proof ~ex1 {
    use example !
}
```

---

## Theorems

"Constant" theorems are refered to by their name in the loaded database, prefixed with a tilde sign `~`. For example,
```
~syl
```
Refers to the `syl` theorem of the loaded database.

Variable theorems are referred to by identifiers with the "approximately equal" sign `≈` prefix (two tildes).


## Formulas

A constant formula is written within dollar signs `$`. For example,
```
$ ( 1 + 1 ) = 2 $
```
Such constant formulas must be syntactically correct for the loaded database.

Variable formulas are referred to by identifiers with the plus sign `+` prefix.

---

### **The `goal` generic formula**

This resolves to the current goal.
```
goal
```

---

### **The `statement` generic formula**

This resolves to the provided theorem's final statement.
```
statement <statement>
```

For example, this would resolve to `( ph -> ch )`:
```
statement ~syl
```

---