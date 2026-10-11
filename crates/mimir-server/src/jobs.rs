//! Background jobs (MIMIR-T-0724): long work (a catalog import) runs on a
//! blocking thread; the client polls `GET /jobs/{id}`. Kept in memory: a
//! restart forgets them (the work itself is in the database).

use std::collections::HashMap;
use std::sync::Mutex;

use mimir_wire::Job;
use serde_json::Value;

#[derive(Default)]
pub struct Jobs {
    map: Mutex<HashMap<String, Job>>,
}

/// A handle to report on one job.
#[derive(Clone)]
pub struct JobHandle {
    jobs: std::sync::Arc<Jobs>,
    pub id: String,
}

impl Jobs {
    pub fn start(self: &std::sync::Arc<Self>, kind: &str, message: &str) -> JobHandle {
        let id = uuid::Uuid::new_v4().to_string();
        let job = Job {
            id: id.clone(),
            kind: kind.into(),
            state: "running".into(),
            message: message.into(),
            result: Value::Null,
        };
        self.map.lock().expect("jobs").insert(id.clone(), job);
        JobHandle {
            jobs: self.clone(),
            id,
        }
    }

    pub fn get(&self, id: &str) -> Option<Job> {
        self.map.lock().expect("jobs").get(id).cloned()
    }
}

impl JobHandle {
    fn update(&self, f: impl FnOnce(&mut Job)) {
        if let Some(j) = self.jobs.map.lock().expect("jobs").get_mut(&self.id) {
            f(j);
        }
    }

    pub fn progress(&self, message: impl Into<String>) {
        let m = message.into();
        self.update(|j| j.message = m);
    }

    pub fn done(&self, message: impl Into<String>, result: Value) {
        let m = message.into();
        self.update(|j| {
            j.state = "done".into();
            j.message = m;
            j.result = result;
        });
    }

    pub fn failed(&self, message: impl Into<String>) {
        let m = message.into();
        self.update(|j| {
            j.state = "failed".into();
            j.message = m;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_job_runs_then_ends() {
        let jobs = std::sync::Arc::new(Jobs::default());
        let h = jobs.start("catalog", "Reading");
        assert_eq!(jobs.get(&h.id).unwrap().state, "running");
        h.progress("Halfway");
        assert_eq!(jobs.get(&h.id).unwrap().message, "Halfway");
        h.done("Done", serde_json::json!({"n": 1}));
        let j = jobs.get(&h.id).unwrap();
        assert_eq!(
            (j.state.as_str(), j.result["n"].as_i64()),
            ("done", Some(1))
        );
        assert!(jobs.get("nope").is_none());
    }
}
