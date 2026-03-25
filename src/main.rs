mod adapters;
mod application;
mod messaging;

use adapters::downstream::db::ComponentRepository;
use application::ComponentService;
use db_utils::init_database;
use dotenv::dotenv;
use inquire::{Select, Text};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let pool = match init_database().await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Failed to initialize database: {:?}", e);
            return;
        }
    };

    let repo = ComponentRepository::new(pool);
    let service = ComponentService::new(repo, "process");
        
    let mut finished = false;
    while !finished {
        let commands: Vec<&str> = vec!["Add Component", "Quit"];
        let command = Select::new("What do you want to do ?", commands).prompt();

        match command {
            Ok("Add Component") => {
                let id = Text::new("Component Id: ").prompt();
                if let Ok(id) = id {
                    let name = Text::new("Component Name: ").prompt();
                    if let Ok(name) = name {
                        match service.create_component(id.as_str(), name.as_str()).await {
                            Ok(component) => println!("Component created: {:?}", component),
                            Err(e) => println!("Error creating component: {:?}", e),
                        }
                    }
                }
            }
            _ => finished = true,
        }
    }
}




