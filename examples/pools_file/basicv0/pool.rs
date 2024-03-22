use super::{
    models::{ComponentAddress, PoolAddress, PoolType},
    poolstore::Pool,
};

#[derive(Debug, Clone)]
pub struct BasicV0Pool {
    pub component_address: String,
    pub native_pool_address: Option<String>,
}

impl Pool for BasicV0Pool {
    fn component_address(&self) -> ComponentAddress {
        self.component_address.clone()
    }
    fn native_pool_address(&self) -> Option<PoolAddress> {
        self.native_pool_address.clone()
    }
    fn pool_type(&self) -> PoolType {
        PoolType::BasicV0
    }
    fn clone_box(&self) -> Box<dyn Pool> {
        Box::new(self.clone())
    }
}
