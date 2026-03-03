A logger that prints [log](https://crates.io/crates/log) messages to configured output channels on the Wii U. It is the Rust alternative to the [WHBLog](https://github.com/devkitPro/wut/blob/master/libraries/libwhb/include/whb/log.h) functions in [wut](https://github.com/devkitPro/wut).

# Usage

```
use cafe_logger::CafeLogger;
fn main() {
    CafeLogger::new().init().unwrap();

    log::warn!("This is an example message.");
}
```
