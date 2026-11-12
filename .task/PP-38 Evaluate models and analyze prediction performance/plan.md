# PP-38 — Evaluate models and analyze prediction performance

## 1. Objective

Provide a rigorous, reproducible evaluation of baseline and graph-aware models, including limitations and error analysis.

## 2. Scope and dependencies

PP-34 frozen dataset/splits; PP-35 baseline; PP-36 combined model; PP-37 clustering outputs where relevant.

Related plans: [PP-34](../PP-34%20Prepare%20multi-label%20protein%20function%20prediction%20dataset/plan.md) · [PP-35](../PP-35%20Implement%20baseline%20protein%20function%20classification%20models/plan.md) · [PP-36](../PP-36%20Implement%20combined%20feature%20and%20graph-based%20prediction/plan.md) · [PP-37](../PP-37%20Implement%20disease-related%20protein%20clustering/plan.md) · [PP-39](../PP-39%20Implement%20protein%20search%20and%20detail%20REST%20APIs/plan.md) · [PP-42](../PP-42%20Integrate%20prediction%20results%20and%20perform%20end-to-end%20testing/plan.md). These plans define the data/API handoffs; preserve their identifiers, versions, and error semantics.

## 3. Step-by-step implementation and to-do

### Step 1 — Define evaluation protocol

Freeze metrics, thresholds, aggregation, and comparison rules before test evaluation.

**To-do**
- [ ] Define micro/macro F1, precision, recall, AP/AUPRC, and per-label metrics
- [ ] Define threshold selection using validation set only
- [ ] Define baseline comparison and confidence interval strategy
- [ ] Document class-frequency buckets and reporting rules

### Step 2 — Run controlled model evaluation

Evaluate all models against identical split manifests and feature/label versions.

**To-do**
- [ ] Load saved PP-35 and PP-36 artifacts
- [ ] Verify dataset, label dictionary, and split hashes match
- [ ] Generate test predictions once using fixed configuration
- [ ] Save raw scores, thresholded outputs, and metric artifacts

### Step 3 — Perform error and robustness analysis

Investigate where predictions fail and how coverage, imbalance, and graph structure affect outcomes.

**To-do**
- [ ] Analyze false positives/negatives by GO term and frequency
- [ ] Evaluate performance by protein/structure/graph coverage
- [ ] Inspect representative high-confidence errors
- [ ] Test sensitivity to confidence cutoff and missing modalities

### Step 4 — Publish evaluation report and release gate

Summarize evidence and define what is safe to expose through API/UI.

**To-do**
- [ ] Produce tables/plots with metric definitions and sample counts
- [ ] Document limitations, leakage checks, and reproducibility metadata
- [ ] Define model acceptance/release criteria
- [ ] Provide model card and inference metadata for PP-39/PP-42

## 4. Deliverables

- Working analysis/API capability and documented contract.
- Reproducible outputs, representative examples, and version/provenance metadata.
- Automated tests, operational guidance, and known limitations.

## 5. Acceptance criteria

- [ ] All step-level to-dos are complete.
- [ ] Inputs, outputs, score semantics, and version metadata are documented.
- [ ] Invalid, missing, and unavailable data/backends are handled explicitly.
- [ ] Resource limits and observability are in place where applicable.
- [ ] Unit and integration tests pass on representative fixtures.

## 6. Testing strategy

- **Unit:** business rules, validation, mapping, and error handling.
- **Integration:** relevant database, graph, vector, model, and REST boundaries.
- **Quality:** reproducibility, referential integrity, metric correctness, and provenance.
- **Performance:** query latency, response size, and bounded resource use.

## 7. Cross-task handoff

- **Upstream:** PP-34 frozen dataset/splits; PP-35 baseline; PP-36 combined model; PP-37 clustering outputs where relevant.
- **Consumers / coordination:** PP-34, PP-35, PP-36, PP-37, PP-39, PP-42
- **Contract rule:** publish API/schema versions, sample requests/responses, evaluation or quality results, and limitations. Coordinate breaking changes with all linked plans.

## 8. Completion notes

Record data/model/backend versions, commands, test results, sample outputs, and deviations.
