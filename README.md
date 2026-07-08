# Identity Calculus

This repository contains my notes and experimental code for studying the reduction graphs of identity lambda-terms, for the internship at IRIF I did with Giulio Manzonetto.
The main idea is to translate identity terms into ordered rooted trees, where beta-reduction corresponds to removing leaves. This gives a simple combinatorial model for reduction DAGs, multiplicities of reductions, and equivalences between terms with the same reduction graph.
See the report in typst/

## Contents

* `typst/`: Typst notes and report sources.
* `search/`: Rust code for generating and comparing reduction DAGs of ordered rooted trees.

## Rust tool

The experimental tool can build reduction DAGs, count reductions, compare DAGs, and search for examples or counterexamples.

From the `search/` directory:

```bash
cargo run -- dag --tree "(()())" --dot out.dot --multi
cargo run -- enumerate --tree "(()())" --multi
cargo run -- equality --tree1 "(()())" --tree2 "((()))" --multi
```

Trees are written using a parenthesized syntax, for example `()` for a leaf and `(()())` for a root with two leaf children.

## Citation

If you use this project, please cite the repository; see the bibliography entry below.

```
@software{cypooos_identity_calculus,
  author       = {{Cypooos}},
  title        = {Identity Calculus},
  year         = {2026},
  url          = {https://github.com/Cypooos/identity-calculus},
  note         = {GitHub repository}
}
```
