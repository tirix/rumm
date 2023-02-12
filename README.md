
# Rumm - A Metamath tactics based proof language

This repository includes:
- a [specification](rumm.md) for a tactics-based proof language for [Metamath](http://us.metamath.org/),
- a [reference implementation](rumm) in Rust for building Metamath proofs with that language,
- a [set of tactics](set.rmm) for building proofs for the ZFC [set.mm](http://us.metamath.org/mpeuni/mmset.html) Metamath database
- a few [example proofs](rumm/examples)

Rumm intends to be simple, generic and yet powerful. Simple in the sense that it only specifies a very limited set of built-in tactics. Generic in the sense that the language itself is not taylored to any specific Metamath database, but can be reused for all of them.

At the origin it intends to answer the feasibility question "*what would a tactics-based language for Metamath look like?*".
