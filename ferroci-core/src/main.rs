mod config;
mod executor;

use config::Pipeline;
use executor::run_pipeline;

fn main() {
    match Pipeline::from_file("pipeline.toml") {
        Ok(pipeline) => run_pipeline(&pipeline),
        Err(e) => eprintln!("❌ Error loading pipeline: {}", e),
    }
}
