#![allow(unused_imports, unused_variables, dead_code, unused_mut, unused_must_use)]

extern crate rdev;
use rdev::{display_size, listen, simulate, Event, EventType, Button, Key, Keyboard, KeyboardState as RDKeyboardState, SimulateError};

// #[cfg(feature = "unstable_grab")]
use rdev::grab;

use std::{thread, time::{Duration, Instant}};
use std::collections::HashSet;
use std::fmt::{self, Debug};

// ketboard state
// #[derive(Debug)]
struct KeyboardState {
    keyboard: Box<dyn RDKeyboardState>,
    pressed_keys: HashSet<Key>,
}


impl KeyboardState {
    fn new() -> Self {
        // Initialize `KeyboardState` from rdev and an empty set of pressed keys
       let keyboard = Box::new(Keyboard::new()
            .expect("Failed to initialize Keyboard"));
        
       KeyboardState {
           keyboard,
           pressed_keys: HashSet::new(),
       }    
    }    

    fn update_state(&mut self, event_type: &EventType) -> Result<(), String> {
        match event_type {
            EventType::KeyPress(key) => {
                // Add key to the set if it's pressed
                self.pressed_keys.insert(*key);
                Ok(())
            }
            EventType::KeyRelease(key) => {
                // Remove key from the set if it's released
                self.pressed_keys.remove(key);
                Ok(())
            }
            _ => Err("Unsupported event type for keyboard".to_string()),
        }
    }

    // Check if a specific key is currently pressed
    fn is_key_pressed(&self, key: Key) -> bool {
        self.pressed_keys.contains(&key)
    }
}


// screen state
#[derive(Debug)]
struct ScreenState {
    width: u64,
    height: u64,
}

impl ScreenState {
    fn new() -> Self {
        let (width, height) = display_size().unwrap_or((0, 0));
        ScreenState { width, height }
    }
}



// mouse state
#[derive(Clone, Copy)]
struct MouseState {
    x: f64,
    y: f64,
    left_button_pressed: bool,
    right_button_pressed: bool,
    wheel_delta_x: i64,
    wheel_delta_y: i64,
}

impl Debug for MouseState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MouseState")
            .field("x", &self.x)
            .field("y", &self.y)
            .finish()
    }
}

impl MouseState {
    fn new() -> Self {
        MouseState {
            x: 0.0,
            y: 0.0,
            left_button_pressed: false,
            right_button_pressed: false,
            wheel_delta_x: 0,
            wheel_delta_y: 0,
        }
    }

    fn update_position(&mut self, new_x: f64, new_y: f64) {
        self.x = new_x;
        self.y = new_y;
        rdev_send(&EventType::MouseMove { x:self .x, y: self.y });
    }

    fn press_left_button(&mut self) {
        self.left_button_pressed = true;
    }

    fn release_left_button(&mut self) {
        self.left_button_pressed = false;
    }

    fn press_right_button(&mut self) {
        self.right_button_pressed = true;
    }

    fn release_right_button(&mut self) {
        self.right_button_pressed = false;
    }

    fn update_scroll(&mut self, delta_x: i64, delta_y: i64) {
        self.wheel_delta_x = delta_x;
        self.wheel_delta_y = delta_y;
    }
}


// #[derive(Debug)]
struct ActionCondition<F, A> {
    condition: F,
    action: A,
}

impl<F, A> ActionCondition<F, A> 
where
    F: FnMut(&MouseState, &ScreenState) -> bool,
    A: FnMut(&mut MouseState),
{
    fn new(condition: F, action: A) -> Self {
        ActionCondition { condition, action }
    }
}


fn rdev_listen_callback(event: Event) {
    match event.event_type {
        EventType::KeyPress(key) => {
            println!("key pressed: {:?}", key);

            if key == rdev::Key::KeyA {
                println!("Key A was pressed (rdev)!");
            }
        }
        EventType::KeyRelease(key) => {
            println!("key released: {:?}", key);
        }
        EventType::ButtonPress(button/* , pressed */) => {
            println!("mouse button pressed: {:?}", button);
        }        
        EventType::ButtonRelease(button/* , pressed */) => {
            println!("mouse button released: {:?}", button);
        }  
        EventType::MouseMove { x, y } => {
            println!("mouse coords: (x: {}, y: {})", x, y);
        }
        EventType::Wheel { delta_x, delta_y } => {
            println!("wheel delta: (x: {}, y: {})", delta_y, delta_x);
        }
        // _ => {}
    }
}


