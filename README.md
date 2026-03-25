# Messaging Sprint
Demonstrating creation transmitting domain events between different services through a transactional outbox pattern with postgres and NATS

- single Event data type 
- common elements as fields in Event 
- JSON payload for specifics

## Database Startup Initialization (No CLI)

The relay initializes outbox schema automatically at startup via embedded SQLx migrations.

- If the target database does not exist yet, it is created first.
- Outbox migrations live in `outbox-relay/migrations`.
- `init_database()` connects, ensures database exists, and creates the `components` table if needed.
- `outbox-relay` applies only pending outbox migrations before it starts listening.
- No `sqlx-cli` step is required.
- Published outbox rows are purged by a daily DB job (retention: 1 day) when `pg_cron` is available.

Environment:

- Source of truth: root `.env`.
- Required database keys: `DB_HOST`, `DB_PORT`, `DB_USER`, `DB_PASSWORD`, `DB_NAME`
- Required relay keys: `NATS_URL`, `OUTBOX_NOTIFY_CHANNEL`, `OUTBOX_SUBJECT_PREFIX`, `OUTBOX_BATCH_SIZE`, `OUTBOX_FALLBACK_POLL_MS`
- Required dummy view keys: `DUMMY_VIEW_STREAM`, `DUMMY_VIEW_CONSUMER`

## End-to-End Demo (Outbox -> Relay -> Dummy View)

This workspace includes:

- a main application with  
    - a minimal cli to take user input
    - `component_service`, issuing domain events
    - a `component_repository`, persisting the `Component` domain aggregate and posting domain events into the outbox table (this is a demo short-cut, normally this should be done by the service triggering a separate outbox repository)
- `outbox-relay`: reads pending outbox rows and publishes to JetStream.
- `dummy-view-service`: consumes those events and sends explicit JetStream acks.

### Run with Docker Compose

From the project root folder,

1. run `docker compose --env-file .env -f docker/compose.yaml up` This will setup the database, the `outbox-relay`, the `dummy-view-service` and the NATS/JetStream server as docker containers
2. in a separate terminal, start the main app with `cargo run`
3. use the interactive cli to create a component

You can look into the database tables to see what was going on: 

`docker exec -it messaging-db-1 bash`
`postgres=# \c process`
`process=# select * from components;`
`process=# select * from outbox;`


After testing, you should tear the system down again:   
`docker compose -f docker/compose.yaml down`

### Run in separate terminals:
From the project root folder,

1. Start the database:  
`docker compose --env-file .env -f docker/postgres.yaml up`

2. Start NATS with JetStream enabled:  
`docker compose -f docker/nats.yaml up`  

2. Start the relay:  
	`cargo run -p outbox-relay`  

3. Start the dummy view service:  
	`cargo run -p dummy-view-service`  

4. Produce data/events from the app:  
	`cargo run -p messages`

CLI traces you will see:

- Relay prints `Publishing to subject ...` for each outbox event.
- Dummy view prints `received ...` and then `ack sent` for each message.

Optional env vars for the dummy consumer:

- `NATS_URL` (default `nats://127.0.0.1:4222`)
- `OUTBOX_SUBJECT_PREFIX` (default `events`)
- `DUMMY_VIEW_STREAM` (default `PROCESS_EVENTS`)
- `DUMMY_VIEW_CONSUMER` (default `DUMMY_VIEW_SERVICE`)

#### Observe Messaging With `nats` CLI

Useful commands while the demo is running:

1. Show streams:
	`nats stream ls`
2. Show stream details and message counters:
	`nats stream info PROCESS_EVENTS`
3. Show consumer state and ack progress:
	`nats consumer info PROCESS_EVENTS DUMMY_VIEW_SERVICE`
4. Watch raw subject traffic (core subscription view):
	`nats sub 'events.>'`

Notes:

- If `dummy-view-service` is running and acking, consumer `Ack Floor` and delivered counters should advance.
- If you stop `dummy-view-service`, messages remain in stream and are delivered when it resumes.
