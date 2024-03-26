use std::sync::Arc;

use golem_worker_service_base::auth::{CommonNamespace, EmptyAuthCtx};

pub type WorkerService = Arc<
    dyn golem_worker_service_base::service::worker::WorkerService<EmptyAuthCtx, CommonNamespace>
        + Sync
        + Send,
>;
