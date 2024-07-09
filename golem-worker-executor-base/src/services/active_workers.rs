// Copyright 2024 Golem Cloud
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{OwnedSemaphorePermit, Semaphore, TryAcquireError};
use tokio::time::timeout;
use tracing::{debug, Instrument};

use golem_common::cache::{BackgroundEvictionMode, Cache, FullCacheEvictionMode, SimpleCache};
use golem_common::model::{OwnedWorkerId, WorkerId};

use crate::error::GolemError;
use crate::services::golem_config::MemoryConfig;
use crate::services::HasAll;
use crate::worker::Worker;
use crate::workerctx::WorkerCtx;

/// Holds the metadata and wasmtime structures of currently active Golem workers
pub struct ActiveWorkers<Ctx: WorkerCtx> {
    workers: Cache<WorkerId, (), Arc<Worker<Ctx>>, GolemError>,
    worker_memory: Arc<Semaphore>,
    worker_estimate_coefficient: f64,
}

impl<Ctx: WorkerCtx> ActiveWorkers<Ctx> {
    pub fn new(memory_config: &MemoryConfig) -> Self {
        let worker_memory_size = memory_config.worker_memory();
        Self {
            workers: Cache::new(
                None,
                FullCacheEvictionMode::None,
                BackgroundEvictionMode::None,
                "active_workers",
            ),
            worker_memory: Arc::new(Semaphore::new(worker_memory_size)),
            worker_estimate_coefficient: memory_config.worker_estimate_coefficient,
        }
    }

    pub async fn get_or_add<T>(
        &self,
        deps: &T,
        owned_worker_id: &OwnedWorkerId,
        worker_args: Option<Vec<String>>,
        worker_env: Option<Vec<(String, String)>>,
        component_version: Option<u64>,
        parent: Option<WorkerId>,
    ) -> Result<Arc<Worker<Ctx>>, GolemError>
    where
        T: HasAll<Ctx> + Clone + Send + Sync + 'static,
    {
        let worker_id = owned_worker_id.worker_id();

        let owned_worker_id = owned_worker_id.clone();
        let deps = deps.clone();
        let worker_estimate_coefficient = self.worker_estimate_coefficient;
        self.workers
            .get_or_insert_simple(&worker_id, || {
                Box::pin(async move {
                    Ok(Arc::new(
                        Worker::new(
                            &deps,
                            owned_worker_id,
                            worker_args,
                            worker_env,
                            component_version,
                            parent,
                            worker_estimate_coefficient,
                        )
                        .in_current_span()
                        .await?,
                    ))
                })
            })
            .await
    }

    pub fn remove(&self, worker_id: &WorkerId) {
        self.workers.remove(worker_id);
    }

    pub fn iter(&self) -> impl Iterator<Item = (WorkerId, Arc<Worker<Ctx>>)> + '_ {
        self.workers.iter()
    }

    pub async fn acquire(&self, memory: u64) -> OwnedSemaphorePermit {
        let mem32: u32 = memory
            .try_into()
            .expect("requested memory size is too large");

        loop {
            match self.worker_memory.clone().try_acquire_many_owned(mem32) {
                Ok(permit) => {
                    debug!(
                        "Acquired {} memory of {}",
                        mem32,
                        self.worker_memory.available_permits()
                    );
                    break permit;
                }
                Err(TryAcquireError::Closed) => panic!("worker memory semaphore has been closed"),
                Err(TryAcquireError::NoPermits) => {
                    debug!("Not enough memory, trying to free some up");
                    if self.try_free_up_memory(memory).await {
                        debug!("Freed up some memory, retrying");
                        // We have enough memory unless another worker has taken it in the meantime,
                        // so retry the loop
                        continue;
                    } else {
                        debug!("Could not free up memory, waiting for permits to be available");
                        // Could not free up enough memory, so waiting for permits to be available.
                        // We cannot wait forever, because we need to force idle workers out of memory
                        // if there is a need - but don't want to early drop them if there's no need.
                        match timeout(
                            Duration::from_millis(500),
                            self.worker_memory.clone().acquire_many_owned(mem32),
                        )
                        .await
                        .expect("worker memory semaphore has been closed")
                        {
                            Ok(permit) => break permit,
                            Err(_) => {
                                debug!("Could not acquire memory in time, retrying");
                                continue;
                            }
                        }
                    }
                }
            }
        }
    }

    async fn try_free_up_memory(&self, memory: u64) -> bool {
        let current_avail = self.worker_memory.available_permits();
        let needed = memory.saturating_sub(current_avail as u64);

        if needed > 0 {
            let mut possibilities = Vec::new();

            debug!("Collecting possibilities");
            // Collecting the workers which are currently idle but loaded into memory
            for (worker_id, worker) in self.workers.iter() {
                if worker.is_currently_idle_but_running() {
                    if let Ok(mem) = worker.memory_requirement().await {
                        let last_changed = worker.last_execution_state_change().await;
                        possibilities.push((worker_id, worker, mem, last_changed));
                    }
                }
            }

            // Sorting them by last time they changed their status - newest first
            possibilities
                .sort_by_key(|(_worker_id, _worker, _mem, last_changed)| last_changed.to_millis());
            possibilities.reverse();

            let mut freed = 0;

            // Dropping the oldest ones until we have enough memory available - rechecking the idle status before
            while freed < needed && !possibilities.is_empty() {
                let (worker_id, worker, mem, _) = possibilities.pop().unwrap();

                debug!("Trying to stop {worker_id} to free up memory");
                if worker.stop_if_idle().await {
                    debug!("Stopped {worker_id} to free up {mem} memory");
                    freed += mem;
                }
            }

            debug!("Freed up {freed}");
            freed >= needed
        } else {
            debug!("Memory was freed up in the meantime");
            // Memory was freed up in the meantime, we can retry
            true
        }
    }
}
