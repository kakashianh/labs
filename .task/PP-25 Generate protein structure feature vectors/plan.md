# PP-25 — Generate protein structure feature vectors

## 1. Objective

Assemble validated structural descriptors into fixed, versioned protein-level feature vectors for downstream fusion and ML.

## 2. Scope and dependencies

PP-23 geometry features; PP-24 secondary structure/confidence; PP-22 residue coordinates; PP-20 protein master.

Related plans: [PP-20](../PP-20%20Generate%20protein%20master%20and%20PPI%20Parquet%20datasets/plan.md) · [PP-22](../PP-22%20Extract%20protein%20residues%20and%20atomic%20coordinates/plan.md) · [PP-23](../PP-23%20Calculate%20geometric%20and%20structural%20features/plan.md) · [PP-24](../PP-24%20Extract%20secondary%20structure%20and%20confidence%20metrics/plan.md) · [PP-30](../PP-30%20Design%20protein%20embedding%20generation%20pipeline/plan.md) · [PP-33](../PP-33%20Integrate%20structure%2C%20sequence%20and%20PPI%20features/plan.md) · [PP-34](../PP-34%20Prepare%20multi-label%20protein%20function%20prediction%20dataset/plan.md) · [PP-35](../PP-35%20undefined/plan.md). Keep graph identifiers, schema version, query semantics, and vector feature versions consistent across these tasks.

## 3. Step-by-step implementation and to-do

### Step 1 — Define vector schema and feature groups

Specify ordered feature names, numeric types, scaling policy, masks, and versioning.

**To-do**
- [ ] Create a feature registry with stable ordered names
- [ ] Define aggregation and missing-value/imputation policy
- [ ] Separate raw descriptors from normalized model-ready vectors
- [ ] Include source structure/version and coverage fields

### Step 2 — Assemble and normalize vectors

Join feature groups by canonical accession and structure version, then normalize only with fitted parameters.

**To-do**
- [ ] Join PP-23 and PP-24 outputs to protein master
- [ ] Validate feature completeness and duplicate structures
- [ ] Fit normalization parameters on training data only where relevant
- [ ] Persist preprocessing parameters alongside vector version

### Step 3 — Add quality and consistency checks

Detect invalid values, unexpected dimensionality, and feature drift.

**To-do**
- [ ] Check vector length, finite values, and schema hash
- [ ] Report missingness and outlier distributions
- [ ] Verify stable feature order across runs
- [ ] Test behavior when a protein has no usable structure

### Step 4 — Publish feature dataset

Write vectors and metadata in a format consumable by Milvus/ML tasks.

**To-do**
- [ ] Export Parquet/NumPy-compatible representation and manifest
- [ ] Provide read API/helper and sample records
- [ ] Document vector dimension and feature dictionary
- [ ] Hand off contract to PP-30, PP-33, and PP-34

## 4. Deliverables

- Versioned graph schema/service/import implementation for this ticket.
- Reproducible load/query examples and automated tests.
- Documentation of IDs, properties, query limits, and operational behavior.

## 5. Acceptance criteria

- [ ] All step-level to-dos are complete.
- [ ] Canonical protein accession remains the stable cross-system identifier.
- [ ] Imports/queries are bounded, observable, and safe to rerun where applicable.
- [ ] Graph data reconciles with the source dataset version.
- [ ] Unit and integration tests pass against a representative graph.

## 6. Testing strategy

- **Unit:** mapping, traversal construction, pagination, validation, and error handling.
- **Integration:** JanusGraph schema, import, and query operations.
- **Reconciliation:** vertex/edge counts, endpoint integrity, duplicate checks, and source lineage.
- **Performance:** measure batch throughput and query latency at representative graph size.

## 7. Cross-task handoff

- **Upstream:** PP-23 geometry features; PP-24 secondary structure/confidence; PP-22 residue coordinates; PP-20 protein master.
- **Consumers / coordination:** PP-20, PP-22, PP-23, PP-24, PP-30, PP-33, PP-34, PP-35
- **Contract rule:** publish graph schema version, key semantics, traversal contract, load/query metrics, and limitations. Coordinate schema changes with every consumer.

## 8. Completion notes

Record graph backend/version, source dataset version, load counts, query examples, test results, and deviations.
