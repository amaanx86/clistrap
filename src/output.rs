use colored::Colorize;

pub fn success(msg: &str) {
    println!("{}", msg.green());
}

pub fn error(msg: &str) {
    eprintln!("{}: {}", "error".red().bold(), msg);
}

pub fn info(msg: &str) {
    println!("{}", msg.dimmed());
}

pub fn warn(msg: &str) {
    println!("{}: {}", "warn".yellow().bold(), msg);
}

pub fn header(msg: &str) {
    println!("\n{}\n", msg.bold());
}

pub fn kv(key: &str, value: &str) {
    println!("  {:<10}  {}", key, value);
}
