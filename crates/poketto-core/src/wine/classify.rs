use crate::models::WineType;

pub fn classify_wine_type(folder_name: &str) -> WineType {
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

pub fn format_version_name(folder_name: &str) -> String {
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

pub fn format_wine_name(folder_name: &str) -> String {
    folder_name.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ge_proton_classifies() {
        assert_eq!(classify_wine_type("GE-Proton9-7"), WineType::ProtonGE);
        assert_eq!(classify_wine_type("proton-ge-custom"), WineType::ProtonGE);
    }

    #[test]
    fn cachyos_and_tkg_classify() {
        assert_eq!(
            classify_wine_type("proton-cachyos-9.0"),
            WineType::ProtonCachyOS
        );
        assert_eq!(classify_wine_type("Proton-TKG-9.0"), WineType::ProtonTKG);
        assert_eq!(classify_wine_type("wine-tkg-9.0"), WineType::WineTKG);
    }

    #[test]
    fn proton_variants_classify() {
        assert_eq!(classify_wine_type("Proton 9.0"), WineType::Proton);
        assert_eq!(classify_wine_type("Proton-9.0"), WineType::Proton);
        assert_eq!(classify_wine_type("Proton Experimental"), WineType::Proton);
    }

    #[test]
    fn wine_variants_classify() {
        assert_eq!(classify_wine_type("wine-ge-8-26"), WineType::WineGE);
        assert_eq!(classify_wine_type("wine-staging-9.0"), WineType::WineStaging);
        assert_eq!(classify_wine_type("lutris-7.2"), WineType::Lutris);
        assert_eq!(classify_wine_type("wine-9.0"), WineType::Wine);
    }

    #[test]
    fn version_names_format() {
        assert_eq!(format_version_name("GE-Proton9-7"), "GE-Proton9-7");
        assert_eq!(
            format_version_name("proton-cachyos-slr-9.0"),
            "Proton CachyOS SLR"
        );
        assert_eq!(
            format_version_name("Proton Experimental"),
            "Proton Experimental"
        );
        assert_eq!(format_version_name("Proton-9.0"), "Proton-9.0");
    }
}
