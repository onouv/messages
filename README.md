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
