use chrono::{DateTime, Utc};
use dyn_clone::DynClone;

#[async_trait::async_trait]
pub trait SystemClock: Send + Sync + DynClone{
    fn now(&self) -> DateTime<Utc>;
}

dyn_clone::clone_trait_object!(SystemClock);

#[derive(Clone)]
pub struct RealSystemClock;

#[async_trait::async_trait]
impl SystemClock for RealSystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}