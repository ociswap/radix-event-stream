use scrypto::network::NetworkDefinition;

use std::collections::HashMap;
use std::fmt::Debug;

use super::models::{ComponentAddress, PoolAddress, PoolType};

#[derive(Debug)]
pub struct PoolStore {
    pub pools: HashMap<ComponentAddress, Box<dyn Pool>>,
    pub native_address_map: HashMap<PoolAddress, ComponentAddress>,
    pub package_addresses: HashMap<String, PoolType>,
    pub network: NetworkDefinition,
}

pub trait Pool: Debug + Send + Sync {
    fn component_address(&self) -> ComponentAddress;
    fn native_pool_address(&self) -> Option<PoolAddress>;
    fn pool_type(&self) -> PoolType;
    fn clone_box(&self) -> Box<dyn Pool>;
}

impl PoolStore {
    pub fn new(network: NetworkDefinition) -> Self {
        PoolStore {
            pools: HashMap::new(),
            native_address_map: HashMap::new(),
            package_addresses: HashMap::new(),
            network,
        }
    }

    /// Add a pool to the store
    pub fn add_pool(&mut self, pool: &Box<dyn Pool>) {
        self.pools
            .insert(pool.component_address(), pool.clone_box());
        match pool.native_pool_address() {
            Some(address) => {
                self.native_address_map
                    .insert(address, pool.component_address());
            }
            None => {}
        }
    }

    /// Find a pool by either its component address or its native pool address
    pub fn find_pool(&self, address: &String) -> Option<&Box<dyn Pool>> {
        match self.pools.get(address) {
            Some(pool) => Some(pool),
            None => match self.native_address_map.get(address) {
                Some(component_address) => self.pools.get(component_address),
                None => None,
            },
        }
    }

    pub fn find_by_component_address(
        &self,
        address: &String,
    ) -> Option<&Box<dyn Pool>> {
        self.pools.get(address)
    }

    pub fn find_by_native_address(
        &self,
        address: &String,
    ) -> Option<&Box<dyn Pool>> {
        match self.native_address_map.get(address) {
            Some(component_address) => self.pools.get(component_address),
            None => None,
        }
    }

    pub fn add_package_address(
        &mut self,
        package_address: &str,
        pool_type: PoolType,
    ) {
        self.package_addresses
            .insert(package_address.to_string(), pool_type);
    }

    pub fn find_by_package_address(
        &self,
        package_address: &str,
    ) -> Option<PoolType> {
        self.package_addresses.get(package_address).copied()
    }
}
