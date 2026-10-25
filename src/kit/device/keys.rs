//! The keys of a registry entry's property table, in our words. A real
//! registry reports each as a string (`IOProviderClass`, `IOProbeScore`,
//! `boot-args`); [`Key::reported`] gives that spelling, so a dump of a
//! device's registry is read into these names and nothing else.

use crate::named;
use crate::kit::tell::{cases, Named};
use crate::quest::Row;

named! {
    /// A property key of a registry entry.
    pub enum Key {
        // the kit and the registry
        KitBuildVersion, KitDiagnostics, KitDebug, RegistryPlanes, Catalogue,
        RegistryEntryId, RegistryEntryPropertyKeys, RegistryEntryAllowableSetProperties, RegistryEntryDefaultLockingSetProperties,
        // matching
        Class, ProviderClass, ProbeScore, UserClass, UserClasses, SupportedProperties, UserServiceProperties,
        NameMatch, PropertyMatch, PropertyExistsMatch, PathMatch, LocationMatch, ParentMatch, ResourceMatch, ResourceMatched,
        MatchedServiceCount, NameMatched, MatchCategory, DefaultMatchCategory, MatchedPersonality, RematchPersonality, RematchCount,
        DextMatchCount, MatchDefer, AllCpuInitialized, CompatibilityMatch, CompatibilityProperties,
        // user clients
        UserClientClass, UserClientEntitlements, ServiceDextEntitlements, UserClientCrossEndian, UserClientCrossEndianCompatible,
        UserClientSharedInstance, UserClientDefaultLocking, UserClientDefaultLockingSetProperties, UserClientDefaultLockingSingleThreadExternalMethod,
        UserClientCreator, UserServerCdHash, UserUserClient, UserServerOneProcess, UserServerPreserveUserspaceReboot,
        // the platform and the device
        MapperId, PlatformDeviceMessage, CfPlugInTypes, Path, Icon, BundleResourceFile, BusBadge, DeviceIcon,
        PlatformSerialNumber, PlatformUuid, BootDevice, BootDevicePath, BootDeviceSize,
        // nvram
        NvramBootArgs, NvramDeleteProperty, NvramSyncNowProperty, NvramActivateCsrConfigProperty,
        // storage limits
        CommandPoolSize, MaximumPriorityCount, MaximumBlockCountRead, MaximumBlockCountWrite, MaximumByteCountRead, MaximumByteCountWrite,
        MaximumSegmentCountRead, MaximumSegmentCountWrite, MaximumSegmentByteCountRead, MaximumSegmentByteCountWrite,
        MinimumSegmentAlignmentByteCount, MaximumSegmentAddressableBitCount, MinimumSaturationByteCount, MaximumSwapWrite,
    }
}

impl Key {
    /// The key as a registry reports it: the identifier after the kit's
    /// prefix, except the few spelled otherwise.
    pub fn reported(self) -> String {
        match self {
            Key::NvramBootArgs => "boot-args".into(),
            Key::NvramDeleteProperty => "IONVRAM-DELETE-PROPERTY".into(),
            Key::NvramSyncNowProperty => "IONVRAM-SYNCNOW-PROPERTY".into(),
            Key::NvramActivateCsrConfigProperty => "IONVRAM-ARMCSR-PROPERTY".into(),
            Key::MatchedServiceCount => "IOMatchedServiceCountMatch".into(),
            Key::PlatformUuid => "IOPlatformUUID".into(),
            Key::UserServerCdHash => "IOUserServerCDHash".into(),
            Key::CfPlugInTypes => "IOCFPlugInTypes".into(),
            Key::DextMatchCount => "IODEXTMatchCount".into(),
            Key::ServiceDextEntitlements => "IOServiceDEXTEntitlements".into(),
            Key::AllCpuInitialized => "IOAllCPUInitialized".into(),
            Key::RegistryEntryId => "IORegistryEntryID".into(),
            Key::MapperId => "IOMapperID".into(),
            _ => format!("IO{}", self.ident()),
        }
    }

    /// The key a registry string names, if any.
    pub fn from_reported(s: &str) -> Option<Key> {
        Key::ALL.iter().copied().find(|k| k.reported() == s)
    }

    /// Whether the key is one matching reads.
    pub fn is_matching(self) -> bool {
        self.ident().ends_with("Match") || matches!(self, Key::Class | Key::ProviderClass | Key::ProbeScore | Key::MatchCategory)
    }
}

/// The module told as rows under `path`: each key, and the string it `names`.
pub fn told(path: &str) -> Vec<Row> {
    let mut r = cases::<Key>(path);
    for k in Key::ALL {
        r.push(crate::kit::tell::row(&k.name(), "names", &k.reported()));
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_round_trip_their_reported_strings() {
        assert_eq!(Key::ProviderClass.reported(), "IOProviderClass");
        assert_eq!(Key::NvramBootArgs.reported(), "boot-args");
        assert_eq!(Key::from_reported("IOProbeScore"), Some(Key::ProbeScore));
        assert_eq!(Key::from_reported("IOPlatformUUID"), Some(Key::PlatformUuid));
        assert!(Key::NameMatch.is_matching() && !Key::Icon.is_matching());
        for k in Key::ALL {
            assert_eq!(Key::from_reported(&k.reported()), Some(*k), "{k:?} is unique");
        }
    }
}
