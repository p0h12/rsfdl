use crate::sfdl::models::{BulkFolder, Package};

/// Builds a BulkFolder package (no FTP connection needed).
pub fn build_bulk_package(path: &str, package_name: &str) -> Package {
    Package {
        name: package_name.to_string(),
        bulk_folder_mode: true,
        file_list: Vec::new(),
        bulk_folder_list: vec![BulkFolder {
            bulk_folder_path: path.to_string(),
            package_name: package_name.to_string(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_bulk_package_creates_correct_structure() {
        let pkg = build_bulk_package("/releases/movie/", "MyPackage");

        assert_eq!(pkg.name, "MyPackage");
        assert!(pkg.bulk_folder_mode);
        assert!(pkg.file_list.is_empty());
        assert_eq!(pkg.bulk_folder_list.len(), 1);
        assert_eq!(pkg.bulk_folder_list[0].bulk_folder_path, "/releases/movie/");
        assert_eq!(pkg.bulk_folder_list[0].package_name, "MyPackage");
    }
}
