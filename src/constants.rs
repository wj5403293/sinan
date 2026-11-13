cfg_select! {
    target_os = "android" => {
        pub const APP_ROOT: &str = "/storage/emulated/0/司南";
    }
    _ => {
        pub const APP_ROOT: &str = "./司南";
    }
}

pub const DIR_OUT: &str = "out";
pub const DIR_TMP: &str = "tmp";
pub const DIR_SCRIPTS: &str = "scripts";

pub const VERSION: &str = "v2.2026.11.13.8";
