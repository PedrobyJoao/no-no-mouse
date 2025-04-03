use anyhow::{Context, Result};
use clap::Parser;
use evdev::{Device, EventType, InputEvent};
use log::{debug, info, warn};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use uinput::event::controller::Mouse;

// CLI arguments
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Path to configuration file
    #[clap(short, long, value_parser)]
    config: Option<PathBuf>,
}

// Configuration structure
#[derive(Debug)]
struct Config {
    keyboard_device: String,
    base_speed: i32,
    shift_multiplier: i32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            keyboard_device: "/dev/input/event0".to_string(),
            base_speed: 5,
            shift_multiplier: 3,
        }
    }
}

fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();
    info!("Starting keyboard-mouse control");

    // Set up signal handler for clean shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        info!("Received termination signal, shutting down...");
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    // Parse command line arguments
    let args = Args::parse();

    // Load configuration
    let config = match args.config {
        Some(path) => load_config(&path)?,
        None => {
            warn!("No config file specified, using defaults");
            Config::default()
        }
    };

    debug!("Using configuration: {:?}", config);

    // Open keyboard device
    let mut keyboard = Device::open(&config.keyboard_device)
        .context(format!("Failed to open keyboard device: {}", config.keyboard_device))?;

    // Set keyboard to non-blocking mode
    keyboard.grab().context("Failed to grab keyboard device")?;

    // Create virtual mouse device
    let mut mouse = uinput::device::Builder::default()
        .unwrap()
        .name("keyboard-mouse")
        .unwrap()
        // Enable relative axis events (mouse movement)
        .event(uinput::event::relative::Position::X)
        .unwrap()
        .event(uinput::event::relative::Position::Y)
        .unwrap()
        // Enable mouse button events
        .event(uinput::event::controller::Controller::Mouse(Mouse::Left))
        .unwrap()
        .event(uinput::event::controller::Controller::Mouse(Mouse::Right))
        .unwrap()
        .create()
        .context("Failed to create virtual mouse device")?;

    info!("Virtual mouse device created");

    // Track key states
    let mut h_pressed = false;
    let mut j_pressed = false;
    let mut k_pressed = false;
    let mut l_pressed = false;
    let mut shift_pressed = false;

    // Main event loop
    while running.load(Ordering::SeqCst) {
        // Read events from keyboard
        if let Ok(events) = keyboard.fetch_events() {
            for event in events {
                process_event(
                    event,
                    &mut h_pressed,
                    &mut j_pressed,
                    &mut k_pressed,
                    &mut l_pressed,
                    &mut shift_pressed,
                    &mut mouse,
                    &config,
                )?;
            }
        }

        // Move mouse based on current key states
        move_mouse(
            h_pressed,
            j_pressed,
            k_pressed,
            l_pressed,
            shift_pressed,
            &mut mouse,
            &config,
        )?;

        // Small sleep to avoid consuming too much CPU
        std::thread::sleep(Duration::from_millis(10));
    }

    // Clean up resources before exiting
    info!("Releasing keyboard device");
    keyboard.ungrab().ok();
    info!("Cleanup complete, exiting");
    
    Ok(())
}

fn load_config(path: &PathBuf) -> Result<Config> {
    let config_str = std::fs::read_to_string(path)
        .context(format!("Failed to read config file: {:?}", path))?;
    
    let config_toml: toml::Value = toml::from_str(&config_str)
        .context("Failed to parse TOML config")?;
    
    let keyboard_device = config_toml
        .get("keyboard_device")
        .and_then(|v| v.as_str())
        .unwrap_or("/dev/input/event0")
        .to_string();
    
    let base_speed = config_toml
        .get("base_speed")
        .and_then(|v| v.as_integer())
        .unwrap_or(5) as i32;
    
    let shift_multiplier = config_toml
        .get("shift_multiplier")
        .and_then(|v| v.as_integer())
        .unwrap_or(3) as i32;
    
    Ok(Config {
        keyboard_device,
        base_speed,
        shift_multiplier,
    })
}

