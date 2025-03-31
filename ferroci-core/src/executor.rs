use std::collections::{HashMap, HashSet};
use std::process::{Command, ExitStatus};
use std::sync::{Arc, Mutex};
use std::thread;
use crate::config::{Pipeline, Job};

pub fn run_pipeline(pipeline: &Pipeline) {
    let completed_jobs = Arc::new(Mutex::new(HashSet::new()));
    let job_queue = Arc::new(Mutex::new(pipeline.jobs.clone()));

    let mut handles = vec![];

    while completed_jobs.lock().unwrap().len() < pipeline.jobs.len() {
        let mut progress = false;

        {
            let mut job_queue_lock = job_queue.lock().unwrap();
            let mut completed_jobs_lock = completed_jobs.lock().unwrap();

            let runnable_jobs: Vec<_> = job_queue_lock
                .iter()
                .filter(|(job_name, job)| {
                    !completed_jobs_lock.contains(job_name.as_str())
                        && job
                        .needs
                        .as_ref()
                        .map_or(true, |deps| deps.iter().all(|dep| completed_jobs_lock.contains(dep)))
                })
                .map(|(name, job)| (name.clone(), job.clone()))
                .collect();

            for (job_name, job) in runnable_jobs {
                let completed_jobs_clone = Arc::clone(&completed_jobs);
                let job_queue_clone = Arc::clone(&job_queue);

                println!("▶ Running job: {}", job_name);

                let handle = thread::spawn(move || {
                    if run_job(&job) {
                        let mut completed_jobs_lock = completed_jobs_clone.lock().unwrap();
                        completed_jobs_lock.insert(job_name.clone());

                        let mut job_queue_lock = job_queue_clone.lock().unwrap();
                        job_queue_lock.remove(&job_name);
                    } else {
                        eprintln!("❌ Job '{}' failed. Stopping execution.", job_name);
                        std::process::exit(1);
                    }
                });

                handles.push(handle);
                progress = true;
            }
        }

        if !progress {
            eprintln!("❌ Dependency loop detected. Exiting.");
            break;
        }

        thread::sleep(std::time::Duration::from_millis(100)); // Avoid CPU overuse
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

fn run_job(job: &Job) -> bool {
    for step in &job.steps {
        match run_command(step) {
            Ok(status) if status.success() => {
                println!("✅ {} - Success", step);
            }
            Ok(status) => {
                println!("⚠️ {} - Failed with exit code: {}", step, status);
                return false;
            }
            Err(e) => {
                eprintln!("❌ Error executing '{}': {}", step, e);
                return false;
            }
        }
    }
    true
}

fn run_command(command: &str) -> Result<ExitStatus, Box<dyn std::error::Error>> {
    let mut parts = command.split_whitespace();
    let program = parts.next().ok_or("Empty command")?;
    let args: Vec<&str> = parts.collect();

    let status = Command::new(program)
        .args(args)
        .status();

    match status {
        Ok(s) => Ok(s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("❌ Command '{}' not found!", command);
            Err(Box::new(e))
        }
        Err(e) => Err(Box::new(e)),
    }
}
