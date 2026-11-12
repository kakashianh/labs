# PP-36 — Implement combined feature and graph-based prediction

## 1. Objective

Improve protein function prediction by combining multimodal feature vectors with PPI graph context.

## 2. Scope and dependencies

PP-33 fused features; PP-34 frozen labels/splits; PP-35 baselines; PP-28 graph queries; PP-29 graph metrics.

Related plans: [PP-28](../PP-28%20Implement%20graph%20query%20and%20neighborhood%20services/plan.md) · [PP-29](../PP-29%20Implement%20PPI%20network%20analytics%20and%20community%20detection/plan.md) · [PP-33](../PP-33%20Integrate%20structure%2C%20sequence%20and%20PPI%20features/plan.md) · [PP-34](../PP-34%20Prepare%20multi-label%20protein%20function%20prediction%20dataset/plan.md) · [PP-35](../PP-35%20Implement%20baseline%20protein%20function%20classification%20models/plan.md) · [PP-38](../PP-38%20Evaluate%20models%20and%20analyze%20prediction%20performance/plan.md). Preserve the shared protein accession, feature schema, annotation policy, and frozen split contract across these plans.

## 3. Step-by-step implementation and to-do

### Step 1 — Choose graph-aware model strategy

Compare a feature-plus-network baseline with an explicit graph neural model only if graph/data volume justifies it.

**To-do**
- [ ] Define baseline graph feature augmentation approach
- [ ] Select GNN architecture only with a clear message-passing design
- [ ] Define neighbor sampling and graph split leakage controls
- [ ] Keep feature-only PP-35 benchmark as control

### Step 2 — Build graph-feature training inputs

Align nodes, edges, features, and labels using canonical accession and frozen splits.

**To-do**
- [ ] Map graph vertex IDs to canonical protein accessions
- [ ] Construct adjacency/edge weights from PP-19/PP-27 graph version
- [ ] Attach PP-33 features and PP-34 labels
- [ ] Ensure test labels are not used in training neighborhoods/targets

### Step 3 — Train and tune combined model

Train with reproducible configuration and validation-only tuning.

**To-do**
- [ ] Implement model training and checkpointing
- [ ] Tune learning rate, depth, hidden dimension, and sampling on validation set
- [ ] Add class imbalance handling and early stopping
- [ ] Save model, graph snapshot, feature schema, and config

### Step 4 — Compare with baseline and publish

Evaluate against PP-35 using identical data splits and metrics.

**To-do**
- [ ] Run PP-35 baseline and graph-aware model on same test manifest
- [ ] Report micro/macro and per-label metrics
- [ ] Analyze gains, regressions, and graph coverage effects
- [ ] Publish inference contract and artifacts for PP-38/PP-39

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

- **Upstream:** PP-33 fused features; PP-34 frozen labels/splits; PP-35 baselines; PP-28 graph queries; PP-29 graph metrics.
- **Consumers / coordination:** PP-28, PP-29, PP-33, PP-34, PP-35, PP-38
- **Contract rule:** publish frozen split IDs, label dictionary, feature schema, model version, inference signature, and evaluation results. Any target/feature changes require synchronized updates to linked plans.

## 8. Completion notes

Record dataset/split version, label policy, feature version, model/config, metrics, artifact URI, and deviations.
