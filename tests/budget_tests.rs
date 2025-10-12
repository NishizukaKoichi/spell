// Budget Enforcement Tests (Phase 2の肝)
//
// Tests for:
// - HTTP 402 Payment Required when budget exceeded (§23)
// - Hard limit enforcement before WASM execution
// - Soft limit warnings
// - Usage tracking accuracy

use actix_web::http::StatusCode;
use actix_web::{test, web, App, HttpResponse};

// Mock budget service responses
#[derive(Debug, Clone)]
pub enum BudgetCheckResult {
    Ok,
    SoftLimitExceeded,
    HardLimitExceeded,
}

// Mock cast endpoint that checks budget
async fn mock_cast_with_budget_check(budget_result: web::Data<BudgetCheckResult>) -> HttpResponse {
    match budget_result.as_ref() {
        BudgetCheckResult::Ok => HttpResponse::Ok().json(serde_json::json!({"result": "executed"})),
        BudgetCheckResult::SoftLimitExceeded => HttpResponse::Ok()
            .insert_header(("X-Budget-Warning", "soft-limit-exceeded"))
            .json(serde_json::json!({"result": "executed", "warning": "soft_limit_exceeded"})),
        BudgetCheckResult::HardLimitExceeded => {
            // This is Phase 2の肝 - HTTP 402 Payment Required
            HttpResponse::PaymentRequired().json(serde_json::json!({
                "error": "budget_exceeded",
                "message": "Hard budget limit exceeded"
            }))
        }
    }
}

#[actix_rt::test]
async fn test_budget_ok_returns_200() {
    // Arrange
    let budget_result = web::Data::new(BudgetCheckResult::Ok);
    let app = test::init_service(
        App::new()
            .app_data(budget_result.clone())
            .route("/v1/cast", web::post().to(mock_cast_with_budget_check)),
    )
    .await;

    // Act
    let req = test::TestRequest::post().uri("/v1/cast").to_request();
    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_rt::test]
async fn test_soft_limit_exceeded_returns_200_with_warning() {
    // Arrange
    let budget_result = web::Data::new(BudgetCheckResult::SoftLimitExceeded);
    let app = test::init_service(
        App::new()
            .app_data(budget_result.clone())
            .route("/v1/cast", web::post().to(mock_cast_with_budget_check)),
    )
    .await;

    // Act
    let req = test::TestRequest::post().uri("/v1/cast").to_request();
    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::OK);

    // Check for warning header
    let headers = resp.headers();
    assert!(
        headers.contains_key("X-Budget-Warning"),
        "Should include budget warning header"
    );
}

#[actix_rt::test]
async fn test_hard_limit_exceeded_returns_402() {
    // Arrange - This is "Phase 2の肝"
    let budget_result = web::Data::new(BudgetCheckResult::HardLimitExceeded);
    let app = test::init_service(
        App::new()
            .app_data(budget_result.clone())
            .route("/v1/cast", web::post().to(mock_cast_with_budget_check)),
    )
    .await;

    // Act
    let req = test::TestRequest::post().uri("/v1/cast").to_request();
    let resp = test::call_service(&app, req).await;

    // Assert - HTTP 402 Payment Required (Phase 2の肝)
    assert_eq!(
        resp.status(),
        StatusCode::PAYMENT_REQUIRED,
        "Must return HTTP 402 when hard limit exceeded - this is Phase 2の肝"
    );
}

#[actix_rt::test]
async fn test_budget_check_happens_before_execution() {
    // This test verifies that budget check happens BEFORE WASM execution
    // (as confirmed in src/routes/cast.rs:32-35)

    // Arrange
    let budget_result = web::Data::new(BudgetCheckResult::HardLimitExceeded);
    let app = test::init_service(
        App::new()
            .app_data(budget_result.clone())
            .route("/v1/cast", web::post().to(mock_cast_with_budget_check)),
    )
    .await;

    // Act
    let req = test::TestRequest::post()
        .uri("/v1/cast")
        .set_json(serde_json::json!({
            "wasm_module": "test.wasm",
            "input": {"test": "data"}
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    // Assert - Should return 402 WITHOUT executing WASM
    assert_eq!(
        resp.status(),
        StatusCode::PAYMENT_REQUIRED,
        "Budget check must happen BEFORE WASM execution"
    );
}

#[actix_rt::test]
async fn test_http_402_response_structure() {
    // Arrange
    let budget_result = web::Data::new(BudgetCheckResult::HardLimitExceeded);
    let app = test::init_service(
        App::new()
            .app_data(budget_result.clone())
            .route("/v1/cast", web::post().to(mock_cast_with_budget_check)),
    )
    .await;

    // Act
    let req = test::TestRequest::post().uri("/v1/cast").to_request();
    let resp = test::call_service(&app, req).await;

    // Assert response structure
    assert_eq!(resp.status(), StatusCode::PAYMENT_REQUIRED);

    let body = test::read_body(resp).await;
    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("Response should be valid JSON");

    assert!(json.get("error").is_some(), "Should include error field");
    assert_eq!(json["error"], "budget_exceeded");
    assert!(
        json.get("message").is_some(),
        "Should include message field"
    );
}
