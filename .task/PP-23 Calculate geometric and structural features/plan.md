# PP-23 — Calculate geometric and structural features

## 1. Objective

Compute reproducible geometric descriptors from residue and atom coordinates.

## 2. Scope and dependencies

PP-22 coordinate arrays; PP-21 structure parser metadata; PP-20 protein identity.

Related plans: [PP-21](../PP-21%20Implement%20PDB%2FmmCIF%20structure%20parser/plan.md) · [PP-22](../PP-22%20Extract%20protein%20residues%20and%20atomic%20coordinates/plan.md) · [PP-24](../PP-24%20Extract%20secondary%20structure%20and%20confidence%20metrics/plan.md) · [PP-25](../PP-25%20Generate%20protein%20structure%20feature%20vectors/plan.md) · [PP-30](../PP-30%20Design%20protein%20embedding%20generation%20pipeline/plan.md) · [PP-33](../PP-33%20Integrate%20structure%2C%20sequence%20and%20PPI%20features/plan.md). Coordinate residue indices, units, missingness, and feature versioning with these linked tasks.

## 3. Step-by-step implementation and to-do

### Step 1 — Define feature catalogue

Select physically interpretable descriptors and document formulas, units, aggregation, and missing-value policy.

**To-do**
- [ ] Define pairwise distance/contact cutoff and atom selection
- [ ] Define radius of gyration and residue-level geometry metrics
- [ ] Define contact density and local neighborhood descriptors
- [ ] Document computational complexity and memory strategy

### Step 2 — Implement geometry calculations

Compute distances and aggregate features using vectorized operations or spatial indexes for scale.

**To-do**
- [ ] Implement residue/atom distance calculations
- [ ] Implement contact map or sparse contact graph
- [ ] Compute radius of gyration and compactness descriptors
- [ ] Avoid allocating full quadratic matrices for large structures unless bounded

### Step 3 — Add numerical and biological validation

Verify units, symmetry, invariance, and behavior for incomplete structures.

**To-do**
- [ ] Test distance symmetry and zero diagonal
- [ ] Test translation/rotation invariance where applicable
- [ ] Validate Å units and finite numeric outputs
- [ ] Test small proteins, missing residues, and chain-separated structures

### Step 4 — Publish structural feature records

Write feature records with schema/version and links to source coordinate artifacts.

**To-do**
- [ ] Define feature schema and feature version
- [ ] Include structure/model provenance and coverage metadata
- [ ] Add sample output and reproducibility test
- [ ] Hand off feature contract to PP-25 and PP-30

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

- **Upstream:** PP-22 coordinate arrays; PP-21 structure parser metadata; PP-20 protein identity.
- **Consumers / coordination:** PP-21, PP-22, PP-24, PP-25, PP-30, PP-33
- **Contract rule:** publish coordinate units, residue indexing, feature definitions, schema version, provenance, and known limitations; notify all linked consumers of changes.

## 8. Completion notes

Record parser/tool versions, parameters, artifact URIs, sample protein accessions, test results, and deviations.
