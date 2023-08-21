use clap::{Parser, Subcommand};
use notify_rust::{Hint, Notification, Urgency};
use regex::Regex;
use std::process::Command;

const APPNAME: &str = "myButler";
const TAG: &str = "x-dunst-stack-tag";

/// Simple utility tool to notify audio volume modifications
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Adjuste volume
    Set {
        /// +/-VALUE, VALUE is in percent, volume will be adjusted relative to current sink volume
        percent: i32,
    },

    /// Toggle audio
    Toggle,
}

enum Volume {
    Muted,
    Percent(i32),
}

#[cfg(all(unix, not(target_os = "macos")))]
fn volume_indicator(volume: Volume) {
    let icon = match volume {
        Volume::Muted => "audio-volume-muted-symbolic",
        Volume::Percent(x) if x == 0 => "audio-volume-muted-symbolic",
        Volume::Percent(x) if x < 33 => "audio-volume-low-symbolic",
        Volume::Percent(x) if x < 67 => "audio-volume-medium-symbolic",
        _ => "audio-volume-high-symbolic",
    };

    let value = match volume {
        Volume::Muted => 0,
        Volume::Percent(p) => p,
    };

    Notification::new()
        .appname(APPNAME)
        .summary("Volume")
        .icon(icon)
        .urgency(Urgency::Low)
        .hint(Hint::Custom(TAG.to_owned(), APPNAME.to_owned()))
        .hint(Hint::CustomInt("value".to_owned(), value))
        .show()
        .unwrap();
}

fn get_volume() -> i32 {
    let re = Regex::new(r"[+-]?[0-9]+%").unwrap();

    let output = Command::new("/usr/bin/pactl")
        .args(["get-sink-volume", "@DEFAULT_SINK@"])
        .output()
        .expect("can't get volume");
    let output_str = String::from_utf8(output.stdout).unwrap();

    let current_volume_str: Vec<&str> = re.find(&output_str).unwrap().as_str().split('%').collect();
    
    current_volume_str[0].parse::<i32>().unwrap()
}

fn set_volume(percent: i32) {
    let current_volume = get_volume();

    let mut new_volume = current_volume + percent;
    if new_volume > 100 {
        new_volume = 100;
    }

    let new_volume_str = format!("{}%", new_volume);

    let _ = Command::new("/usr/bin/pactl")
        .args(["set-sink-volume", "@DEFAULT_SINK@", new_volume_str.as_str()])
        .status()
        .expect("set volume failed");

    if new_volume <= 0 {
        volume_indicator(Volume::Muted);
    } else {
        volume_indicator(Volume::Percent(new_volume));
    }
}

fn toggle_audio() {
    let _ = Command::new("/usr/bin/pactl")
        .args(["set-sink-mute", "@DEFAULT_SINK@", "toggle"])
        .status()
        .expect("toggle audio failed");

    let output = Command::new("/usr/bin/pactl")
        .args(["get-sink-mute", "@DEFAULT_SINK@"])
        .output()
        .expect("can't get audio mute status");
    let output_str = String::from_utf8(output.stdout).unwrap();

    if output_str.contains("no") {
        let current_volume = get_volume();

        volume_indicator(Volume::Percent(current_volume));
    } else {
        volume_indicator(Volume::Muted);
    }
}

#[cfg(any(windows, target_os = "macos"))]
fn main() {
    println!("this is an xdg only feature")
}

#[cfg(all(unix, not(target_os = "macos")))]
fn main() {
    let args = Args::parse();

    match &args.command {
        Commands::Set { percent } => set_volume(*percent),
        Commands::Toggle => toggle_audio(),
    };
}
