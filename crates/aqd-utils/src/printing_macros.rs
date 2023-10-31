// SPDX-License-Identifier: Apache-2.0

/// macro to print a title (cyan and bold)
#[macro_export]
macro_rules! print_title {
    ($title:expr) => {
        println!("{}", format!("\n{}", $title.bold().cyan(),));
    };
}

/// macro to print a subtitle (cyan and bold) indented with 2 spaces
#[macro_export]
macro_rules! print_subtitle {
    ($title:expr) => {
        println!("{}", format!("\n  {}", $title.bold().cyan(),));
    };
}

/// macro to print a key and value (green and bold) indented with 4 spaces
#[macro_export]
macro_rules! print_key_value {
    ($key:expr, $value:expr) => {
        println!("    {}: {}", format!("{:<15}", $key.bold().green()), $value);
    };
}

/// macro to print a value (indented with 4 spaces)
#[macro_export]
macro_rules! print_value {
    ($val:expr) => {
        println!("    {}", $val);
    };
}

/// macro to print a warning (yellow and bold)
#[macro_export]
macro_rules! print_warning {
    ($warning:expr) => {
        println!(
            "{}",
            format!("\n{} {}", "Warning:".bold().yellow(), $warning.yellow())
        );
    };
}
