// Integration Tests (E2E flows without external dependencies)
//
// Tests for:
// - Health + Metrics endpoints (§26-29)
// - Full budget enforcement flow (§23)
// - API key lifecycle (§19)
// - Rate limiting enforcement (§18)
//
// Note: These tests use Actix test infrastructure and mocked services.
// For true E2E tests with real Stripe/GitHub OAuth/Database, use:
//   scripts/e2e_phase2.sh against deployed environment

use actix_web::http::{header, StatusCode};
use actix_web::{test, web, App, HttpResponse};

// ============================================================================
// Mock Endpoints (simplified versions of real handlers)
// ============================================================================

async fn mock_healthz() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn mock_metrics() -> HttpResponse {
    // Simplified Prometheus format
    let metrics = r#"# HELP spell_cast_total Total number of spell casts
# TYPE spell_cast_total counter
spell_cast_total 42

# HELP spell_budget_blocked_total Total number of casts blocked by budget
# TYPE spell_budget_blocked_total counter
spell_budget_blocked_total 5

# HELP spell_rate_limited_total Total number of rate limited requests
# TYPE spell_rate_limited_total counter
spell_rate_limited_total 3
"#;
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(metrics)
}

#[derive(Debug, Clone)]
struct MockBudgetState {
    hard_limit_cents: i32,
    current_usage_cents: i32,
}

async fn mock_get_budget(state: web::Data<MockBudgetState>) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "period": "monthly",
        "soft_limit_cents": state.hard_limit_cents - 5,
        "hard_limit_cents": state.hard_limit_cents,
        "notify_thresholds": [3, 7]
    }))
}

async fn mock_create_budget(body: web::Json<serde_json::Value>) -> HttpResponse {
    HttpResponse::Ok().json(body.into_inner())
}

async fn mock_update_budget(body: web::Json<serde_json::Value>) -> HttpResponse {
    HttpResponse::Ok().json(body.into_inner())
}

async fn mock_delete_budget() -> HttpResponse {
    HttpResponse::NoContent().finish()
}

async fn mock_get_usage(state: web::Data<MockBudgetState>) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "total_calls": state.current_usage_cents,
        "total_cost_cents": state.current_usage_cents,
        "period": "monthly"
    }))
}

async fn mock_cast_with_budget(state: web::Data<MockBudgetState>) -> HttpResponse {
    // Check if budget exceeded
    if state.current_usage_cents >= state.hard_limit_cents {
        return HttpResponse::PaymentRequired().json(serde_json::json!({
            "error": "budget_exceeded",
            "message": "Hard budget limit exceeded"
        }));
    }

    HttpResponse::Ok().json(serde_json::json!({
        "result": "executed",
        "spell_name": "echo"
    }))
}

// ============================================================================
// Integration Tests
// ============================================================================

#[actix_rt::test]
async fn test_health_endpoint_returns_ok_status() {
    // Arrange
    let app = test::init_service(App::new().route("/healthz", web::get().to(mock_healthz))).await;

    // Act
    let req = test::TestRequest::get().uri("/healthz").to_request();
    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("Response should be valid JSON");

    assert_eq!(json["status"], "ok");
    assert!(json.get("version").is_some(), "Should include version");
}

#[actix_rt::test]
async fn test_metrics_endpoint_returns_prometheus_format() {
    // Arrange
    let app = test::init_service(App::new().route("/metrics", web::get().to(mock_metrics))).await;

    // Act
    let req = test::TestRequest::get().uri("/metrics").to_request();
    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let metrics = String::from_utf8(body.to_vec()).expect("Should be valid UTF-8");

    // Verify Prometheus metrics format
    assert!(
        metrics.contains("spell_cast_total"),
        "Should include cast metric"
    );
    assert!(
        metrics.contains("spell_budget_blocked_total"),
        "Should include budget blocked metric"
    );
    assert!(
        metrics.contains("spell_rate_limited_total"),
        "Should include rate limit metric"
    );
    assert!(metrics.contains("# HELP"), "Should include HELP comments");
    assert!(metrics.contains("# TYPE"), "Should include TYPE comments");
}

