use anyhow::{Context, Result};
use async_nats::jetstream;
use async_nats::jetstream::consumer;
use async_nats::jetstream::stream;
use dotenv::dotenv;
use futures::StreamExt;

#[derive(Debug, Clone)]
struct Config {
    nats_url: String,
    subject_prefix: String,
    stream_name: String,
    consumer_name: String,
}

impl Config {
    fn from_env() -> Result<Self> {
        Ok(Self {
            nats_url: require_env("NATS_URL")?,
            subject_prefix: require_env("OUTBOX_SUBJECT_PREFIX")?,
            stream_name: require_env("DUMMY_VIEW_STREAM")?,
            consumer_name: require_env("DUMMY_VIEW_CONSUMER")?,
        })
    }
}

fn require_env(name: &str) -> Result<String> {
    std::env::var(name).with_context(|| format!("missing required environment variable: {name}"))
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let cfg = Config::from_env()?;
    println!("[dummy-view-service] starting");
    println!("[dummy-view-service] NATS_URL={}", cfg.nats_url);

    let nats = async_nats::connect(&cfg.nats_url)
        .await
        .with_context(|| format!("failed to connect to NATS at {}", cfg.nats_url))?;
    let js = jetstream::new(nats);

    let subject_filter = format!("{}.>", cfg.subject_prefix);

    let stream = js
        .get_or_create_stream(stream::Config {
            name: cfg.stream_name.clone(),
            subjects: vec![subject_filter.clone()],
            ..Default::default()
        })
        .await
        .with_context(|| format!("failed to create/get stream {}", cfg.stream_name))?;

    println!(
        "[dummy-view-service] stream={} subjects=[{}]",
        cfg.stream_name, subject_filter
    );

    let consumer = stream
        .get_or_create_consumer(
            &cfg.consumer_name,
            consumer::pull::Config {
                durable_name: Some(cfg.consumer_name.clone()),
                ack_policy: consumer::AckPolicy::Explicit,
                ..Default::default()
            },
        )
        .await
        .with_context(|| format!("failed to create/get consumer {}", cfg.consumer_name))?;

    println!(
        "[dummy-view-service] consumer={} ack_policy=explicit",
        cfg.consumer_name
    );
    println!("[dummy-view-service] waiting for events. Press Ctrl-C to stop.");

    let mut messages = consumer.messages().await?;

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("[dummy-view-service] ctrl-c received, shutting down");
                break;
            }
            maybe_message = messages.next() => {
                match maybe_message {
                    Some(Ok(message)) => {
                        let payload = String::from_utf8_lossy(&message.payload);
                        println!("[dummy-view-service] received subject={} payload={}", message.subject, payload);

                        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&message.payload) {
                            let id = json.get("id").and_then(|v| v.as_str()).unwrap_or("<no-id>");
                            let event_type = json.get("event_type").and_then(|v| v.as_str()).unwrap_or("<no-event-type>");
                            println!("[dummy-view-service] parsed id={} event_type={}", id, event_type);
                        }

                        match message.ack().await {
                            Ok(_) => println!("[dummy-view-service] ack sent"),
                            Err(err) => println!("[dummy-view-service] ack failed: {err}"),
                        }
                    }
                    Some(Err(err)) => {
                        println!("[dummy-view-service] receive error: {err}");
                    }
                    None => {
                        println!("[dummy-view-service] message stream closed");
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
