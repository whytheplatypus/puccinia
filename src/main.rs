use clap::{Args, Parser};
use dbus::blocking::Connection;
use dbus_crossroads::Crossroads;
use notify_rust::{Notification, NotificationHandle};
use std::{thread, time::Duration};

const DBUS_NAME: &str = "com.example.Pomodoro";
const DBUS_PATH: &str = "/com/example/Pomodoro/Timers";

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
        SubCommand::Server(_) => Ok(run_server()?),
        SubCommand::Start(args) => Ok(start_timer(args.time)?),
    }
}

struct Pomodoro {
    notification: Notification,
    duration: u64,
}

fn timer(minutes: u64, notification: &Notification) -> Result<(), Box<dyn std::error::Error>> {
    let duration = Duration::new(minutes * 60, 0);
    println!("Pomodoro started for {} minutes...", minutes);
    let mut handle = notification.show()?;
    let mut duration = 0;
    while duration < minutes {
        println!("{} minutes left...", minutes - duration);
        thread::sleep(Duration::new(60, 0));
        duration += 1;
        handle.body(&format!("{} minutes left...", minutes - duration));
        handle.update();
    }
    Ok(())
}

fn run_server() -> Result<(), dbus::Error> {
    let connection = Connection::new_session()?;
    connection.request_name(DBUS_NAME, false, true, false)?;

    let mut cr = Crossroads::new();

    let iface_token = cr.register(DBUS_NAME, |b| {
        b.method(
            "Start",
            ("minutes",),
            (),
            move |_, _, (minutes,): (u64,)| {
                let break_over_notification = Notification::new()
                    .summary("Work Time Over")
                    .body("Time for a break!")
                    .finalize();

                let work_time_over_notification = Notification::new()
                    .summary("Break Over")
                    .body("Time to get back to work!")
                    .finalize();

                let work_pomodoro = Pomodoro {
                    notification: work_time_over_notification,
                    duration: minutes,
                };

                let break_pomodoro = Pomodoro {
                    notification: break_over_notification,
                    duration: 2,
                };

                thread::spawn(move || {
                    let pomodoros = [work_pomodoro, break_pomodoro];
                    for pomodoro in pomodoros.iter().cycle() {
                        timer(pomodoro.duration, &pomodoro.notification)
                            .expect("Failed to start timer");
                    }
                });
                Ok(())
            },
        );
    });

    cr.insert(DBUS_PATH, &[iface_token], ());
    cr.serve(&connection)
}

fn start_timer(minutes: u64) -> Result<(), dbus::Error> {
    let connection = Connection::new_session()?;
    let proxy = connection.with_proxy(DBUS_NAME, DBUS_PATH, Duration::from_millis(5000));

    proxy.method_call(DBUS_NAME, "Start", (minutes,))
}
