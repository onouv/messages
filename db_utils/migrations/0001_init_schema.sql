CREATE TABLE IF NOT EXISTS components (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS outbox (
    id UUID PRIMARY KEY,
    aggregate_type TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    view_id TEXT NOT NULL,
    payload JSONB,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    published_at TIMESTAMPTZ,
    attempts INT NOT NULL DEFAULT 0,
    last_error TEXT
);

CREATE INDEX IF NOT EXISTS outbox_pending_idx
    ON outbox (published_at, occurred_at)
    WHERE published_at IS NULL;

-- Notify relay processes after insert commit. Notification payload is the outbox id.
CREATE OR REPLACE FUNCTION notify_outbox_new() RETURNS trigger AS $$
BEGIN
    PERFORM pg_notify('outbox_new', NEW.id::text);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS outbox_notify_insert ON outbox;
CREATE TRIGGER outbox_notify_insert
AFTER INSERT ON outbox
FOR EACH ROW EXECUTE FUNCTION notify_outbox_new();
