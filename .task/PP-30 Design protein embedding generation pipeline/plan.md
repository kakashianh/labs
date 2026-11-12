# PP-30 — Design protein embedding generation pipeline

## 1. Objective

Define a reproducible pipeline to create sequence and structural embeddings with consistent IDs, dimensions, and model provenance.

## 2. Scope and dependencies

PP-20 protein master; PP-21/PP-22 structure parsing and coordinates; PP-25 structure feature vectors; PP-11 compute/runtime.

Related plans: [PP-20](../PP-20%20Generate%20protein%20master%20and%20PPI%20Parquet%20datasets/plan.md) · [PP-21](../PP-21%20Implement%20PDB%2FmmCIF%20structure%20parser/plan.md) · [PP-22](../PP-22%20Extract%20protein%20residues%20and%20atomic%20coordinates/plan.md) · [PP-25](../PP-25%20Generate%20protein%20structure%20feature%20vectors/plan.md) · [PP-31](../PP-31%20Configure%20Milvus%20collections%20and%20vector%20indexing/plan.md) · [PP-32](../PP-32%20Implement%20protein%20similarity%20search/plan.md) · [PP-33](../PP-33%20Integrate%20structure%2C%20sequence%20and%20PPI%20features/plan.md) · [PP-34](../PP-34%20Prepare%20multi-label%20protein%20function%20prediction%20dataset/plan.md) · [PP-35](../PP-35%20Implement%20baseline%20protein%20function%20classification%20models/plan.md). These links define the expected producer/consumer relationships for this feature.

## 3. Step-by-step implementation and to-do

### Step 1 — Select embedding modalities and models

Choose sequence and optional structure encoders based on compute budget and project scope.

**To-do**
- [ ] Define sequence embedding model/version (e.g. ESM-2 checkpoint)
- [ ] Define structural embedding approach and required inputs
- [ ] Record licensing, checkpoint source, hardware, and expected dimensions
- [ ] Define truncation/windowing policy for long sequences

### Step 2 — Define embedding record contract

Specify canonical ID, modality, vector dimension, model version, normalization, and source artifact references.

**To-do**
- [ ] Define embedding schema and unique key
- [ ] Define float precision and vector normalization policy
- [ ] Include model/checkpoint hash and preprocessing version
- [ ] Define missing-structure and failed-inference states

### Step 3 — Implement batch inference workflow

Build deterministic, resumable embedding generation with batching and resource controls.

**To-do**
- [ ] Implement sequence tokenization and batching
- [ ] Implement structural input preparation when selected
- [ ] Add GPU/CPU fallback and memory-aware batch sizing
- [ ] Checkpoint per batch and record failed proteins

### Step 4 — Validate and publish embeddings

Verify vector dimensions, numeric validity, and repeatability before indexing.

**To-do**
- [ ] Check dimensions and finite values for every vector
- [ ] Compare repeat runs for deterministic behavior where expected
- [ ] Measure throughput and memory on representative sample
- [ ] Publish vector manifest and model card for PP-31–PP-33

## 4. Deliverables

- Tested implementation and versioned data/service contract.
- Reproducible sample run and representative fixtures.
- Configuration, provenance, monitoring, and operational documentation.

## 5. Acceptance criteria

- [ ] All step-level to-dos are complete.
- [ ] Identifiers and model/index versions are traceable.
- [ ] Input validation, bounded resource use, and failure behavior are implemented.
- [ ] Quality and performance are measured on a representative sample.
- [ ] Unit and integration tests pass and consumers can use the documented contract.

## 6. Testing strategy

- **Unit:** algorithms, transformations, schema checks, score semantics, and edge cases.
- **Integration:** graph/vector infrastructure and metadata lookups.
- **Quality:** reproducibility, coverage, dimensionality, ranking correctness, and provenance.
- **Performance:** batch throughput, memory, index build time, and query latency as applicable.

## 7. Cross-task handoff

- **Upstream:** PP-20 protein master; PP-21/PP-22 structure parsing and coordinates; PP-25 structure feature vectors; PP-11 compute/runtime.
- **Consumers / coordination:** PP-20, PP-21, PP-22, PP-25, PP-31, PP-32, PP-33, PP-34, PP-35
- **Contract rule:** publish schemas, versions, parameters, evaluation results, and limitations. Do not change embedding dimensions, score semantics, or graph feature definitions without coordinating with linked consumers.

## 8. Completion notes

Record model/data/index versions, parameters, artifact locations, quality/performance results, and deviations.
