#![feature(env, core, path)]
extern crate gcc;

fn main() {
    fn parse_env(key: &str, default: bool) -> bool {
        use std::env::{var_string, VarError};

        match var_string(key) {
            Ok(val) => {
                match val.as_slice() {
                    "0" => false,
                    "1" => true,
                    _ => default
                }
            },
            Err(VarError::NotPresent) => default,
            Err(VarError::NotUnicode(_)) => panic!("Environment variable is not unicode: {}", key),
        }
    }

    let linear_interpolation = parse_env("XM_LINEAR_INTERPOLATION", true);
    let ramping = parse_env("XM_RAMPING", true);
    let debug = parse_env("XM_DEBUG", false);
    let big_endian = parse_env("XM_BIG_ENDIAN", false);

    fn on_off(value: bool) -> Option<&'static str> {
        Some(if value { "1" } else { "0" })
    }

    gcc::Config::new()
        .file("libxm/src/context.c")
        .file("libxm/src/load.c")
        .file("libxm/src/play.c")
        .file("libxm/src/xm.c")
        .include(Path::new("libxm/include"))
        .define("XM_LINEAR_INTERPOLATION", on_off(linear_interpolation))
        .define("XM_RAMPING", on_off(ramping))
        .define("XM_DEBUG", on_off(debug))
        .define("XM_BIG_ENDIAN", on_off(big_endian))
        .flag("--std=c11")
        .compile("libxm.a");
}
