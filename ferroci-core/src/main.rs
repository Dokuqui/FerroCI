mod config;
mod executor;

use config::Pipeline;
use executor::run_pipeline;

fn main() {
    env_logger::init();

    match Pipeline::from_file("pipeline.toml") {
        Ok(pipeline) => run_pipeline(&pipeline),
        Err(e) => eprintln!("❌ Error loading pipeline: {}", e),
    }
}
