use std::collections::{HashMap, HashSet};
use std::process::{Command, ExitStatus};
use std::sync::{Arc, Mutex};
use std::thread;
use log::{info, error, warn};
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
                        && job.depends_on.as_ref().map_or(true, |deps| {
                        deps.iter().all(|dep| completed_jobs_lock.contains(dep))
                    })
                })
                .map(|(name, job)| (name.clone(), job.clone()))
                .collect();

            for (job_name, job) in runnable_jobs {
                let completed_jobs_clone = Arc::clone(&completed_jobs);
                let job_queue_clone = Arc::clone(&job_queue);

                info!("▶ Running job: {}", job_name);

                let handle = thread::spawn(move || {
                    if run_job(&job) {
                        let mut completed_jobs_lock = completed_jobs_clone.lock().unwrap();
                        completed_jobs_lock.insert(job_name.clone());

                        let mut job_queue_lock = job_queue_clone.lock().unwrap();
                        job_queue_lock.remove(&job_name);
                    } else {
                        let mut completed_jobs_lock = completed_jobs_clone.lock().unwrap();
                        completed_jobs_lock.insert(job_name.clone());

                        error!("❌ Job '{}' failed. Stopping execution.", job_name);
                        std::process::exit(1);
                    }
                });

                handles.push(handle);
                progress = true;
            }
        }

        if !progress {
            error!("❌ No progress detected. Possible dependency loop.");
            break;
        }

        thread::sleep(std::time::Duration::from_millis(100));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

fn run_job(job: &Job) -> bool {
    for step in &job.steps {
        match run_command(step) {
            Ok(status) if status.success() => {
                info!("✅ {} - Success", step);
            }
            Ok(status) => {
                warn!("⚠️ {} - Failed with exit code: {}", step, status);
                return false;
            }
            Err(e) => {
                error!("❌ Error executing '{}': {}", step, e);
                return false;
            }
        }
    }
    true
}

fn run_command(command: &str) -> Result<ExitStatus, Box<dyn std::error::Error>> {
    let (shell, shell_arg) = if cfg!(windows) {
        ("cmd.exe", "/C")
    } else {
        ("sh", "-c")
    };

    let status = Command::new(shell)
        .arg(shell_arg)
        .arg(command)
        .status()
        .map_err(|e| format!("❌ Failed to execute command '{}': {}", command, e))?;

    if status.success() {
        Ok(status)
    } else {
        error!("❌ Command '{}' failed with exit code: {:?}", command, status.code());
        Ok(status)
    }
}