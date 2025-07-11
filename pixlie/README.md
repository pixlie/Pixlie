# Pixlie Backend

This directory contains the Rust backend for Pixlie, an application for smart entity analysis of Hacker News discussions.

## Features

-   Manages data storage for Hacker News items and users.
-   Provides an API for controlling data downloads from Hacker News.
-   Handles entity extraction from downloaded content using ONNX models.
-   Serves a web API for the frontend application.

## Development

### Prerequisites

-   Rust (latest stable version recommended)
-   Cargo

### Building

To build the backend application:

```bash
cargo build
```

### Running

To run the backend server:

```bash
cargo run
```
The server will typically start on `http://localhost:8080` (or as configured).

### Generating TypeScript Types for the API

The Pixlie backend uses `ts-rs` to generate TypeScript type definitions from the Rust structs used in its API. This ensures type safety between the frontend and backend.

The generated types are output to `webapp/src/types/api.ts`.

**When to Regenerate Types:**

You should regenerate the TypeScript types whenever you make changes to the Rust structs that are part of the API request or response bodies. This includes:

-   Structs in `pixlie::handlers` that are serialized/deserialized.
-   Supporting structs in `pixlie::database` or `pixlie::entity_extraction` that are embedded in API structs (e.g., `DownloadStats`, `ModelInfo`).

**How to Regenerate Types:**

1.  Ensure you have Rust and Cargo installed.
2.  Navigate to the `pixlie` directory (this directory).
3.  Run the following command:

    ```bash
    cargo run --bin export_types
    ```

This will execute the `src/bin/export_types.rs` script, which inspects the annotated Rust structs and writes the corresponding TypeScript interfaces to `../../webapp/src/types/api.ts`.

After running the script, review the changes in `api.ts` and update the frontend code in the `webapp/` directory as necessary to use the new or modified types.
