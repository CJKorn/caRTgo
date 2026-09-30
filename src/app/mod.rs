pub mod cli;
pub mod menu;
pub mod orbit;
pub mod viewport;

use indicatif::{ProgressBar, ProgressStyle};

pub fn progress_bar(passes: u32) -> ProgressBar {
    ProgressBar::new(passes as u64).with_style(
        ProgressStyle::with_template("[{elapsed_precise}] {wide_bar} {pos}/{len} passes ({per_sec}, ETA {eta})")
            .unwrap(),
    )
}
