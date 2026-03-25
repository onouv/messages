CREATE OR REPLACE FUNCTION purge_published_outbox_rows() RETURNS void AS $$
BEGIN
    DELETE FROM outbox
    WHERE published_at IS NOT NULL
      AND published_at < now() - INTERVAL '1 day';
END;
$$ LANGUAGE plpgsql;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_available_extensions WHERE name = 'pg_cron') THEN
        CREATE EXTENSION IF NOT EXISTS pg_cron;

        IF NOT EXISTS (
            SELECT 1
            FROM cron.job
            WHERE command = 'SELECT purge_published_outbox_rows();'
        ) THEN
            PERFORM cron.schedule(
                '0 2 * * *',
                'SELECT purge_published_outbox_rows();'
            );
        END IF;
    ELSE
        RAISE NOTICE 'pg_cron is not available; skipping outbox daily purge scheduler setup.';
    END IF;
END;
$$;