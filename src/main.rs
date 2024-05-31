/*
 * Copyright 2021-2022 Jochen Kupperschmidt
 * License: MIT
 */

use anyhow::Result;
use evdev::{Device, EventType, InputEventKind, Key};
use rodio::{OutputStream, Sink};
use rust_gpiozero::Button;
use std::process::exit;
mod audio;
mod cli;
mod config;

fn get_char(key: Key) -> Option<char> {
    match key {
        Key::KEY_1 => Some('1'),
        Key::KEY_2 => Some('2'),
        Key::KEY_3 => Some('3'),
        Key::KEY_4 => Some('4'),
        Key::KEY_5 => Some('5'),
        Key::KEY_6 => Some('6'),
        Key::KEY_7 => Some('7'),
        Key::KEY_8 => Some('8'),
        Key::KEY_9 => Some('9'),
        Key::KEY_0 => Some('0'),
        _ => None,
    }
}

fn main() -> Result<()> {
    println!("Programm gestartet...");
   
    let args = cli::parse_args();

    let config = config::load_config(&args.config_filename)?;

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    sink.sleep_until_end();

    let mut input_device = Device::open(&args.input_device)?;
    println!(
        "Opened input device \"{}\".",
        input_device.name().unwrap_or("unnamed device")
    );
    
    match input_device.grab() {
        Ok(_) => println!("Successfully obtained exclusive access to input device."),
        Err(error) => {
            eprintln!("Could not get exclusive access to input device: {}", error);
            exit(1);
        }
    }

    println!("now we should handle events!");

    let mut read_chars = String::new();
    let button_red = Button::new(11);
    let button_white = Button::new(8);
    let button_blue = Button::new(7);
    let VALUE_BUTTON_PRESSED = 20;
    let mut red_pressed_count = 0;
    let mut blue_pressed_count = 0;
    let mut white_pressed_count = 0;

    loop {        
        if button_red.is_active() {
            red_pressed_count += 1;
            println!("red plus 1");
        }
        if button_red.is_active() && red_pressed_count > VALUE_BUTTON_PRESSED {
            sink.set_volume(0.1);
            red_pressed_count = 0;
            println!("increased volume.");
        }
        if button_blue.is_active() {
            blue_pressed_count += 1;
        }
        if button_blue.is_active() && blue_pressed_count > VALUE_BUTTON_PRESSED {
            sink.set_volume(-0.1);
            blue_pressed_count = 0;
            println!("decreased volume.");
        }
        if button_white.is_active() {
            white_pressed_count += 1;
        }
        if button_white.is_active() && white_pressed_count > VALUE_BUTTON_PRESSED {
            sink.stop();
            white_pressed_count = 0;
            println!("stopped music");
        }
        for event in input_device.fetch_events()? {
           // println!("event value was \"{}\".",event.value().as_str());
            // Only handle pressed key events.
            if event.event_type() != EventType::KEY || event.value() == 1 {
                continue;
            }

            match event.kind() {
                InputEventKind::Key(Key::KEY_ENTER) => {
                    let input = read_chars.as_str();
                    println!("{}", input);
                    audio::play_sound(
                        &config.inputs_to_filenames,
                        input,
                        config.sounds_path.as_path(),
                        &sink,
                    )?;
                }
               InputEventKind::Key(key) => {
                    if let Some(ch) = get_char(key) {
                        read_chars.push(ch)
                    }
                }
                _ => (),
           }
        }
    }
}
