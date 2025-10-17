# Evolve

A program that exposes a set of simple functions to evolutionary pressure to see what comes out.  The goal is to allow for some level of open ended evolution in an as yet undetermined environment.  At a high level there should be a set of primitives that an algorithm can select from (eg. z=add(x,y), push(x,fifo), z=pop(fifo), ...).  The alogorithm then should be able to create groups of these to do more advanced things then save these "templates" and be able to choose from those templates like they are a primitive themselves.  This lets it build something up.  Finally the "genetic" code should be able to express a selection of these primitives and templates and be used in an evolutionary process.  The algorithm should then be placed in the environment to operate on some date and try and output some data to a response file then complete or something like that.

## Technology
- Rust
- Storage (SQL or JSON file to store history of things)
- CLI to explore what the evolutionary algorithm has produced and to run the simulations

## Safety
- Each algorithm should live in a sandbox where it can edit it's environment (and potentially itself).
- There should be limits to what it can produce (max file size or something).

