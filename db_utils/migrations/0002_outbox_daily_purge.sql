-- Purge published outbox rows older than 1 day.
CREATE OR REPLACE FUNCTION purge_published_outbox_rows() RETURNS void AS $$
BEGIN
    DELETE FROM outbox
    WHERE published_at IS NOT NULL
      AND published_at < now() - INTERVAL '1 day';
END;
$$ LANGUAGE plpgsql;

-- Schedule daily cleanup if pg_cron is available in this Postgres installation.
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_available_extensions WHERE name = 'pg_cron') THEN
        CREATE EXTENSION IF NOT EXISTS pg_cron;
        PERFORM cron.schedule(
            '0 2 * * *',
            'SELECT purge_published_outbox_rows();'
        );
    ELSE
        RAISE NOTICE 'pg_cron is not available; skipping outbox daily purge scheduler setup.';
    END IF;
END;
$$;
