use ioevent::rpc::{DefaultProcedureWright, ProcedureCallWright};

pub trait SithraState: ProcedureCallWright + Clone + Send + Sync + 'static {
    fn self_id(&self) -> u64;
    fn create(self_id: u64) -> Self;
}

#[derive(Clone)]
pub struct CommonState {
    pub self_id: u64,
    pub pcw: DefaultProcedureWright,
}
impl SithraState for CommonState {
    fn self_id(&self) -> u64 {
        self.self_id
    }
    fn create(self_id: u64) -> Self {
        Self {
            self_id,
            pcw: DefaultProcedureWright::default(),
        }
    }
}
impl ProcedureCallWright for CommonState {
    fn next_echo(&self) -> impl Future<Output = u64> + Send {
        self.pcw.next_echo()
    }
}
