use clap::{Args, Parser};
use dbus::blocking::Connection;
use dbus_crossroads::Crossroads;
use notify_rust::Notification;
use std::{thread, time::Duration};

/// Pomodoro application that can either start a timer or initiate a server for managing timers.
#[derive(Parser)]
#[command(version = "1.0", author = "Author Name <author@example.com>")]
struct Cli {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    Server(Server),
    Start(Start),
}

/// Starts the Pomodoro server that listens to D-Bus and handles timer requests.
#[derive(Args)]
struct Server {}

/// Sends a command to the Pomodoro server via D-Bus to start a timer.
#[derive(Args)]
struct Start {
    /// Sets a custom time for the Pomodoro in minutes.
    #[arg(short, long, default_value = "25")]
    time: u64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.subcmd {
        SubCommand::Server(_) => {
            Ok(run_server()?)
        }
        SubCommand::Start(args) => {
            Ok(start_timer(args.time)?)
        }
    }
}

fn send_notification() -> Result<(), notify_rust::error::Error>{
    Notification::new()
        .summary("Pomodoro Timer")
        .body("Time's up! Take a break.")
        .show()?;
    Ok(())
}

fn timer(minutes: u64) -> Result<(), Box<dyn std::error::Error>> {
    let duration = Duration::new(minutes * 60, 0);
    println!("Pomodoro started for {} minutes...", minutes);
    thread::sleep(duration);
    Ok(send_notification()?)
}

fn run_server() -> Result<(), dbus::Error> {
    // The logic for the D-Bus server remains the same as in your previous `server.rs`.
    let connection = Connection::new_session()?;
    connection.request_name("com.example.Pomodoro", false, true, false)?;

    let mut cr = Crossroads::new();

    let iface_token = cr.register("com.example.Pomodoro", |b| {
        b.method("Start", ("minutes",), (), move |_, _, (minutes,): (u64,)| {
            thread::spawn(move || {
                timer(minutes).expect("Failed to start timer");
            });
            Ok(())
        });
    });

    cr.insert("/com/example/Pomodoro/Timers", &[iface_token], ());
    cr.serve(&connection)
}

fn start_timer(minutes: u64) -> Result<(), dbus::Error> {
    let interface = "com.example.Pomodoro";
    let path = "/com/example/Pomodoro/Timers";
    // The logic for the D-Bus client remains the same as in your previous `client.rs`.
    let connection = Connection::new_session()?;
    let proxy = connection.with_proxy(
        "com.example.Pomodoro",
        "/com/example/Pomodoro/Timers",
        Duration::from_millis(5000),
    );

    proxy.method_call("com.example.Pomodoro", "Start", (minutes,))
}
