#![allow(unused_imports, unused_variables)]


use rdev::{listen, Event, EventType};

fn callback(event: Event) {
    if let EventType::KeyPress(key) = event.event_type {
        println!("Key pressed: {:?}", key);
        if key == rdev::Key::KeyA {
            // Trigger your custom script or command here
            println!("Key A was pressed",);    
        }
    }
}

fn main() {
    if let Err(error) = listen(callback) {
        println!("Error: {:?}", error)
    }
}

