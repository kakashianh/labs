# Data flow

## Offline data lifecycle

~~~mermaid
flowchart TD
    A[External source releases] --> B[Source adapters]
    B --> C[Raw artifacts and source manifests]
    C --> D[Schema validation and quality checks]
    D --> E[Identifier mapping and normalization]
    E --> F[Canonical protein and PPI datasets]
    F --> G[Structure lookup and parsing]
    G --> H[Structural feature extraction]
    F --> I[Graph dataset and graph loading]
    F --> J[Training dataset assembly]
    H --> J
    J --> K[Model training and evaluation]
    K --> L[Versioned prediction outputs]
    F --> M[Embedding generation]
    H --> M
    M --> N[Vector indexing]
    F --> O[Metadata and artifact registry]
    H --> O
    K --> O
    L --> O
~~~

### Data-flow rules

1. Preserve raw source artifacts and source metadata before transformation.
2. Validate and normalize into canonical records before downstream graph, feature, or ML processing.
3. Keep source identifiers and mapping outcomes so records can be traced back to their origin.
4. Attach source/dataset versions to derived outputs; attach feature/model versions to generated features and predictions.
5. Store large files in object storage or the designated graph/vector store. Persist URI, checksum, format, and provenance in metadata.
6. Make batch stages restartable and deterministic where practical. A rerun with the same inputs and configuration should not silently create duplicate logical records.

## Online request lifecycle

~~~mermaid
sequenceDiagram
    actor User
    participant UI as Web dashboard
    participant API as FastAPI
    participant DB as PostgreSQL
    participant Graph as JanusGraph
    participant Vector as Milvus
    participant Store as Object storage

    User->>UI: Search or open a protein
    UI->>API: Search/detail request
    API->>DB: Resolve canonical protein and metadata
    opt Interaction neighborhood requested
        API->>Graph: Query PPI neighborhood
        Graph-->>API: Interaction records
    end
    opt Similar proteins requested
        API->>Vector: Similarity query
        Vector-->>API: Candidate protein IDs and scores
    end
    opt Structure artifact requested
        API->>Store: Resolve artifact URI
        Store-->>API: Artifact reference or authorized URL
    end
    API-->>UI: Stable response payload
    UI-->>User: Protein, structure, graph, and prediction views
~~~

## Failure and data-quality handling

| Condition | Required behavior |
|---|---|
| Unknown source identifier | Preserve the source record and mark mapping as unresolved; do not invent a canonical protein |
| Deprecated or merged accession | Preserve the submitted identifier and record the mapping outcome to the canonical accession when known |
| Missing structure | Represent structure availability as missing/unknown; do not infer that the protein has no structure |
| Missing GO annotation | Do not treat absence as a negative annotation |
| Invalid source row | Quarantine or reject with a recorded reason; retain enough provenance for diagnosis |
| Missing artifact | Return an explicit unavailable-artifact condition and retain metadata for repair |
| Stale model/dataset version | Expose version metadata so clients and operators can identify the provenance |

## Ownership

This document defines the logical lifecycle. PP-13–PP-20 implement acquisition and canonical datasets; PP-21–PP-25 implement structure processing; PP-26–PP-33 implement graph and vector capabilities; PP-34–PP-38 implement ML; PP-39–PP-42 implement API and UI behavior.
