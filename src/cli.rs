use clap::{command, Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Serve the application
    Serve(ServeArgs),
}

#[derive(Args, Debug, Clone)]
pub struct ServeArgs {
    /// Server bind host
    #[clap(long, env, default_value = "0.0.0.0:8080")]
    pub listen: String,

    /// Backend server address
    #[clap(long, env, default_value = "127.0.0.1:8081")]
    pub backend: String,

    /// Buffer size
    #[clap(long, env, default_value = "32768")]
    pub buf_size: usize,

    /// MySQL connection URI
    #[clap(short, long, env)]
    pub mysql_uri: String,

    /// Redis connection URI
    #[clap(short, long, env)]
    pub redis_uri: String,

    /// Lua script path
    #[clap(short, long, env, default_value = "app.lua")]
    pub script: String,

    /// Lua script entry function
    #[clap(long, env, default_value = "handle")]
    pub script_entry: String,
}
