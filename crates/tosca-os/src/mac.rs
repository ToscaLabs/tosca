#[cfg(target_os = "linux")]
mod os_mac {
    use std::fs;
    use std::path::Path;

    use tracing::warn;

    const IFACE_TYPE_ETHERNET: u16 = 1;
    const IFACE_TYPE_WIFI: u16 = 801;

    // Known MAC OUIs for common virtual machine vendors.
    // Source: https://standards-oui.ieee.org/oui.txt and common known VM vendors.
    const VM_MAC_PREFIXES: &[[u8; 3]] = &[
        [0x00, 0x05, 0x69], // VMware
        [0x00, 0x0C, 0x29], // VMware
        [0x00, 0x1C, 0x14], // VMware
        [0x00, 0x50, 0x56], // VMware
        [0x00, 0x03, 0xFF], // Microsoft Hyper-V
        [0x00, 0x15, 0x5D], // Microsoft Hyper-V
        [0x08, 0x00, 0x27], // Oracle VirtualBox
        [0x0A, 0x00, 0x27], // Oracle VirtualBox
        [0x00, 0x1C, 0x42], // Parallels
    ];

    fn is_locally_administered_mac(mac: [u8; 6]) -> bool {
        (mac[0] & 0x02) != 0
    }

    fn is_virtual_mac_vendor(mac: [u8; 6]) -> bool {
        VM_MAC_PREFIXES.iter().any(|prefix| prefix == &mac[0..3])
    }

    fn is_virtual_interface(iface_path: &Path, mac: [u8; 6]) -> bool {
        // If the interface does not have a "device" entry,
        // it is considered virtual.
        if !iface_path.join("device").exists() {
            return true;
        }

        // Checks if the MAC address is locally administered
        // (bit 1 of the first byte is set).
        // A locally administered address is one assigned by software rather
        // than by the hardware manufacturer, and is typically used in
        // virtual machines, containers, or custom network configurations.
        if is_locally_administered_mac(mac) {
            return true;
        }

        // Checks if the MAC address is from a known virtual machine vendor
        // based on MAC OUI prefix.
        // Returns true if the MAC prefix matches known virtual adapters:
        // VMware, Hyper-V, VirtualBox, etc.
        if is_virtual_mac_vendor(mac) {
            return true;
        }

        // Canonical path checks for virtual devices or hypervisor interfaces.
        if let Ok(canon) = fs::canonicalize(iface_path)
            && let Some(s) = canon.to_str()
        {
            // Virtual devices path.
            if s.contains("/sys/devices/virtual/") {
                return true;
            }

            // Hyper-V virtual interface.
            if s.contains("VMBUS") {
                return true;
            }

            // Other hypervisors or virtualization platforms can be added here
            // if needed.
            // This check looks for specific path fragments that identify
            // virtual interfaces.
            // If a new platform uses a different path structure,
            // its identifying strings can be added here to ensure
            // those interfaces are correctly recognized as virtual.
        }

        if let Some(name) = iface_path.file_name().and_then(|n| n.to_str()) {
            // Exclude loopback interface "lo".
            if name == "lo" {
                return true;
            }

            // Exclude common container and virtual network interface prefixes.
            let prefixes = ["docker", "br-", "veth", "tun", "tap", "vmnet"];
            if prefixes.iter().any(|p| name.starts_with(p)) {
                return true;
            }
        }

        false
    }

    fn read_mac(iface_path: &Path) -> Option<[u8; 6]> {
        // The MAC address is stored in the "address" file of
        // the network interface.
        let mac_str = std::fs::read_to_string(iface_path.join("address")).ok()?;
        let mac_str = mac_str.trim();

        let mut mac = [0u8; 6];
        let mut parts = mac_str.split(':');

        for byte in &mut mac {
            let part = parts.next()?;
            *byte = u8::from_str_radix(part, 16).ok()?;
        }

        if parts.next().is_some() {
            return None;
        }

        Some(mac)
    }

