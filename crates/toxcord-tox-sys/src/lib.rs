#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::all)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tox_version() {
        unsafe {
            let major = tox_version_major();
            let minor = tox_version_minor();
            let patch = tox_version_patch();
            println!("c-toxcore version: {major}.{minor}.{patch}");
            assert!(major > 0 || minor > 0);
        }
    }
}
