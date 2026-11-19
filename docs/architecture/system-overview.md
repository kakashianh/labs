# System overview

## Purpose

Protein3D-PPI integrates protein identity and sequence data, protein-protein interactions (PPI), Gene Ontology (GO) annotations, three-dimensional structure information, derived features, embeddings, and multi-label function predictions. The platform supports both offline data preparation/model workflows and online protein discovery and visualization.

## System context

~~~mermaid
flowchart LR
    Sources[External biological sources]
    Ingest[Ingestion adapters]
    Raw[(Raw files and manifests)]
    Batch[PySpark processing]
    Domain[Shared domain contracts]
    Struct[Structure parsing and features]
    Graph[(JanusGraph)]
    Vector[(Milvus)]
    Meta[(PostgreSQL metadata)]
    Objects[(MinIO object storage)]
    ML[Training and inference]
    API[FastAPI]
    UI[Web dashboard]

    Sources --> Ingest --> Raw
    Raw --> Batch
    Batch --> Meta
    Batch --> Objects
    Batch --> Graph
    Batch --> Struct
    Struct --> Objects
    Struct --> ML
    Batch --> ML
    ML --> Meta
    ML --> Vector
    Domain -. contracts .- Ingest
    Domain -. contracts .- Batch
    Domain -. contracts .- Struct
    Domain -. contracts .- ML
    API --> Meta
    API --> Graph
    API --> Vector
    API --> Objects
    UI --> API
~~~

The diagram is logical, not a deployment topology. PP-11 owns the concrete local service topology and runtime configuration.

## Component responsibilities

| Component | Responsibility | Primary output |
|---|---|---|
| External source adapters | Retrieve source records and preserve source identifiers | Raw source files and manifests |
| Ingestion and normalization | Validate, normalize, map identifiers, and produce deterministic datasets | Canonical protein/PPI/annotation datasets |
| Structure processing | Parse PDB/mmCIF and derive structural descriptors | Structure metadata and feature artifacts |
| Metadata store | Hold relational metadata, provenance, job state, and artifact references | Queryable metadata |
| Graph store | Represent protein nodes and interaction relationships | PPI neighborhood and graph traversals |
| Vector store | Index model-generated embeddings | Similarity search results |
| ML workflows | Prepare datasets, train/evaluate models, and generate function predictions | Versioned predictions and model artifacts |
| API | Orchestrate online reads and expose stable response contracts | Search, protein detail, and prediction responses |
| Dashboard | Present protein details, 3D structures, PPI networks, and prediction information | User-facing exploration |

## Operating modes

### Offline/batch

Batch workloads acquire and normalize source data, build canonical datasets, parse structures, calculate features, create graph/vector indexes, and train or evaluate models. Jobs must be reproducible from recorded source versions, configuration, and output manifests.

### Online

The API serves interactive requests. It reads metadata and prediction records, queries graph/vector stores where needed, and resolves artifact references for the dashboard. Online handlers should not launch full Spark jobs or perform expensive dataset-wide transformations.

## Cross-cutting concerns

- **Identity:** canonical UniProt accession is the cross-system protein identifier; source IDs are retained in mapping records.
- **Provenance:** source, dataset, feature, embedding, and model versions are recorded where applicable.
- **Artifact management:** large files are referenced by URI and checksum, not copied into metadata rows.
- **Data semantics:** unknown/missing values are distinct from negative biological assertions and numeric zero.
- **Observability:** jobs and API requests should emit structured logs and useful failure context without exposing secrets.
- **Reproducibility:** deterministic outputs and explicit version metadata are preferred for batch workflows.