    fn get_interface_type(iface_path: &Path) -> Option<u16> {
        // Interface type is stored as a numeric value in the "type" file.
        fs::read_to_string(iface_path.join("type"))
            .ok()
            .and_then(|s| s.trim().parse::<u16>().ok())
    }

    fn is_wireless(iface_path: &Path) -> bool {
        // A wireless interface has a "wireless" subdirectory.
        iface_path.join("wireless").exists()
    }

    pub(crate) fn get_mac_addresses() -> (Option<[u8; 6]>, Option<[u8; 6]>) {
        let mut wifi_mac = None;
        let mut ethernet_mac = None;

        // Root directory for network interfaces metadata on Linux.
        let net_dir = Path::new("/sys/class/net");

        let Ok(entries) = fs::read_dir(net_dir) else {
            warn!("Unable to read {}.", net_dir.display());
            return (None, None);
        };

        // Iterate over each directory entry representing a network interface.
        for entry in entries.flatten() {
            let iface_path = entry.path();

            // Skip interface if MAC is invalid or unreadable.
            let Some(mac) = read_mac(&iface_path) else {
                continue;
            };

            // Skip interface if it is virtual.
            if is_virtual_interface(&iface_path, mac) {
                continue;
            }

            // Skip if interface type is unknown or unparsable.
            let Some(iface_type) = get_interface_type(&iface_path) else {
                continue;
            };

            // Classify based on wireless flag and interface type.
            match (is_wireless(&iface_path), iface_type) {
                (true, t) if t == IFACE_TYPE_WIFI => wifi_mac = Some(mac),
                (false, t) if t == IFACE_TYPE_ETHERNET => ethernet_mac = Some(mac),
                _ => {}
            }
        }

        (wifi_mac, ethernet_mac)
    }

    #[cfg(test)]
    mod tests {
        use super::{VM_MAC_PREFIXES, is_locally_administered_mac, is_virtual_mac_vendor};

        #[test]
        fn test_is_locally_administered_mac() {
            assert!(is_locally_administered_mac([0x02, 0, 0, 0, 0, 0]));
            assert!(is_locally_administered_mac([0xFE, 0, 0, 0, 0, 0]));
        }

        #[test]
        fn test_is_not_locally_administered_mac() {
            assert!(!is_locally_administered_mac([0x00, 0, 0, 0, 0, 0]));
            assert!(!is_locally_administered_mac([0xFC, 0, 0, 0, 0, 0]));
        }

        #[test]
        fn test_is_virtual_mac_vendor() {
            for prefix in VM_MAC_PREFIXES {
                let mac = [prefix[0], prefix[1], prefix[2], 0, 0, 0];

                // Ensures all known VM prefixes are detected.
                //
                // Failure means a known prefix was not matched
                assert!(
                    is_virtual_mac_vendor(mac),
                    "Failed for prefix {prefix:02X?}"
                );
            }
        }

        #[test]
        fn test_is_not_virtual_mac_vendor() {
            assert!(!is_virtual_mac_vendor([0x00, 0x1A, 0x2B, 0, 0, 0]));
            assert!(!is_virtual_mac_vendor([0xFF, 0xFF, 0xFF, 0, 0, 0]));
        }
    }
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
mod os_mac {
    use std::{mem, ptr};

    use tracing::warn;

    use windows_sys::Win32::Foundation::ERROR_SUCCESS;
    use windows_sys::Win32::NetworkManagement::IpHelper::{
        GetAdaptersAddresses, GetIfEntry2, IF_TYPE_ETHERNET_CSMACD, IF_TYPE_IEEE80211,
        IP_ADAPTER_ADDRESSES_LH, MIB_IF_ROW2,
    };
    use windows_sys::Win32::NetworkManagement::Ndis::{
        IfOperStatusUp, NdisPhysicalMedium802_3 as NDIS_PHYSICAL_MEDIUM802_3,
        NdisPhysicalMediumNative802_11 as NDIS_PHYSICAL_MEDIUM_NATIVE802_11,
    };
    use windows_sys::Win32::Networking::WinSock::AF_UNSPEC;

