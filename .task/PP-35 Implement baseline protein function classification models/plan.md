# PP-35 — Implement baseline protein function classification models

## 1. Objective

Establish reproducible baseline multi-label models and metrics before adding graph/feature-fusion complexity.

## 2. Scope and dependencies

PP-34 frozen train/validation/test dataset; PP-33 feature contract; PP-10 model conventions.

Related plans: [PP-33](../PP-33%20Integrate%20structure%2C%20sequence%20and%20PPI%20features/plan.md) · [PP-34](../PP-34%20Prepare%20multi-label%20protein%20function%20prediction%20dataset/plan.md) · [PP-36](../PP-36%20Implement%20combined%20feature%20and%20graph-based%20prediction/plan.md) · [PP-38](../PP-38%20Evaluate%20models%20and%20analyze%20prediction%20performance/plan.md). Preserve the shared protein accession, feature schema, annotation policy, and frozen split contract across these plans.

## 3. Step-by-step implementation and to-do

### Step 1 — Select baseline methods

Choose simple, interpretable baselines suitable for sparse multi-label GO prediction.

**To-do**
- [ ] Implement most-frequent/label-prior baseline
- [ ] Implement one-vs-rest logistic regression or linear classifier
- [ ] Add one tree-based or neural baseline if dataset size supports it
- [ ] Define threshold selection using validation data only

### Step 2 — Build training and inference pipeline

Use consistent preprocessing and train only on the frozen training split.

**To-do**
- [ ] Load versioned features, labels, and split manifest
- [ ] Fit imputation/scaling only on training data
- [ ] Train models with reproducible seeds and configuration
- [ ] Save model artifacts, label dictionary, and preprocessing objects

### Step 3 — Evaluate multi-label predictions

Calculate per-label and aggregate metrics with class imbalance awareness.

**To-do**
- [ ] Compute micro/macro F1 and precision/recall
- [ ] Compute per-label AP/AUPRC and thresholded metrics
- [ ] Compare validation-selected thresholds on held-out test set
- [ ] Report performance by label frequency

### Step 4 — Publish baseline benchmark

Create a reproducible benchmark report that PP-36 can compare against.

**To-do**
- [ ] Save predictions and metric artifacts for fixed test split
- [ ] Record hyperparameters, runtime, and model version
- [ ] Document baseline limitations and failure cases
- [ ] Provide common evaluation interface to PP-36 and PP-38

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

- **Upstream:** PP-34 frozen train/validation/test dataset; PP-33 feature contract; PP-10 model conventions.
- **Consumers / coordination:** PP-33, PP-34, PP-36, PP-38
- **Contract rule:** publish frozen split IDs, label dictionary, feature schema, model version, inference signature, and evaluation results. Any target/feature changes require synchronized updates to linked plans.

## 8. Completion notes

Record dataset/split version, label policy, feature version, model/config, metrics, artifact URI, and deviations.
