//! User Management HTTP Handlers

use crate::app_state::AppState;
use crate::dtos::{
    AuthenticateUserRequest, AuthenticateUserResponse, ChangePasswordRequest,
    ChangePasswordResponse, ErrorResponse, GetUserResponse, RegisterUserRequest,
    RegisterUserResponse,
};
use actix_web::{web, HttpResponse, ResponseError};
use application::ports::input::{
    AuthenticateUserInput, ChangePasswordInput, RegisterUserInput, UserUseCase,
};
use application::ports::{
    AccessTokenRepository, AuthCodeRepository, ClientRepository, RefreshTokenRepository,
    UserRepository,
};
use domain::value_objects::UserId;

/// POST /users/register
///
/// Register a new user account
pub async fn register_user<UR, CR, AR, ATR, RTR>(
    state: web::Data<AppState<UR, CR, AR, ATR, RTR>>,
    req: web::Json<RegisterUserRequest>,
) -> HttpResponse
where
    UR: UserRepository,
    CR: ClientRepository,
    AR: AuthCodeRepository,
    ATR: AccessTokenRepository,
    RTR: RefreshTokenRepository,
{
    let input = RegisterUserInput {
        email: req.email.clone(),
        password: req.password.clone(),
    };

    match state.user_service.register_user(input).await {
        Ok(output) => {
            let response = RegisterUserResponse {
                user_id: output.user_id.to_string(),
                email: output.email.to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
            };
            HttpResponse::Created().json(response)
        }
        Err(e) => {
            let error = match e {
                application::errors::UseCaseError::Validation(msg) => {
                    ErrorResponse::bad_request(&msg)
                }
                application::errors::UseCaseError::Domain(msg) => ErrorResponse::conflict(&msg),
                _ => ErrorResponse::internal_error("Failed to register user"),
            };
            error.error_response()
        }
    }
}

/// POST /auth/login
///
/// Authenticate user and return user information
pub async fn authenticate_user<UR, CR, AR, ATR, RTR>(
    state: web::Data<AppState<UR, CR, AR, ATR, RTR>>,
    req: web::Json<AuthenticateUserRequest>,
) -> HttpResponse
where
    UR: UserRepository,
    CR: ClientRepository,
    AR: AuthCodeRepository,
    ATR: AccessTokenRepository,
    RTR: RefreshTokenRepository,
{
    let input = AuthenticateUserInput {
        email: req.email.clone(),
        password: req.password.clone(),
    };

    match state.user_service.authenticate_user(input).await {
        Ok(output) => {
            let response = AuthenticateUserResponse {
                user_id: output.user_id.to_string(),
                email: output.email.to_string(),
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            let error = match e {
                application::errors::UseCaseError::Unauthorized => {
                    ErrorResponse::unauthorized("Invalid credentials")
                }
                application::errors::UseCaseError::NotFound => {
                    ErrorResponse::not_found("User not found")
                }
                _ => ErrorResponse::internal_error("Authentication failed"),
            };
            error.error_response()
        }
    }
}

/// GET /users/{id}
///
/// Get user information by ID
pub async fn get_user<UR, CR, AR, ATR, RTR>(
    state: web::Data<AppState<UR, CR, AR, ATR, RTR>>,
    user_id_str: web::Path<String>,
) -> HttpResponse
where
    UR: UserRepository,
    CR: ClientRepository,
    AR: AuthCodeRepository,
    ATR: AccessTokenRepository,
    RTR: RefreshTokenRepository,
{
    let user_id = match UserId::parse(&user_id_str.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse::bad_request("Invalid user ID format");
            return error.error_response();
        }
    };

    match state.user_service.get_user(user_id).await {
        Ok(user) => {
            let response = GetUserResponse {
                user_id: user.id().to_string(),
                email: user.email().to_string(),
                created_at: user.created_at().to_rfc3339(),
                updated_at: user.updated_at().to_rfc3339(),
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            let error = match e {
                application::errors::UseCaseError::NotFound => {
                    ErrorResponse::not_found("User not found")
                }
                application::errors::UseCaseError::Validation(msg) => {
                    ErrorResponse::bad_request(&msg)
                }
                _ => ErrorResponse::internal_error("Failed to get user"),
            };
            error.error_response()
        }
    }
}

/// POST /users/{id}/password
///
/// Change user password
pub async fn change_password<UR, CR, AR, ATR, RTR>(
    state: web::Data<AppState<UR, CR, AR, ATR, RTR>>,
    user_id_str: web::Path<String>,
    req: web::Json<ChangePasswordRequest>,
) -> HttpResponse
where
    UR: UserRepository,
    CR: ClientRepository,
    AR: AuthCodeRepository,
    ATR: AccessTokenRepository,
    RTR: RefreshTokenRepository,
{
    let user_id = match UserId::parse(&user_id_str.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse::bad_request("Invalid user ID format");
            return error.error_response();
        }
    };

    let input = ChangePasswordInput {
        user_id,
        old_password: req.old_password.clone(),
        new_password: req.new_password.clone(),
    };

    match state.user_service.change_password(input).await {
        Ok(_) => {
            let response = ChangePasswordResponse {
                success: true,
                message: "Password changed successfully".to_string(),
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            let error = match e {
                application::errors::UseCaseError::Unauthorized => {
                    ErrorResponse::unauthorized("Invalid old password")
                }
                application::errors::UseCaseError::NotFound => {
                    ErrorResponse::not_found("User not found")
                }
                application::errors::UseCaseError::Validation(msg) => {
                    ErrorResponse::bad_request(&msg)
                }
                _ => ErrorResponse::internal_error("Failed to change password"),
            };
            error.error_response()
        }
    }
}
