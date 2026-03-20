use serde_json::Result;
pub struct Publisher {}

impl Publisher {
    pub fn publish(e: super::Event) -> Result<()> {
        let message = serde_json::to_string_pretty(&e)?;
        println!("{}", message);

        Ok(())
    }
}
