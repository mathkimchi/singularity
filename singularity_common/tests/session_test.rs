use singularity_common::sap::session::SavedWl;

fn with_title(title: &str) -> SavedWl {
    SavedWl {
        argv: Vec::new(),
        cwd: None,
        app_id: None,
        title: title.to_string(),
    }
}

#[test]
fn url_from_title() {
    assert_eq!(
        with_title("A | B | https://example.com/?q=a").url(),
        Some("https://example.com/?q=a")
    );
    assert_eq!(
        with_title("https://example.com/a | A | B — Mozilla Firefox").url(),
        Some("https://example.com/a")
    );
    assert_eq!(with_title("bash | ~/code").url(), None);
    assert_eq!(with_title("https://example.com").url(), None);
}
