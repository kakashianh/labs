# Contract: Protein function prediction

## Purpose

Represent a model prediction linking a protein to a GO term or other explicitly versioned target label.

## Fields

| Field | Meaning | Required |
|---|---|---:|
| protein_id | Canonical protein reference | Yes |
| go_id | Predicted GO term | Yes |
| score | Model output score | Yes |
| score_type | Probability, logit, ranking score, or other defined semantics | Yes |
| model_name | Model identity | Yes |
| model_version | Model version/artifact digest | Yes |
| dataset_version | Training/evaluation dataset version | Yes |
| threshold | Applied decision threshold, when binarized | When applicable |
| generated_at | Prediction timestamp | Yes |
| schema_version | Contract version | Yes |

## Rules

- score must always have an explicit score_type; never assume every score is a calibrated probability.
- Preserve model and dataset versions so predictions can be reproduced and compared.
- Keep raw score separate from any thresholded label.
- A missing prediction is not equivalent to a negative prediction.
- Evaluation metrics and calibration policy are owned by the ML tasks PP-34–PP-38.
