//! Integration tests for User API endpoints

mod common;

use actix_web::{test, http::StatusCode};
use infrastructure::{
    AuthenticateUserRequest, AuthenticateUserResponse, ChangePasswordRequest,
    ChangePasswordResponse, GetUserResponse, RegisterUserRequest, RegisterUserResponse,
};

#[actix_web::test]
async fn test_user_registration_success() {
    let state = common::create_test_app_state().await;
    let app = test::init_service(common::create_test_app(state)).await;

    let request_body = RegisterUserRequest {
        email: "test@example.com".to_string(),
        password: "SecurePassword123!".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/users/register")
        .set_json(&request_body)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body: RegisterUserResponse = test::read_body_json(resp).await;
    assert_eq!(body.email, "test@example.com");
    assert!(!body.user_id.is_empty());
}

#[actix_web::test]
async fn test_user_registration_duplicate_email() {
    let state = common::create_test_app_state().await;
    let app = test::init_service(common::create_test_app(state)).await;

    let request_body = RegisterUserRequest {
        email: "duplicate@example.com".to_string(),
        password: "SecurePassword123!".to_string(),
    };

    // Register first time
    let req1 = test::TestRequest::post()
        .uri("/users/register")
        .set_json(&request_body)
        .to_request();
    let resp1 = test::call_service(&app, req1).await;
    assert_eq!(resp1.status(), StatusCode::CREATED);

    // Try to register again with same email
    let req2 = test::TestRequest::post()
        .uri("/users/register")
        .set_json(&request_body)
        .to_request();
    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), StatusCode::CONFLICT);
}

#[actix_web::test]
async fn test_user_authentication_success() {
    let state = common::create_test_app_state().await;
    let app = test::init_service(common::create_test_app(state)).await;

    // First register a user
    let register_body = RegisterUserRequest {
        email: "auth@example.com".to_string(),
        password: "SecurePassword123!".to_string(),
    };

    let req1 = test::TestRequest::post()
        .uri("/users/register")
        .set_json(&register_body)
        .to_request();
    test::call_service(&app, req1).await;

    // Now authenticate
    let auth_body = AuthenticateUserRequest {
        email: "auth@example.com".to_string(),
        password: "SecurePassword123!".to_string(),
    };

    let req2 = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(&auth_body)
        .to_request();

    let resp = test::call_service(&app, req2).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: AuthenticateUserResponse = test::read_body_json(resp).await;
    assert_eq!(body.email, "auth@example.com");
    assert!(!body.user_id.is_empty());
}

#[actix_web::test]
async fn test_user_authentication_wrong_password() {
    let state = common::create_test_app_state().await;
    let app = test::init_service(common::create_test_app(state)).await;

    // First register a user
    let register_body = RegisterUserRequest {
        email: "wrongpass@example.com".to_string(),
        password: "CorrectPassword123!".to_string(),
    };

    let req1 = test::TestRequest::post()
        .uri("/users/register")
        .set_json(&register_body)
        .to_request();
    test::call_service(&app, req1).await;

    // Try to authenticate with wrong password
    let auth_body = AuthenticateUserRequest {
        email: "wrongpass@example.com".to_string(),
        password: "WrongPassword123!".to_string(),
    };

    let req2 = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(&auth_body)
        .to_request();

    let resp = test::call_service(&app, req2).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn test_get_user_by_id() {
    let state = common::create_test_app_state().await;
    let app = test::init_service(common::create_test_app(state)).await;

    // First register a user
    let register_body = RegisterUserRequest {
        email: "getuser@example.com".to_string(),
        password: "SecurePassword123!".to_string(),
    };

    let req1 = test::TestRequest::post()
        .uri("/users/register")
        .set_json(&register_body)
        .to_request();

    let resp1 = test::call_service(&app, req1).await;
    let register_response: RegisterUserResponse = test::read_body_json(resp1).await;

    // Now get the user
    let req2 = test::TestRequest::get()
        .uri(&format!("/users/{}", register_response.user_id))
        .to_request();

    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), StatusCode::OK);

    let body: GetUserResponse = test::read_body_json(resp2).await;
    assert_eq!(body.email, "getuser@example.com");
    assert_eq!(body.user_id, register_response.user_id);
}

#[actix_web::test]
async fn test_get_user_not_found() {
    let state = common::create_test_app_state().await;
    let app = test::init_service(common::create_test_app(state)).await;

    let req = test::TestRequest::get()
        .uri("/users/00000000-0000-0000-0000-000000000000")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_change_password_success() {
    let state = common::create_test_app_state().await;
    let app = test::init_service(common::create_test_app(state)).await;

    // First register a user
    let register_body = RegisterUserRequest {
        email: "changepass@example.com".to_string(),
        password: "OldPassword123!".to_string(),
    };

    let req1 = test::TestRequest::post()
        .uri("/users/register")
        .set_json(&register_body)
        .to_request();

    let resp1 = test::call_service(&app, req1).await;
    let register_response: RegisterUserResponse = test::read_body_json(resp1).await;

    // Change the password
    let change_pass_body = ChangePasswordRequest {
        old_password: "OldPassword123!".to_string(),
        new_password: "NewPassword123!".to_string(),
    };

    let req2 = test::TestRequest::post()
        .uri(&format!("/users/{}/password", register_response.user_id))
        .set_json(&change_pass_body)
        .to_request();

    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), StatusCode::OK);

    let body: ChangePasswordResponse = test::read_body_json(resp2).await;
    assert!(body.success);

    // Verify we can authenticate with the new password
    let auth_body = AuthenticateUserRequest {
        email: "changepass@example.com".to_string(),
        password: "NewPassword123!".to_string(),
    };

    let req3 = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(&auth_body)
        .to_request();

    let resp3 = test::call_service(&app, req3).await;
    assert_eq!(resp3.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_change_password_wrong_old_password() {
    let state = common::create_test_app_state().await;
    let app = test::init_service(common::create_test_app(state)).await;

    // First register a user
    let register_body = RegisterUserRequest {
        email: "wrongold@example.com".to_string(),
        password: "CorrectPassword123!".to_string(),
    };

    let req1 = test::TestRequest::post()
        .uri("/users/register")
        .set_json(&register_body)
        .to_request();

    let resp1 = test::call_service(&app, req1).await;
    let register_response: RegisterUserResponse = test::read_body_json(resp1).await;

    // Try to change password with wrong old password
    let change_pass_body = ChangePasswordRequest {
        old_password: "WrongOldPassword123!".to_string(),
        new_password: "NewPassword123!".to_string(),
    };

    let req2 = test::TestRequest::post()
        .uri(&format!("/users/{}/password", register_response.user_id))
        .set_json(&change_pass_body)
        .to_request();

    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), StatusCode::UNAUTHORIZED);
}
