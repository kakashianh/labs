# KAN-6 sample GIS data

This folder contains source references and reproducible outputs for the TP.HCM 2D OSM dataset.

- `source/`: verified HCMC boundary input and instructions for obtaining the Geofabrik PBF.
- `processed/`: generated GeoJSON boundary and building footprints.

See `scripts/gis/README.md` for prerequisites and execution. See the KAN-6 deliverables for metadata, field definitions, and quality results.

The PBF and temporary files are excluded from Git. Do not commit processed GeoJSON until it has been generated from real source data and validated in QGIS.
