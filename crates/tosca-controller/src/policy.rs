use std::collections::HashMap;

use tosca::hazards::Hazards;

// TODO: Eventually rewrite policy IDs as &'static str.

/// A privacy policy manager.
///
/// It allows or blocks the requests to devices, or to a specific device,
/// according to a set of privacy rules.
#[derive(Debug, PartialEq)]
pub struct Policy {
    block_on_hazards: Hazards,
    block_device_on_hazards: HashMap<usize, Hazards>,
}

impl Policy {
    /// Creates a [`Policy`] to block **all** requests that have the given
    /// [`Hazards`] in their routes.
    #[must_use]
    #[inline]
    pub fn new(block_on_hazards: Hazards) -> Self {
        Self {
            block_on_hazards,
            block_device_on_hazards: HashMap::new(),
        }
    }

    /// Creates a [`Policy`] to block **all** [`crate::device::Device`] id
    /// requests that have the given [`Hazards`] in their routes.
    #[must_use]
    #[inline]
    pub fn only_local_policy(id: usize, hazards: Hazards) -> Self {
        let policy = Self::init();
        policy.block_device_on_hazards(id, hazards)
    }

    /// Adds a new [`Policy`] to block **all** [`crate::device::Device`] id
    /// requests that have the given [`Hazards`] in their routes.
    #[must_use]
    #[inline]
    pub fn block_device_on_hazards(mut self, id: usize, hazards: Hazards) -> Self {
        self.block_device_on_hazards.insert(id, hazards);
        self
    }

    pub(crate) fn init() -> Self {
        Self {
            block_on_hazards: Hazards::new(),
            block_device_on_hazards: HashMap::new(),
        }
    }

    pub(crate) fn global_blocked_hazards(&self, hazards: &Hazards) -> Hazards {
        let mut blocked_hazards = Hazards::new();
        for hazard in hazards {
            if self.block_on_hazards.contains(hazard) {
                blocked_hazards.add(*hazard);
            }
        }
        blocked_hazards
    }

    pub(crate) fn local_blocked_hazards(&self, id: usize, hazards: &Hazards) -> Hazards {
        if let Some(local_hazards) = self.block_device_on_hazards.get(&id) {
            let mut blocked_hazards = Hazards::new();
            for hazard in hazards {
                if local_hazards.contains(hazard) {
                    blocked_hazards.add(*hazard);
                }
            }
            blocked_hazards
        } else {
            Hazards::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tosca::hazards::{Hazard, Hazards};

    use super::Policy;

    fn create_policy() -> (Hazards, Policy) {
        let hazards = Hazards::new().insert(Hazard::ElectricEnergyConsumption);

        let policy = Policy::new(hazards.clone());

        (hazards, policy)
    }

    fn check_device_policies(policy: &Policy, block_on_hazards: Hazards, local_hazards: &Hazards) {
        let mut devices_hazards = HashMap::new();
        devices_hazards.insert(1, local_hazards.clone());
        devices_hazards.insert(2, local_hazards.clone());

        assert_eq!(
            policy,
            &Policy {
                block_on_hazards,
                block_device_on_hazards: devices_hazards,
            }
        );
    }

    #[test]
    fn only_global_policy() {
        let (hazards, policy) = create_policy();

        assert_eq!(
            policy,
            Policy {
                block_on_hazards: hazards,
                block_device_on_hazards: HashMap::new()
            }
        );
    }

    #[test]
    fn only_local_policy() {
        let local_hazards = Hazards::new().insert(Hazard::Explosion);

        let policy = Policy::only_local_policy(1, local_hazards.clone())
            .block_device_on_hazards(2, local_hazards.clone());

        check_device_policies(&policy, Hazards::new(), &local_hazards);
    }

    #[test]
    fn complete_policy() {
        let (global_hazards, policy) = create_policy();

        let local_hazards = Hazards::new().insert(Hazard::Explosion);

        let policy = policy
            .block_device_on_hazards(1, local_hazards.clone())
            .block_device_on_hazards(2, local_hazards.clone());

        check_device_policies(&policy, global_hazards, &local_hazards);
    }
}
