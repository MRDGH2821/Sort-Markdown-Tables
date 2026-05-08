# Changelog

All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

---

## [0.2.0](/compare/v0.1.5..v0.2.0) - 2026-05-08

### Bug Fixes

- **(smt)** make multi-file -i transactional - ([fd0a76d](/commit/fd0a76dc8829186ca4c89ca33e12efdc72b74385)) - MRDGH2821

### Features

- **(parser)** preserve CRLF line endings in output - ([e343929](/commit/e34392963f0ed5b27d6ca172113e2199e3170ab4)) - MRDGH2821

### Style

- format files - ([ea18cea](/commit/ea18ceada207893302730ef8242d2c3ca4499c36)) - MRDGH2821

### Tests

- **(parser)** add regression for case_insensitive fixture - ([9cfae29](/commit/9cfae2984027c9a01baf8f121940057fd11120fe)) - MRDGH2821
- **(smt)** add perf fixture and record Phase 7 checks - ([001938e](/commit/001938e0bfb43ccc33e8e05ee48bd7186d28b76d)) - MRDGH2821

### Ci

- **(github)** run release job after build matrix - ([20447d9](/commit/20447d95c98950a8d8a250793d0f4e6f4cc5a12a)) - MRDGH2821
- **(github)** build remaining tier1 targets - ([01cc9e6](/commit/01cc9e64a29ebceec446ac3775c79e864b307256)) - MRDGH2821
- **(github)** add tier2 host-tools targets to release matrix - ([81d35a8](/commit/81d35a8b82272c0b899b19c64ecbfc90266f4904)) - MRDGH2821
- **(github)** name release artifacts by target triple - ([14f3ea5](/commit/14f3ea5aff0f954e3b72961fecf02cf153be4a78)) - MRDGH2821
- **(github)** add remaining cross-supported targets - ([0a17cbb](/commit/0a17cbbcf2df20fa39044baf8ea65f371eec38da)) - MRDGH2821

---

## [0.1.5](/compare/v0.1.4..v0.1.5) - 2026-05-08

### Bug Fixes

- **(smt)** align check output and in-place writes with spec - ([472c69c](/commit/472c69c76341f36bdec454c1c5704e77f720669c)) - MRDGH2821

### Documentation

- **(ai)** migrate smt planning docs into openspec - ([10bfd84](/commit/10bfd8450ec86ed7bd3c80b88737cdd649440c46)) - MRDGH2821
- **(smt)** move plan and architecture into openspec specs - ([4e88398](/commit/4e883988b179ec1ff29cfa9b4b95e1bedd6ce344)) - MRDGH2821
- fix documented options - ([d436ec8](/commit/d436ec85f8fccc5d976359eb507324a60698f952)) - MRDGH2821

### Miscellaneous Chores

- **(cocogitto)** prevent bumping via ci - ([326437c](/commit/326437cc8149d5c673ff3a369f9f6c845189eb22)) - MRDGH2821
- **(copier)** update template - ([69dd12d](/commit/69dd12dfba378b0de6038dd2b33ad45744f6f15d)) - MRDGH2821
- **(copier)** update template - ([5d2a716](/commit/5d2a716c3a1325d005dec97f88a8fa2c9421273b)) - MRDGH2821
- **(treefmt)** add ignore list - ([4a17d93](/commit/4a17d93113cd8604f91ea934c6d9ec4ff98ff617)) - MRDGH2821
- initialise with openspec - ([86b9960](/commit/86b99606d7d15a1721aa9da271178f32aa1cfa72)) - MRDGH2821
- add github cli skill - ([40ffed3](/commit/40ffed37c2cb3ae59c4ee4a8998145bad86fa0d6)) - MRDGH2821

### Style

- format files - ([74b1610](/commit/74b16109f773c112a200bcc86eb934c284de7000)) - MRDGH2821

### Ci

- **(github)** fix cross release builds by removing rust-cache - ([573956c](/commit/573956cf45faab287e364501f81fb0dbf7f46f54)) - MRDGH2821
- move tests into ci.yml - ([77da62b](/commit/77da62b244c805dae8290c3991adffe8c7895734)) - MRDGH2821

---

## [0.1.4](/compare/v0.1.3..v0.1.4) - 2026-03-14

### Miscellaneous Chores

- **(cocogitto)** add cargo check in bump hook - ([561e89a](/commit/561e89ac89d4c48a69a8156e7a6565de407a5808)) - MRDGH2821

### Ci

- fix linter errors - ([a763cf9](/commit/a763cf9578f877b5ff00b8cb593e300c8537dbea)) - MRDGH2821

---

## [0.1.3](/compare/v0.1.2..v0.1.3) - 2026-03-14

### Miscellaneous Chores

- **(cocogitto)** bump when ci commit is made - ([7ca32c9](/commit/7ca32c983270fc491c724abd12fab8b063d0cdd2)) - MRDGH2821

### Style

- format file - ([48f1507](/commit/48f15075bc3152a718d3e692a3855658d8c2fc5e)) - MRDGH2821

### Ci

- simplify release workflow with graceful failure handling - ([f9dcbbe](/commit/f9dcbbe6b3765ff87443c630d69b1fe069e04c41)) - MRDGH2821

