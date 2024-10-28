#![allow(unused_imports, unused_variables, dead_code, unused_mut, unused_must_use)]

extern crate rdev;
use rdev::{listen, Event, EventType};

extern crate device_query_revamped;
use device_query_revamped::{DeviceQuery, DeviceState, MouseState, Keycode};
use std::collections::HashSet;
use std::thread;
use std::time::{Duration, Instant};


fn rdev_callback(event: Event) {
    match event.event_type {
        EventType::KeyPress(key) => {
            println!("key pressed: {:?}", key);
        }
        EventType::KeyRelease(key) => {
            println!("key released: {:?}", key);
        }
        EventType::MouseMove { x, y } => {
            println!("mouse coords: (x: {}, y: {})", x, y);
        }
        EventType::ButtonPress(button/* , is_pressed */) => {
            println!("mouse button pressed: {:?}", button);
        }        
        EventType::ButtonRelease(button/* , is_pressed */) => {
            println!("mouse button released: {:?}", button);
        }  
        _ => {}
    }
}

fn rdev() {
    if let Err(error) = listen(rdev_callback) {
        println!("Error: {:?}", error)
    }

}


// dq is inferior to rdev because
//
// On Ubuntu/Debian:
//
// sudo apt install libx11-dev
// On Fedora/RHEL/CentOS:
//
// sudo dnf install xorg-x11-server-devel


fn dq() {
    let device_state = DeviceState::new();
    let mut last_keys: HashSet<Keycode> = HashSet::new();
    // let mut keys: Vec<Keycode> = device_state.get_keys();
println!("Is A pressed? {}", last_keys.contains(&Keycode::A));


let mut last_mouse: MouseState = device_state.get_mouse();

        
    let mut last_mouse_event_time = Instant::now(); // Track last mouse event time

println!("Current Mouse Coordinates: {:?}", last_mouse.coords);

    loop {
        let keys: HashSet<Keycode> = device_state.get_keys().into_iter().collect();
        let mouse: MouseState = device_state.get_mouse();
        
        // Check if the last event was a mouse event
        if last_mouse_event_time.elapsed() < Duration::from_millis(50) {
            // Small delay to allow state refresh
            thread::sleep(Duration::from_millis(10)); // Adjust as necessary
        }

        // Detect newly pressed keys
        for key in &keys {
            if !last_keys.contains(key) {
                println!("key pressed: {:?}", key);
                if *key == Keycode::A {
                    println!("'A' key is pressed (device_query)!");
                }
            }
        }

        // Detect released keys
        for key in &last_keys {
            if !keys.contains(key) {
                println!("Key released: {:?}", key);
            }
        }

        // Update last keys to the current state
        last_keys = keys;
                
        if mouse != last_mouse {
            println!("{:?}", mouse);
            // last_mouse = mouse;
        }
        
        last_mouse = device_state.get_mouse();

        // Prevent high CPU usage by sleeping briefly
        thread::sleep(Duration::from_millis(50));
    }
}

fn main() {
    // dq();

    rdev();
}

