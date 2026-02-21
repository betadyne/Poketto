use crate::models::WineType;

pub(super) fn classify_wine_type(folder_name: &str) -> WineType {
    let name = folder_name.to_lowercase();

    if name.contains("ge-proton") || name.contains("proton-ge") {
        WineType::ProtonGE
    } else if name.contains("cachyos") {
        WineType::ProtonCachyOS
    } else if name.contains("proton") && name.contains("tkg") {
        WineType::ProtonTKG
    } else if name.starts_with("proton") || name.contains("-proton") {
        WineType::Proton
    } else if name.contains("wine-ge") || name.contains("ge-wine") {
        WineType::WineGE
    } else if name.contains("staging") {
        WineType::WineStaging
    } else if name.contains("wine") && name.contains("tkg") {
        WineType::WineTKG
    } else if name.contains("lutris") {
        WineType::Lutris
    } else {
        WineType::Wine
    }
}

pub(super) fn format_version_name(folder_name: &str) -> String {
    let name_lower = folder_name.to_lowercase();

    if name_lower.starts_with("ge-proton") {
        folder_name.to_string()
    } else if name_lower.contains("cachyos") && name_lower.contains("slr") {
        "Proton CachyOS SLR".to_string()
    } else if name_lower.contains("cachyos") {
        "Proton CachyOS".to_string()
    } else if name_lower.contains("experimental") {
        "Proton Experimental".to_string()
    } else if name_lower.starts_with("proton ") || name_lower.starts_with("proton-") {
        folder_name.replace(" - ", " ").replace("_", " ")
    } else {
        folder_name.to_string()
    }
}

pub(super) fn format_wine_name(folder_name: &str) -> String {
    folder_name.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    mod classify_wine_type_tests {
        use super::*;

        #[test]
        fn test_ge_proton_variants() {
            assert_eq!(classify_wine_type("GE-Proton9-20"), WineType::ProtonGE);
            assert_eq!(classify_wine_type("GE-Proton8-1"), WineType::ProtonGE);
            assert_eq!(classify_wine_type("proton-ge-custom"), WineType::ProtonGE);
            assert_eq!(classify_wine_type("Proton-GE-8-25"), WineType::ProtonGE);
        }

        #[test]
        fn test_cachyos_variants() {
            assert_eq!(
                classify_wine_type("proton-cachyos"),
                WineType::ProtonCachyOS
            );
            assert_eq!(
                classify_wine_type("proton-cachyos-slr"),
                WineType::ProtonCachyOS
            );
            assert_eq!(
                classify_wine_type("CachyOS-Proton"),
                WineType::ProtonCachyOS
            );
        }

        #[test]
        fn test_proton_tkg() {
            assert_eq!(classify_wine_type("proton-tkg"), WineType::ProtonTKG);
            assert_eq!(classify_wine_type("Proton-TKG-9.0"), WineType::ProtonTKG);
        }

        #[test]
        fn test_standard_proton() {
            assert_eq!(classify_wine_type("Proton 9.0"), WineType::Proton);
            assert_eq!(classify_wine_type("Proton-9.0-3"), WineType::Proton);
            assert_eq!(classify_wine_type("proton_experimental"), WineType::Proton);
        }

        #[test]
        fn test_wine_ge_variants() {
            assert_eq!(classify_wine_type("wine-ge-custom"), WineType::WineGE);
            assert_eq!(classify_wine_type("GE-Wine-8-26"), WineType::WineGE);
        }

        #[test]
        fn test_wine_staging() {
            assert_eq!(classify_wine_type("wine-staging"), WineType::WineStaging);
            assert_eq!(
                classify_wine_type("Wine-Staging-9.0"),
                WineType::WineStaging
            );
        }

        #[test]
        fn test_wine_tkg() {
            assert_eq!(classify_wine_type("wine-tkg"), WineType::WineTKG);
            assert_eq!(classify_wine_type("Wine-TKG-fsync"), WineType::WineTKG);
        }

        #[test]
        fn test_lutris() {
            assert_eq!(classify_wine_type("lutris-runner"), WineType::Lutris);
            assert_eq!(classify_wine_type("wine-lutris-7.2"), WineType::Lutris);
        }

        #[test]
        fn test_fallback_to_wine() {
            assert_eq!(classify_wine_type(""), WineType::Wine);
            assert_eq!(classify_wine_type("unknown-runner"), WineType::Wine);
            assert_eq!(classify_wine_type("some-random-name"), WineType::Wine);
        }

        #[test]
        fn test_case_insensitivity() {
            assert_eq!(classify_wine_type("GE-PROTON9-20"), WineType::ProtonGE);
            assert_eq!(classify_wine_type("ge-proton9-20"), WineType::ProtonGE);
            assert_eq!(
                classify_wine_type("CACHYOS-Proton"),
                WineType::ProtonCachyOS
            );
        }
    }

    mod format_version_name_tests {
        use super::*;

        #[test]
        fn test_ge_proton_keeps_original() {
            assert_eq!(format_version_name("GE-Proton9-20"), "GE-Proton9-20");
            assert_eq!(format_version_name("GE-Proton8-1"), "GE-Proton8-1");
        }

        #[test]
        fn test_cachyos_slr() {
            assert_eq!(
                format_version_name("proton-cachyos-slr"),
                "Proton CachyOS SLR"
            );
            assert_eq!(
                format_version_name("CachyOS-SLR-Proton"),
                "Proton CachyOS SLR"
            );
        }

        #[test]
        fn test_cachyos_plain() {
            assert_eq!(format_version_name("proton-cachyos"), "Proton CachyOS");
            assert_eq!(format_version_name("CachyOS-Proton-9"), "Proton CachyOS");
        }

        #[test]
        fn test_experimental() {
            assert_eq!(
                format_version_name("Proton-Experimental"),
                "Proton Experimental"
            );
            assert_eq!(
                format_version_name("proton_experimental"),
                "Proton Experimental"
            );
        }

        #[test]
        fn test_standard_proton_formatting() {
            assert_eq!(format_version_name("Proton 9.0 - 3"), "Proton 9.0 3");
            assert_eq!(format_version_name("Proton-8.0_beta"), "Proton-8.0 beta");
        }

        #[test]
        fn test_other_names_unchanged() {
            assert_eq!(format_version_name("wine-staging-9.0"), "wine-staging-9.0");
            assert_eq!(format_version_name("lutris-runner"), "lutris-runner");
        }

        #[test]
        fn test_empty_string() {
            assert_eq!(format_version_name(""), "");
        }
    }

    mod format_wine_name_tests {
        use super::*;

        #[test]
        fn test_returns_same_string() {
            assert_eq!(format_wine_name("wine-staging"), "wine-staging");
            assert_eq!(format_wine_name("Wine-GE-8-26"), "Wine-GE-8-26");
        }

        #[test]
        fn test_empty_string() {
            assert_eq!(format_wine_name(""), "");
        }
    }
}