---

## [0.1.2](/compare/v0.1.1..v0.1.2) - 2026-03-14

### Bug Fixes

- remove invalid target - ([b013ae5](/commit/b013ae525baa3e27dacf1bddf067f8b2964b9135)) - MRDGH2821

### Ci

- add musl targets - ([c36b7ec](/commit/c36b7ec6716180ea40dd4caded45b0a011b72450)) - MRDGH2821

---

## [0.1.1](/compare/v0.1.0..v0.1.1) - 2026-03-14

### Bug Fixes

- **(parser)** allow blank lines between smt comment and table - ([2f3ba94](/commit/2f3ba94e656edf823b6988d0910318e9759c00d6)) - MRDGH2821

### Documentation

- update readme - ([2ffc5a2](/commit/2ffc5a2b8cf3abf15d04c9c8f7bfd45bb26e8f82)) - MRDGH2821
- change text - ([da9dfe8](/commit/da9dfe8ca9a2cbe46e84b16a148872ebba20bcb0)) - MRDGH2821

### Refactoring

- rename licence file - ([b462da8](/commit/b462da883e6d32be02529e3d9cacf344a022db70)) - MRDGH2821

---

## [0.1.0] - 2026-03-14

### Documentation

- **(ai)** add docs for spec driven development - ([62c3054](/commit/62c30544d86e959bd65f38f8c602019c2c0eddc0)) - MRDGH2821
- **(ai)** add git commit scope workflow to AGENTS.md - ([5caf612](/commit/5caf6126ccab9e0914e86932a0966e21ec562a85)) - MRDGH2821
- **(smt)** add comprehensive README with usage, features, and examples - ([b676201](/commit/b6762018cd7d531f54ec5f1bd628634ef84c3f04)) - MRDGH2821
- add documents - ([a2d84c6](/commit/a2d84c6d77c03fa5f847332e6a3a0a8dd796f574)) - MRDGH2821

### Features

- **(smt)** initialize rust project with error, cli, and parser modules - ([2bad4d6](/commit/2bad4d6437b9a31d3ba629d9d4e62cd5e566f69b)) - MRDGH2821
- **(sorter)** implement table sorting with numeric/lexicographic comparators and stable sort guarantee - ([fa1a686](/commit/fa1a686a877ad31b2785091181bc5427dfd51001)) - MRDGH2821

### Miscellaneous Chores

- **(cocogitto)** update scopes - ([f475ae5](/commit/f475ae5d41bcaaa3e6c5b7f66ef2d99c3111901e)) - MRDGH2821
- **(cocogitto)** add new scope - ([e254b60](/commit/e254b604c17a32dfa9afd7a9ddae1d50af1f2676)) - MRDGH2821
- **(cocogitto)** add sorter and parser scopes - ([ad93f14](/commit/ad93f1465967661fbb9281da277ec565c9f25e31)) - MRDGH2821
- **(cocogitto)** add writer scope - ([a8bfd4b](/commit/a8bfd4b297e4aa32f94647fb7b04958f5910ea82)) - MRDGH2821
- **(cocogitto)** add cargo bump command - ([33339da](/commit/33339da5234e340bd7220910f2dc7d6f6adc9091)) - MRDGH2821
- **(smt)** add GPL-3.0-or-later license - ([fedbdbe](/commit/fedbdbee792c6e2c2c01be4f603f368e4113ead9)) - MRDGH2821
- **(smt)** update repository URL to GitHub - ([ff671d8](/commit/ff671d80d00da71b14c0edd09d835a8cfb96300c)) - MRDGH2821
- **(treefmt)** ignore test files - ([8e02eca](/commit/8e02ecaac441b38986b596696529b6fe93e5f003)) - MRDGH2821
- initial commit - ([5c88d6e](/commit/5c88d6ea73259404d85c53ac6a5ac2c8a6b7c9d8)) - MRDGH2821

### Refactoring

- **(smt)** fix all clippy warnings and improve code quality - ([192af14](/commit/192af14608702620d18a636c61768d38f12b00c5)) - MRDGH2821

### Style

- format files - ([247253b](/commit/247253b3c7970f8e59149e290a1d6d252c9b008a)) - MRDGH2821
- format files - ([235d2c2](/commit/235d2c2f919dc3acb0ea12bc8e59472dd7eebb85)) - MRDGH2821
- format file - ([6423b13](/commit/6423b13927bac4968d16ab456863b0d4807f4eb2)) - MRDGH2821

### Tests

- **(smt)** implement integration tests and fixtures for phase 6 - ([a446025](/commit/a4460257002479c19b11f8d933cffd14ceb4c4b8)) - MRDGH2821

### Build

- setup comprehensive CI/CD workflows and remove manual changelog - ([1e660a0](/commit/1e660a0635cd9cff4c7d810433f4d1c6d190d6e9)) - MRDGH2821
- restrict release workflow to semantic version tags only - ([cfdd334](/commit/cfdd33417905f1356d7bf3639c44bf4f1c3b17de)) - MRDGH2821

<!-- generated by git-cliff -->
