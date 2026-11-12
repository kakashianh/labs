# PP-21 — Implement PDB/mmCIF structure parser

## 1. Objective

Parse AlphaFold mmCIF/PDB files into validated structural records that downstream feature extraction can consume.

## 2. Scope and dependencies

PP-13 raw structure files and manifests; PP-20 canonical protein master; PP-10 structure metadata contract.

Related plans: [PP-13](../PP-13%20Implement%20AlphaFold%20structure%20data%20crawler/plan.md) · [PP-20](../PP-20%20Generate%20protein%20master%20and%20PPI%20Parquet%20datasets/plan.md) · [PP-22](../PP-22%20Extract%20protein%20residues%20and%20atomic%20coordinates/plan.md) · [PP-23](../PP-23%20Calculate%20geometric%20and%20structural%20features/plan.md) · [PP-24](../PP-24%20Extract%20secondary%20structure%20and%20confidence%20metrics/plan.md) · [PP-25](../PP-25%20Generate%20protein%20structure%20feature%20vectors/plan.md). Coordinate residue indices, units, missingness, and feature versioning with these linked tasks.

## 3. Step-by-step implementation and to-do

### Step 1 — Select parser and define supported structure inputs

Choose Biopython/Biotite parser strategy and explicitly support mmCIF first, with PDB only if required.

**To-do**
- [ ] Pin parser library and supported format versions
- [ ] Define chain selection and model selection policy
- [ ] Define handling for missing atoms, alternate locations, and modified residues
- [ ] Map structure accession to canonical protein accession

### Step 2 — Implement file discovery and parsing

Read only validated structure manifests and convert files into a normalized internal representation.

**To-do**
- [ ] Resolve file URI and checksum from manifest
- [ ] Parse model, chain, residue, atom, and coordinate data
- [ ] Preserve chain IDs, residue numbering, insertion codes, and residue names
- [ ] Capture parse warnings and failures per structure

### Step 3 — Validate sequence and structural identity

Check parsed chain sequence against expected protein sequence and record coverage/mismatch.

**To-do**
- [ ] Extract amino-acid sequence per chain
- [ ] Compare against PP-20/UniProt sequence using documented alignment policy
- [ ] Record coverage, identity, missing residues, and mismatch reason
- [ ] Avoid silently assigning a multi-chain structure to the wrong protein

### Step 4 — Publish parser output

Expose normalized structure records and fixtures for feature extraction tasks.

**To-do**
- [ ] Define parser output schema and version
- [ ] Add fixtures for valid, incomplete, multi-chain, and corrupt files
- [ ] Test deterministic parsing and graceful failure
- [ ] Document supported formats and known limitations

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

- **Upstream:** PP-13 raw structure files and manifests; PP-20 canonical protein master; PP-10 structure metadata contract.
- **Consumers / coordination:** PP-13, PP-20, PP-22, PP-23, PP-24, PP-25
- **Contract rule:** publish coordinate units, residue indexing, feature definitions, schema version, provenance, and known limitations; notify all linked consumers of changes.

## 8. Completion notes

Record parser/tool versions, parameters, artifact URIs, sample protein accessions, test results, and deviations.
