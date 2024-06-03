/*
 * Copyright 2021-2022 Jochen Kupperschmidt
 * License: MIT
 */

use anyhow::Result;
use evdev::{Device, EventType, InputEventKind, Key};
use rodio::{OutputStream, Sink};
use std::{process::exit, sync::Arc, thread, time};
use rppal::gpio::Gpio;
mod audio;
mod cli;
mod config;

const GPIO_RED: u8 = 2;
const GPIO_BLUE: u8 = 4;
//const GPIO_WHITE: u8 = 4;

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
    let sink = Arc::new(Sink::try_new(&stream_handle).unwrap());
    let nfc_sinc = sink.clone();
    
    //sink.sleep_until_end();

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

    let mut read_chars = String::new();
    let gpio = Gpio::new().unwrap();
    let button_red = gpio.get(GPIO_RED)?.into_input_pullup();
    //let button_white = gpio.get(GPIO_WHITE)?.into_input_pullup();
    let button_blue = gpio.get(GPIO_BLUE)?.into_input_pullup();
    let debounce_time = time::Duration::from_millis(500);
    
    let button_handler = thread::spawn(move || loop {
        if button_red.is_low(){
            println!("vol up");
            sink.set_volume(1.5);
            thread::sleep(debounce_time);
        }
        if button_blue.is_low(){
            println!("vol down");
            sink.set_volume(0.5);
            thread::sleep(debounce_time);
        }
        //if button_white.is_low(){
        //    println!("stop playing");
        //    sink.clear();
        //    thread::sleep(debounce_time);
        //}    
     });


    let nfc_handler = thread::spawn(move || {
        loop {
            for event in input_device.fetch_events().unwrap() {
                // println!("event value was \"{}\".",event.value());
                // Only handle pressed key events.
                if event.event_type() != EventType::KEY || event.value() == 1 {
                    continue;
                }

                match event.kind() {
                    InputEventKind::Key(Key::KEY_ENTER) => {
                        let input = read_chars.as_str();
                        audio::play_sound(
                            &config.inputs_to_filenames,
                            input,
                            config.sounds_path.as_path(),
                            &nfc_sinc,
                        )
                        .unwrap();
                    }
                    InputEventKind::Key(key) => {
                        if let Some(ch) = get_char(key) {
                            read_chars.push(ch);
                        }
                    }
                    _ => (),
                }
            }
        }
    });

    button_handler.join().unwrap();
    nfc_handler.join().unwrap();
    Ok(())
}
