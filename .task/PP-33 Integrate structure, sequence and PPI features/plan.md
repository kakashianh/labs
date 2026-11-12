# PP-33 — Integrate structure, sequence and PPI features

## 1. Objective

Fuse structure-derived, sequence embedding, and graph-derived features into a consistent multimodal representation.

## 2. Scope and dependencies

PP-25 structural vectors; PP-30 sequence/structure embeddings; PP-29 graph features; PP-20 canonical protein master.

Related plans: [PP-20](../PP-20%20Generate%20protein%20master%20and%20PPI%20Parquet%20datasets/plan.md) · [PP-25](../PP-25%20Generate%20protein%20structure%20feature%20vectors/plan.md) · [PP-29](../PP-29%20Implement%20PPI%20network%20analytics%20and%20community%20detection/plan.md) · [PP-30](../PP-30%20Design%20protein%20embedding%20generation%20pipeline/plan.md) · [PP-31](../PP-31%20undefined/plan.md) · [PP-32](../PP-32%20undefined/plan.md) · [PP-34](../PP-34%20Prepare%20multi-label%20protein%20function%20prediction%20dataset/plan.md) · [PP-35](../PP-35%20Implement%20baseline%20protein%20function%20classification%20models/plan.md) · [PP-36](../PP-36%20Implement%20combined%20feature%20and%20graph-based%20prediction/plan.md) · [PP-37](../PP-37%20Implement%20disease-related%20protein%20clustering/plan.md). Preserve the shared protein accession, feature schema, annotation policy, and frozen split contract across these plans.

## 3. Step-by-step implementation and to-do

### Step 1 — Define feature fusion contract

Specify modality alignment, missing-modality policy, feature versioning, and leakage-safe preprocessing.

**To-do**
- [ ] Define canonical accession join key and one-row-per-protein target
- [ ] Define feature groups and dimensions for each modality
- [ ] Define missing modality masks/imputation policy
- [ ] Define train/validation/test split compatibility and preprocessing boundaries

### Step 2 — Build modality alignment pipeline

Join vectors and graph metrics with canonical protein records and retain provenance.

**To-do**
- [ ] Load versioned PP-25, PP-30, and PP-29 outputs
- [ ] Validate one-to-one or documented one-to-many joins
- [ ] Detect missing, duplicate, and stale modality records
- [ ] Preserve per-modality source/model/graph version metadata

### Step 3 — Implement fusion representations

Create baseline concatenation and optional modality-specific projections only when justified.

**To-do**
- [ ] Implement deterministic concatenation with fixed feature order
- [ ] Add modality masks for unavailable data
- [ ] Fit scaling/imputation on training split only
- [ ] Persist fusion schema and preprocessing artifacts

### Step 4 — Evaluate fusion data quality

Compare modality coverage and ensure fused records are suitable for downstream models.

**To-do**
- [ ] Report coverage per modality and combined coverage
- [ ] Check vector dimensions, finite values, and feature distributions
- [ ] Compare sample records to source modality records
- [ ] Publish fused dataset for PP-34–PP-38

## 4. Deliverables

- Reproducible dataset/model pipeline and versioned artifacts.
- Model/data cards describing scope, assumptions, limitations, and provenance.
- Automated tests and evaluation report on a frozen dataset split.

## 5. Acceptance criteria

- [ ] All step-level to-dos are complete.
- [ ] Canonical protein IDs and feature/label versions are consistent.
- [ ] Data leakage is prevented and preprocessing is fitted on training data only.
- [ ] Evaluation uses the same frozen splits and metrics for fair comparison.
- [ ] Model artifacts can be reloaded for inference and tests pass.

## 6. Testing strategy

- **Unit:** label encoding, split logic, feature mapping, model input/output, and metrics.
- **Integration:** dataset loader, graph/feature stores, and model serialization.
- **ML validation:** leakage checks, class imbalance, per-label performance, calibration/threshold checks.
- **Reproducibility:** rerun with fixed seeds and record software/model/data versions.

## 7. Cross-task handoff

- **Upstream:** PP-25 structural vectors; PP-30 sequence/structure embeddings; PP-29 graph features; PP-20 canonical protein master.
- **Consumers / coordination:** PP-20, PP-25, PP-29, PP-30, PP-31, PP-32, PP-34, PP-35, PP-36, PP-37
- **Contract rule:** publish frozen split IDs, label dictionary, feature schema, model version, inference signature, and evaluation results. Any target/feature changes require synchronized updates to linked plans.

## 8. Completion notes

Record dataset/split version, label policy, feature version, model/config, metrics, artifact URI, and deviations.