fn process_event(
    event: InputEvent,
    h_pressed: &mut bool,
    j_pressed: &mut bool,
    k_pressed: &mut bool,
    l_pressed: &mut bool,
    shift_pressed: &mut bool,
    mouse: &mut uinput::Device,
    _config: &Config,
) -> Result<()> {
    // Only process key events
    if event.event_type() != EventType::KEY {
        return Ok(());
    }

    let key_code = event.code();
    let pressed = event.value() != 0;

    // Define key codes as constants
    const KEY_H: u16 = evdev::Key::KEY_H.code();
    const KEY_J: u16 = evdev::Key::KEY_J.code();
    const KEY_K: u16 = evdev::Key::KEY_K.code();
    const KEY_L: u16 = evdev::Key::KEY_L.code();
    const KEY_LEFTSHIFT: u16 = evdev::Key::KEY_LEFTSHIFT.code();
    const KEY_RIGHTSHIFT: u16 = evdev::Key::KEY_RIGHTSHIFT.code();
    const KEY_S: u16 = evdev::Key::KEY_S.code();
    const KEY_D: u16 = evdev::Key::KEY_D.code();
    const KEY_ESC: u16 = evdev::Key::KEY_ESC.code();

    match key_code {
        // H key (left)
        KEY_H => {
            *h_pressed = pressed;
        }
        // J key (down)
        KEY_J => {
            *j_pressed = pressed;
        }
        // K key (up)
        KEY_K => {
            *k_pressed = pressed;
        }
        // L key (right)
        KEY_L => {
            *l_pressed = pressed;
        }
        // Left Shift
        KEY_LEFTSHIFT => {
            *shift_pressed = pressed;
        }
        // Right Shift
        KEY_RIGHTSHIFT => {
            *shift_pressed = pressed;
        }
        // S key with shift (left click)
        KEY_S => {
            if pressed && *shift_pressed {
                debug!("Left mouse button click");
                mouse.press(&uinput::event::controller::Controller::Mouse(Mouse::Left))?;
                mouse.synchronize()?;
                std::thread::sleep(Duration::from_millis(10));
                mouse.release(&uinput::event::controller::Controller::Mouse(Mouse::Left))?;
                mouse.synchronize()?;
            }
        }
        // D key with shift (right click)
        KEY_D => {
            if pressed && *shift_pressed {
                debug!("Right mouse button click");
                mouse.press(&uinput::event::controller::Controller::Mouse(Mouse::Right))?;
                mouse.synchronize()?;
                std::thread::sleep(Duration::from_millis(10));
                mouse.release(&uinput::event::controller::Controller::Mouse(Mouse::Right))?;
                mouse.synchronize()?;
            }
        }
        // ESC key to exit the application
        KEY_ESC => {
            if pressed {
                info!("ESC key pressed, exiting application");
                // Just exit the program
                std::process::exit(0);
            }
        }
        _ => {}
    }

    Ok(())
}

fn move_mouse(
    h_pressed: bool,
    j_pressed: bool,
    k_pressed: bool,
    l_pressed: bool,
    shift_pressed: bool,
    mouse: &mut uinput::Device,
    config: &Config,
) -> Result<()> {
    let mut x_movement = 0;
    let mut y_movement = 0;

    // Calculate horizontal movement
    if h_pressed {
        x_movement -= config.base_speed;
    }
    if l_pressed {
        x_movement += config.base_speed;
    }

    // Calculate vertical movement
    if k_pressed {
        y_movement -= config.base_speed;
    }
    if j_pressed {
        y_movement += config.base_speed;
    }

    // Apply shift multiplier if shift is pressed
    if shift_pressed {
        x_movement *= config.shift_multiplier;
        y_movement *= config.shift_multiplier;
    }

    // Send movement events if needed
    if x_movement != 0 {
        mouse.position(&uinput::event::relative::Position::X, x_movement)?;
    }
    if y_movement != 0 {
        mouse.position(&uinput::event::relative::Position::Y, y_movement)?;
    }

    // Synchronize events
    if x_movement != 0 || y_movement != 0 {
        mouse.synchronize()?;
    }

    Ok(())
}
