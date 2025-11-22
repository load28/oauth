//! Integration tests for OAuth Client API endpoints

mod common;

use actix_web::{test, http::StatusCode};
use infrastructure::{GetClientResponse, RegisterClientRequest, RegisterClientResponse};

#[actix_web::test]
async fn test_register_public_client() {
    let state = common::create_test_app_state().await;
    let app = test::init_service(common::create_test_app(state)).await;

    let request_body = RegisterClientRequest {
        name: "Test Public Client".to_string(),
        redirect_uris: vec!["https://example.com/callback".to_string()],
        scopes: "read write".to_string(),
        client_secret: None, // Public client
    };

    let req = test::TestRequest::post()
        .uri("/clients/register")
        .set_json(&request_body)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body: RegisterClientResponse = test::read_body_json(resp).await;
    assert_eq!(body.name, "Test Public Client");
    assert_eq!(body.client_type, "public");
    assert!(!body.client_id.is_empty());
    assert!(body.client_secret.is_none());
}

#[actix_web::test]
async fn test_register_confidential_client() {
    let state = common::create_test_app_state().await;
    let app = test::init_service(common::create_test_app(state)).await;

    let request_body = RegisterClientRequest {
        name: "Test Confidential Client".to_string(),
        redirect_uris: vec!["https://example.com/callback".to_string()],
        scopes: "read write".to_string(),
        client_secret: Some("my-secret-key".to_string()),
    };

    let req = test::TestRequest::post()
        .uri("/clients/register")
        .set_json(&request_body)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body: RegisterClientResponse = test::read_body_json(resp).await;
    assert_eq!(body.name, "Test Confidential Client");
    assert_eq!(body.client_type, "confidential");
    assert!(!body.client_id.is_empty());
    assert!(body.client_secret.is_some());
}

#[actix_web::test]
async fn test_register_client_without_redirect_uris() {
    let state = common::create_test_app_state().await;
    let app = test::init_service(common::create_test_app(state)).await;

    let request_body = RegisterClientRequest {
        name: "Invalid Client".to_string(),
        redirect_uris: vec![], // Empty redirect URIs
        scopes: "read".to_string(),
        client_secret: None,
    };

    let req = test::TestRequest::post()
        .uri("/clients/register")
        .set_json(&request_body)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn test_get_client_by_id() {
    let state = common::create_test_app_state().await;
    let app = test::init_service(common::create_test_app(state)).await;

    // First register a client
    let register_body = RegisterClientRequest {
        name: "Get Test Client".to_string(),
        redirect_uris: vec!["https://example.com/callback".to_string()],
        scopes: "read".to_string(),
        client_secret: None,
    };

    let req1 = test::TestRequest::post()
        .uri("/clients/register")
        .set_json(&register_body)
        .to_request();

    let resp1 = test::call_service(&app, req1).await;
    let register_response: RegisterClientResponse = test::read_body_json(resp1).await;

    // Now get the client
    let req2 = test::TestRequest::get()
        .uri(&format!("/clients/{}", register_response.client_id))
        .to_request();

    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), StatusCode::OK);

    let body: GetClientResponse = test::read_body_json(resp2).await;
    assert_eq!(body.name, "Get Test Client");
    assert_eq!(body.client_id, register_response.client_id);
}

#[actix_web::test]
async fn test_get_client_not_found() {
    let state = common::create_test_app_state().await;
    let app = test::init_service(common::create_test_app(state)).await;

    // Use a validly formatted but nonexistent client ID (valid UUID format)
    let req = test::TestRequest::get()
        .uri("/clients/00000000-0000-0000-0000-000000000000")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_delete_client() {
    let state = common::create_test_app_state().await;
    let app = test::init_service(common::create_test_app(state)).await;

    // First register a client
    let register_body = RegisterClientRequest {
        name: "Delete Test Client".to_string(),
        redirect_uris: vec!["https://example.com/callback".to_string()],
        scopes: "read".to_string(),
        client_secret: None,
    };

    let req1 = test::TestRequest::post()
        .uri("/clients/register")
        .set_json(&register_body)
        .to_request();

    let resp1 = test::call_service(&app, req1).await;
    let register_response: RegisterClientResponse = test::read_body_json(resp1).await;

    // Delete the client
    let req2 = test::TestRequest::delete()
        .uri(&format!("/clients/{}", register_response.client_id))
        .to_request();

    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), StatusCode::NO_CONTENT);

    // Verify client is deleted
    let req3 = test::TestRequest::get()
        .uri(&format!("/clients/{}", register_response.client_id))
        .to_request();

    let resp3 = test::call_service(&app, req3).await;
    assert_eq!(resp3.status(), StatusCode::NOT_FOUND);
}