    // Returns the MAC address only if the interface is active ("up") and
    // has a valid 6-byte address.
    // An "up" status means the interface is enabled and ready
    // for network communication.
    fn extract_mac_from_row(row: &MIB_IF_ROW2) -> Option<[u8; 6]> {
        if row.OperStatus == IfOperStatusUp && row.PhysicalAddressLength == 6 {
            let mut mac = [0u8; 6];
            mac.copy_from_slice(&row.PhysicalAddress[..6]);
            Some(mac)
        } else {
            None
        }
    }

    // Traverses the adapter linked list and extracts the first matching Wi-Fi
    // and Ethernet MAC addresses.
    fn process_adapter(
        adapter: *mut IP_ADAPTER_ADDRESSES_LH,
    ) -> (Option<[u8; 6]>, Option<[u8; 6]>) {
        let mut wifi = None;
        let mut ethernet = None;

        let mut current = adapter;
        while !current.is_null() {
            // SAFETY: `current` is a valid pointer to an
            // IP_ADAPTER_ADDRESSES_LH structure.
            // The list is well-formed and terminated with a null pointer.
            let addr = unsafe { &*current };

            // SAFETY: `row` is zero-initialized and safe to pass to
            // GetIfEntry2, which will write valid data into this structure.
            let mut row: MIB_IF_ROW2 = unsafe { mem::zeroed() };
            row.InterfaceLuid = addr.Luid;

            // Populate row with interface information based on adapter LUID.
            //
            // SAFETY: GetIfEntry2 is called with a valid pointer to `row`.
            // Return value 0 indicates success.
            if unsafe { GetIfEntry2(&mut row) } == 0 {
                if let Some(mac) = extract_mac_from_row(&row) {
                    // Verify interface type and physical medium.
                    match (row.Type, row.PhysicalMediumType) {
                        // We intentionally store only the last valid Wi-Fi and
                        // Ethernet MAC address found, as we're only interested
                        // in retrieving at least one valid MAC address.
                        (IF_TYPE_IEEE80211, NDIS_PHYSICAL_MEDIUM_NATIVE802_11) => wifi = Some(mac),
                        (IF_TYPE_ETHERNET_CSMACD, NDIS_PHYSICAL_MEDIUM802_3) => {
                            ethernet = Some(mac)
                        }
                        _ => {}
                    }
                }
            }

            // Move to next adapter in the linked list.
            current = addr.Next;
        }

        (wifi, ethernet)
    }

    pub(crate) fn get_mac_addresses() -> (Option<[u8; 6]>, Option<[u8; 6]>) {
        let mut size = 0;

        // SAFETY: First call only fills `size` to determine required
        // buffer size.
        unsafe {
            GetAdaptersAddresses(
                AF_UNSPEC as u32,
                0,
                ptr::null_mut(),
                ptr::null_mut(),
                &mut size,
            );
        }

        if size == 0 {
            warn!("`GetAdaptersAddresses` returned zero size.");
            return (None, None);
        }

        let mut buffer = vec![0u8; size as usize];
        let adapter = buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH;

        // SAFETY: `adapter` points to valid buffer of correct size
        // for storing adapter data.
        if unsafe { GetAdaptersAddresses(AF_UNSPEC as u32, 0, ptr::null_mut(), adapter, &mut size) }
            == ERROR_SUCCESS
        {
            process_adapter(adapter)
        } else {
            warn!("Unable to retrieve adapters addresses.");
            (None, None)
        }
    }
}

pub(crate) fn get_mac_addresses() -> (Option<[u8; 6]>, Option<[u8; 6]>) {
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    {
        os_mac::get_mac_addresses()
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        (None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::get_mac_addresses;

    // This test only runs on systems that have physical MAC addresses.
    // Systems with virtual MAC interfaces, such as CI environments,
    // are skipped.
    #[test]
    fn test_mac_addresses_local() {
        if option_env!("CI").is_none() {
            let (wifi_mac, ethernet_mac) = get_mac_addresses();
            assert!(
                wifi_mac.is_some() || ethernet_mac.is_some(),
                "At least one Wi-Fi or Ethernet MAC address must be present."
            );
        }
    }
}
