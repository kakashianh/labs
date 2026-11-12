# PP-22 — Extract protein residues and atomic coordinates

## 1. Objective

Convert parsed structures into residue-level and atom-level coordinate arrays with consistent indexing.

## 2. Scope and dependencies

PP-21 parser output; PP-20 protein master; PP-10 structural feature contracts.

Related plans: [PP-20](../PP-20%20Generate%20protein%20master%20and%20PPI%20Parquet%20datasets/plan.md) · [PP-21](../PP-21%20Implement%20PDB%2FmmCIF%20structure%20parser/plan.md) · [PP-23](../PP-23%20Calculate%20geometric%20and%20structural%20features/plan.md) · [PP-24](../PP-24%20Extract%20secondary%20structure%20and%20confidence%20metrics/plan.md) · [PP-25](../PP-25%20Generate%20protein%20structure%20feature%20vectors/plan.md) · [PP-30](../PP-30%20Design%20protein%20embedding%20generation%20pipeline/plan.md) · [PP-33](../PP-33%20Integrate%20structure%2C%20sequence%20and%20PPI%20features/plan.md). Coordinate residue indices, units, missingness, and feature versioning with these linked tasks.

## 3. Step-by-step implementation and to-do

### Step 1 — Define residue/atom representation

Specify residue index, amino-acid code, chain, atom name, element, coordinates, and missingness representation.

**To-do**
- [ ] Define canonical residue numbering independent of raw author numbering
- [ ] Define atom coordinate dtype and units (Å)
- [ ] Define backbone atom and all-heavy-atom policies
- [ ] Preserve mapping to source chain/residue identifiers

### Step 2 — Implement coordinate extraction

Generate ordered arrays/tables from parser output without losing residue-to-atom relationships.

**To-do**
- [ ] Extract x/y/z coordinates for selected atoms
- [ ] Extract residue sequence and atom-to-residue index mapping
- [ ] Handle alternate locations and missing atoms consistently
- [ ] Preserve chain boundaries and residue masks

### Step 3 — Validate coordinate integrity

Check coordinate ranges, residue counts, duplicate atoms, and sequence alignment metadata.

**To-do**
- [ ] Reject NaN/inf coordinates and malformed records
- [ ] Verify atom records map to exactly one residue
- [ ] Compare extracted sequence/length with PP-21 validation results
- [ ] Add tests for gaps, insertion codes, and multi-chain structures

### Step 4 — Publish coordinate artifacts

Write compact, versioned residue/coordinate artifacts for feature engineering and embeddings.

**To-do**
- [ ] Define serialization format and metadata sidecar
- [ ] Store artifact URI, checksum, and parser version
- [ ] Add read-back test and sample visual inspection
- [ ] Document coordinate conventions for PP-23/PP-25/PP-30

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

- **Upstream:** PP-21 parser output; PP-20 protein master; PP-10 structural feature contracts.
- **Consumers / coordination:** PP-20, PP-21, PP-23, PP-24, PP-25, PP-30, PP-33
- **Contract rule:** publish coordinate units, residue indexing, feature definitions, schema version, provenance, and known limitations; notify all linked consumers of changes.

## 8. Completion notes

Record parser/tool versions, parameters, artifact URIs, sample protein accessions, test results, and deviations.
