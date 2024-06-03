/*
 * Copyright 2021-2022 Jochen Kupperschmidt
 * License: MIT
 */

use anyhow::Result;
use evdev::{Device, EventType, InputEventKind, Key};
use player::Player;
use rppal::gpio::Gpio;
use std::{
    process::exit,
    sync::{Arc, Mutex},
    thread, time,
};
mod cli;
mod config;
mod player;

const GPIO_RED: u8 = 2;
const GPIO_BLUE: u8 = 4;

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

    let btn_thread_player = Arc::new(Mutex::new(Player::new(config)));
    let nfc_thread_player = btn_thread_player.clone();

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
    let button_blue = gpio.get(GPIO_BLUE)?.into_input_pullup();
    let debounce_time = time::Duration::from_millis(500);

    let button_handler = thread::spawn(move || loop {
        if button_red.is_low() {
            println!("vol up");
            btn_thread_player.lock().unwrap().set_volume(1.5);
            thread::sleep(debounce_time);
        }
        if button_blue.is_low() {
            println!("vol down");
            btn_thread_player.lock().unwrap().set_volume(0.5);
            thread::sleep(debounce_time);
        }
    });

    let nfc_handler = thread::spawn(move || loop {
        for event in input_device.fetch_events().unwrap() {
            if event.event_type() != EventType::KEY || event.value() == 1 {
                continue;
            }

            match event.kind() {
                InputEventKind::Key(Key::KEY_ENTER) => {
                    let input = read_chars.as_str();
                    nfc_thread_player.lock().unwrap().stop();
                    nfc_thread_player.lock().unwrap().play_song_by_id(input);
                }
                InputEventKind::Key(key) => {
                    if let Some(ch) = get_char(key) {
                        read_chars.push(ch);
                    }
                }
                _ => (),
            }
        }
    });

    button_handler.join().unwrap();
    nfc_handler.join().unwrap();
    Ok(())
}
