use app::error::AppError;
use domain::DomainError;

#[test]
fn invalid_ratio_maps_to_validation() {
    let e: AppError = DomainError::InvalidRatio(-1.0).into();
    assert_eq!(e.code, "VALIDATION");
    assert!(e.detail.contains("invalid ratio"));
}

#[test]
fn dependency_cycle_maps_to_domain() {
    let e: AppError = DomainError::DependencyCycle(7).into();
    assert_eq!(e.code, "DOMAIN");
}

#[test]
fn not_found_maps_to_not_found() {
    let e: AppError = DomainError::NotFound("task 5".into()).into();
    assert_eq!(e.code, "NOT_FOUND");
    assert_eq!(e.detail, "task 5");
}

#[test]
fn app_error_serializes_to_code_detail() {
    let e = AppError::domain("boom".into());
    let json = serde_json::to_string(&e).unwrap();
    assert!(json.contains("\"code\":\"DOMAIN\""));
    assert!(json.contains("\"detail\":\"boom\""));
}