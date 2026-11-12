# PP-24 — Extract secondary structure and confidence metrics

## 1. Objective

Derive secondary-structure and AlphaFold confidence descriptors while retaining residue-level provenance.

## 2. Scope and dependencies

PP-21 parsed structures; PP-22 residue indexing; PP-13 AlphaFold metadata.

Related plans: [PP-13](../PP-13%20Implement%20AlphaFold%20structure%20data%20crawler/plan.md) · [PP-21](../PP-21%20Implement%20PDB%2FmmCIF%20structure%20parser/plan.md) · [PP-22](../PP-22%20Extract%20protein%20residues%20and%20atomic%20coordinates/plan.md) · [PP-23](../PP-23%20Calculate%20geometric%20and%20structural%20features/plan.md) · [PP-25](../PP-25%20Generate%20protein%20structure%20feature%20vectors/plan.md) · [PP-30](../PP-30%20Design%20protein%20embedding%20generation%20pipeline/plan.md). Coordinate residue indices, units, missingness, and feature versioning with these linked tasks.

## 3. Step-by-step implementation and to-do

### Step 1 — Define secondary structure and confidence semantics

Choose DSSP-compatible assignment and map AlphaFold confidence (pLDDT) from the correct source field.

**To-do**
- [ ] Define helix/sheet/coil categories and assignment tool/version
- [ ] Identify confidence field and verify its units/range
- [ ] Define residue-level and protein-level aggregation
- [ ] Define missing/unsupported structure behavior

### Step 2 — Implement per-residue extraction

Calculate or parse secondary structure and confidence aligned to canonical residue indices.

**To-do**
- [ ] Run selected secondary-structure assignment on compatible structures
- [ ] Extract pLDDT/confidence per residue
- [ ] Align outputs to PP-22 residue index mapping
- [ ] Preserve chain and residue coverage information

### Step 3 — Aggregate protein-level descriptors

Summarize confidence and structural composition without discarding per-residue data.

**To-do**
- [ ] Calculate mean/median and configurable confidence fractions
- [ ] Calculate helix/sheet/coil fractions
- [ ] Report coverage and missing-residue proportions
- [ ] Distinguish predicted confidence from experimental resolution

### Step 4 — Validate and publish

Test against known structures and publish a versioned feature dataset.

**To-do**
- [ ] Validate confidence range and residue alignment
- [ ] Test structures with missing residues and unsupported residues
- [ ] Record tool versions and parameters
- [ ] Hand off schema to PP-25 and PP-30

## 4. Deliverables

- Parser/extractor/feature implementation and versioned output schema.
- Representative structural fixtures and reproducible sample artifacts.
- Documentation of units, indexing, assumptions, tool versions, and limitations.

## 5. Acceptance criteria

- [ ] All step-level to-dos are complete.
- [ ] Residue and atom mappings remain traceable to source structures.
- [ ] Units, numbering, missing-value policy, and feature definitions are documented.
- [ ] Numerical and structural validation tests pass.
- [ ] Downstream consumers can load the output without custom undocumented transformations.

## 6. Testing strategy

- **Unit:** parser edge cases, coordinate transforms, formulas, alignment, and missingness.
- **Integration:** raw structure manifest to normalized feature artifact.
- **Regression:** compare fixed structure fixtures and schema/version metadata.
- **Scientific sanity:** validate ranges, invariances, coverage, and representative structures.

## 7. Cross-task handoff

- **Upstream:** PP-21 parsed structures; PP-22 residue indexing; PP-13 AlphaFold metadata.
- **Consumers / coordination:** PP-13, PP-21, PP-22, PP-23, PP-25, PP-30
- **Contract rule:** publish coordinate units, residue indexing, feature definitions, schema version, provenance, and known limitations; notify all linked consumers of changes.

## 8. Completion notes

Record parser/tool versions, parameters, artifact URIs, sample protein accessions, test results, and deviations.
