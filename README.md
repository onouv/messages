# Messaging Sprint

Demonstrating 2 Methods of creating domain events / messages in Rust 

Method 1:   
- dedicated data types per event 
- named fields with specifics
- common header
- helper trait for common header access

Method 2:  
- single Event data type 
- common elements as fields in Event 
- JSON payload for specifics

## Database Startup Initialization (No CLI)

The application initializes schema automatically at startup via embedded SQLx migrations
in `db_utils`.

- If the target database does not exist yet, it is created first.
- Migrations live in `db_utils/migrations`.
- `init_database()` connects and runs only pending migrations.
- No `sqlx-cli` step is required.

Environment:

- Source of truth: `.env` with `DB_TYPE`, `DB_HOST`, `DB_PORT`, `DB_USER`, `DB_PASSWORD`, `DB_NAME`