fn rdev_grab_callback(event: Event) -> Option<Event> {
    match event.event_type {
        EventType::KeyPress(key) => {
            println!("key pressed: {:?}", key);

            if key == rdev::Key::KeyA {
                println!("!! key A pressed that  was blocked");
                None // block the event
            }
            else { Some(event) }
        }
        EventType::KeyRelease(key) => {
            println!("key released: {:?}", key);
            Some(event)
        }
        EventType::ButtonPress(button/* , pressed */) => {
            println!("mouse button pressed: {:?}", button);
            Some(event)
        }        
        EventType::ButtonRelease(button/* , pressed */) => {
            println!("mouse button released: {:?}", button);
            Some(event)
        }  
        EventType::MouseMove { x, y } => {
            println!("mouse coords: (x: {}, y: {})", x, y);
            Some(event)
        }
        EventType::Wheel { delta_x, delta_y } => {
            println!("wheel delta: (x: {}, y: {})", delta_y, delta_x);
            Some(event)
        }
        // _ => {}
    }
}




/// simulating and sending events
fn rdev_send(event_type: &EventType) {
    let delay = Duration::from_millis(20);

    match simulate(event_type) {
        Ok(()) => (),
        Err(SimulateError) => {
            println!("We could not send {:?}", event_type);
        }
    }
    // Let ths OS catchup (at least MacOS)
    thread::sleep(delay);
}



fn rdev_send_loop(screen: &ScreenState, mouse: &mut MouseState) {
    loop {
        // Move the mouse to the top-left corner of the screen
        mouse.update_position(0.0, 0.0);
        rdev_send(&EventType::MouseMove { x: mouse.x, y: mouse.y });

        // Simulate a scroll event
        mouse.update_scroll(0, 1);
        rdev_send(&EventType::Wheel { delta_x: mouse.wheel_delta_x, delta_y: mouse.wheel_delta_y });

        // Example of simulating key press and release
        // rdev_send(&EventType::KeyPress(rdev::Key::KeyS));
        // rdev_send(&EventType::KeyRelease(rdev::Key::KeyS));


        

        thread::sleep(Duration::from_millis(60));
    }
}





fn rdev() {
    // Spawn a thread for rdev listening
    let rdev_thread = std::thread::spawn(move || {
        if let Err(error) = listen(rdev_listen_callback) {
            println!("Error: {:?}", error);
        }
    });
    
    // Wait for the rdev thread to finish (it won't, as it listens indefinitely)
    // let _ = rdev_thread.join();
}

struct MouseController {
    mouse_state: MouseState,
    mouse_dir_x: f64,
}

impl MouseController {
    fn new(initial_x: f64, initial_y: f64) -> Self {
        MouseController {
            mouse_state: MouseState::new(),
            mouse_dir_x: 1.0, // Start moving to the right
        }
    }

    fn update_position(&mut self, screen_width: u64) {
        // Update the mouse position based on the current direction
        self.mouse_state.x += self.mouse_dir_x * 20.0;

        self.mouse_state.update_position(self.mouse_state.x, self.mouse_state.y);

        // Check boundaries and reverse direction if necessary
        if self.mouse_state.x.round() as u64 >= screen_width || self.mouse_state.x < 0.0 {
            self.mouse_dir_x *= -1.0; // Reverse the direction
        }
    }
}



fn main() {

    let mut screen_state = ScreenState::new();
    let mut mouse_state = MouseState::new();
    let mut keyboard_state = KeyboardState::new();

// Example condition: Reset mouse x to 0 when reaching the right side of the screen
//     let move_delta = 50.0;
// let mut mouse_dir_x = 1.0; // Initially moving to the right

// let mut test_condition = ActionCondition::new(
//     |mouse, screen| {
//         mouse.x.round() as u64 >= screen.width || mouse.x.round() as u64 <= 0
//     },
//     |mouse| {
//         // mouse_dir_x *= -1.0; // Reverse the direction
//     },
// );

 let sceen_center_height  = screen_state.height / 2;
let mut mouse_controller = MouseController::new(0.0, sceen_center_height as f64);
    // List of conditions to apply in each loop iteration
    // let conditions = &mut[test_condition];
    
 mouse_controller.mouse_state.y =  sceen_center_height as f64;
mouse_controller.update_position(screen_state.width);

    rdev();

    // Move the mouse to the center of the screen
       // mouse_state.update_position(0.0, sceen_center_height as f64);
    // rdev_send(&EventType::MouseMove { x: 0.0, y: mouse_state.y });


    loop {
        // Update mouse position
        // mouse_state.x += 1.0;
  // mouse_state.update_position(mouse_state.x + (mouse_dir_x * move_delta), mouse_state.y);

mouse_controller.update_position(screen_state.width);
    // Check conditions and apply actions
    // for condition in conditions.iter_mut() { // Use iter_mut to mutate the conditions
    //     if (condition.condition)(&mouse_state, &screen_state) {
    //         (condition.action)(&mut mouse_state);
    //     }
    // }

        // Output mouse state for debugging
        println!("Mouse position: x = {}, y = {}", mouse_state.x, mouse_state.y);

        // Simulate a small delay in loop
        thread::sleep(Duration::from_millis(1));
    }



}







