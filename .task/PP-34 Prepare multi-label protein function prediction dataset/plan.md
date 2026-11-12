# PP-34 — Prepare multi-label protein function prediction dataset

## 1. Objective

Build a leakage-aware supervised dataset using GO annotations as multi-label protein function targets.

## 2. Scope and dependencies

PP-15 GO annotations; PP-20 protein master; PP-33 fused features; PP-10 data contracts.

Related plans: [PP-15](../PP-15%20Integrate%20UniProt%20ID%20mapping%20and%20Gene%20Ontology%20annotations/plan.md) · [PP-20](../PP-20%20Generate%20protein%20master%20and%20PPI%20Parquet%20datasets/plan.md) · [PP-25](../PP-25%20Generate%20protein%20structure%20feature%20vectors/plan.md) · [PP-29](../PP-29%20Implement%20PPI%20network%20analytics%20and%20community%20detection/plan.md) · [PP-30](../PP-30%20Design%20protein%20embedding%20generation%20pipeline/plan.md) · [PP-33](../PP-33%20Integrate%20structure%2C%20sequence%20and%20PPI%20features/plan.md) · [PP-35](../PP-35%20Implement%20baseline%20protein%20function%20classification%20models/plan.md) · [PP-36](../PP-36%20Implement%20combined%20feature%20and%20graph-based%20prediction/plan.md) · [PP-38](../PP-38%20Evaluate%20models%20and%20analyze%20prediction%20performance/plan.md). Preserve the shared protein accession, feature schema, annotation policy, and frozen split contract across these plans.

## 3. Step-by-step implementation and to-do

### Step 1 — Define target label semantics

Select GO namespace(s), evidence-code policy, ontology propagation policy, and minimum support thresholds.

**To-do**
- [ ] Define whether target is MF, BP, CC, or selected namespaces
- [ ] Define evidence codes included/excluded and annotation date cutoff
- [ ] Define obsolete-term and ontology propagation handling
- [ ] Define minimum positive count per label

### Step 2 — Construct protein-label matrix

Join canonical proteins to curated GO terms and encode multi-hot targets.

**To-do**
- [ ] Normalize GO IDs and deduplicate annotation evidence
- [ ] Build sparse protein-by-GO target matrix
- [ ] Preserve label metadata and evidence provenance
- [ ] Exclude proteins without reliable target labels from supervised evaluation

### Step 3 — Create leakage-aware data splits

Split by protein and, where appropriate, sequence similarity/cluster to reduce homology leakage.

**To-do**
- [ ] Define train/validation/test split strategy and random seed
- [ ] Prevent identical proteins/isoforms crossing splits
- [ ] Consider sequence-cluster-aware split and report it
- [ ] Freeze split manifest for all model comparisons

### Step 4 — Publish ML-ready dataset

Package features, labels, split manifest, and label dictionary with version metadata.

**To-do**
- [ ] Join PP-33 features to label matrix by canonical accession
- [ ] Report class imbalance and label frequency distribution
- [ ] Export sparse targets and dataset manifest
- [ ] Hand off frozen dataset to PP-35–PP-38

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

- **Upstream:** PP-15 GO annotations; PP-20 protein master; PP-33 fused features; PP-10 data contracts.
- **Consumers / coordination:** PP-15, PP-20, PP-25, PP-29, PP-30, PP-33, PP-35, PP-36, PP-38
- **Contract rule:** publish frozen split IDs, label dictionary, feature schema, model version, inference signature, and evaluation results. Any target/feature changes require synchronized updates to linked plans.

## 8. Completion notes

Record dataset/split version, label policy, feature version, model/config, metrics, artifact URI, and deviations.