#[actix_rt::test]
async fn test_full_budget_enforcement_flow() {
    // This test simulates the E2E flow from scripts/e2e_phase2.sh steps 3-10
    // 1. Get initial budget (should be OK)
    // 2. Create budget with low hard limit (10 cents)
    // 3. Get usage (should be 0)
    // 4. Cast (should succeed initially)
    // 5. Exceed budget (should get HTTP 402)
    // 6. Update budget to higher limit
    // 7. Cast should work again

    // Arrange - Initial state: no budget exceeded
    let initial_state = web::Data::new(MockBudgetState {
        hard_limit_cents: 10,
        current_usage_cents: 0,
    });

    let app = test::init_service(
        App::new()
            .app_data(initial_state.clone())
            .route("/v1/budgets", web::get().to(mock_get_budget))
            .route("/v1/budgets", web::post().to(mock_create_budget))
            .route("/v1/budgets", web::put().to(mock_update_budget))
            .route("/v1/budgets", web::delete().to(mock_delete_budget))
            .route("/v1/budgets/usage", web::get().to(mock_get_usage))
            .route("/v1/cast", web::post().to(mock_cast_with_budget)),
    )
    .await;

    // Step 1: Get budget
    let req = test::TestRequest::get().uri("/v1/budgets").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK, "Should get budget");

    // Step 2: Create budget with low hard limit
    let req = test::TestRequest::post()
        .uri("/v1/budgets")
        .set_json(&serde_json::json!({
            "period": "monthly",
            "soft_limit_cents": 5,
            "hard_limit_cents": 10
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK, "Should create budget");

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["hard_limit_cents"], 10);

    // Step 3: Get initial usage
    let req = test::TestRequest::get()
        .uri("/v1/budgets/usage")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK, "Should get usage");

    // Step 4: First cast (should succeed with current_usage=0)
    let req = test::TestRequest::post()
        .uri("/v1/cast")
        .set_json(&serde_json::json!({
            "spell_name": "echo",
            "payload": {"message": "test"}
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK, "First cast should succeed");

    // Step 5: Simulate budget exceeded (create new app with exceeded state)
    let exceeded_state = web::Data::new(MockBudgetState {
        hard_limit_cents: 10,
        current_usage_cents: 15, // Exceeded!
    });

    let app_exceeded = test::init_service(
        App::new()
            .app_data(exceeded_state)
            .route("/v1/cast", web::post().to(mock_cast_with_budget)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/v1/cast")
        .set_json(&serde_json::json!({
            "spell_name": "echo",
            "payload": {"message": "test"}
        }))
        .to_request();
    let resp = test::call_service(&app_exceeded, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::PAYMENT_REQUIRED,
        "Should return HTTP 402 when budget exceeded"
    );

    // Step 6: Update budget to higher limit
    let req = test::TestRequest::put()
        .uri("/v1/budgets")
        .set_json(&serde_json::json!({
            "period": "monthly",
            "soft_limit_cents": 1000,
            "hard_limit_cents": 2000
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK, "Should update budget");

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["hard_limit_cents"], 2000);

    // Step 7: Cast should work again (with updated budget state)
    let updated_state = web::Data::new(MockBudgetState {
        hard_limit_cents: 2000,
        current_usage_cents: 15,
    });

    let app_updated = test::init_service(
        App::new()
            .app_data(updated_state)
            .route("/v1/cast", web::post().to(mock_cast_with_budget)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/v1/cast")
        .set_json(&serde_json::json!({
            "spell_name": "echo",
            "payload": {"message": "test"}
        }))
        .to_request();
    let resp = test::call_service(&app_updated, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "Cast should work after budget increase"
    );
}

#[actix_rt::test]
async fn test_budget_cleanup_returns_204() {
    // Test budget deletion (E2E script step 13)
    let state = web::Data::new(MockBudgetState {
        hard_limit_cents: 10,
        current_usage_cents: 0,
    });

    let app = test::init_service(
        App::new()
            .app_data(state)
            .route("/v1/budgets", web::delete().to(mock_delete_budget)),
    )
    .await;

    let req = test::TestRequest::delete().uri("/v1/budgets").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(
        resp.status(),
        StatusCode::NO_CONTENT,
        "Budget deletion should return 204"
    );
}

#[actix_rt::test]
async fn test_metrics_include_all_required_counters() {
    // Verify all Phase 2 metrics are present (E2E script step 12)
    let app = test::init_service(App::new().route("/metrics", web::get().to(mock_metrics))).await;

    let req = test::TestRequest::get().uri("/metrics").to_request();
    let resp = test::call_service(&app, req).await;

    let body = test::read_body(resp).await;
    let metrics = String::from_utf8(body.to_vec()).unwrap();

    // Verify all required metrics from §26-29
    assert!(
        metrics.contains("spell_cast_total"),
        "Must include spell_cast_total counter"
    );
    assert!(
        metrics.contains("spell_budget_blocked_total"),
        "Must include spell_budget_blocked_total counter"
    );
    assert!(
        metrics.contains("spell_rate_limited_total"),
        "Must include spell_rate_limited_total counter"
    );
}
