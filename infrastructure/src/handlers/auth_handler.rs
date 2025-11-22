//! OAuth Authorization HTTP Handlers

use crate::app_state::AppState;
use crate::dtos::{
    AuthorizeRequest, AuthorizeResponse, ConsentRequest, ConsentResponse, ErrorResponse,
};
use actix_web::{web, HttpResponse, ResponseError};
use application::ports::input::{AuthUseCase, AuthorizeInput};
use application::ports::{
    AccessTokenRepository, AuthCodeRepository, ClientRepository, RefreshTokenRepository,
    UserRepository,
};
use domain::value_objects::{ClientId, UserId};

/// GET /authorize
///
/// OAuth 2.0 authorization endpoint (initiate authorization flow)
pub async fn authorize<UR, CR, AR, ATR, RTR>(
    _state: web::Data<AppState<UR, CR, AR, ATR, RTR>>,
    query: web::Query<AuthorizeRequest>,
) -> HttpResponse
where
    UR: UserRepository,
    CR: ClientRepository,
    AR: AuthCodeRepository,
    ATR: AccessTokenRepository,
    RTR: RefreshTokenRepository,
{
    // Validate request parameters
    if query.response_type != "code" {
        let error = ErrorResponse::bad_request("Unsupported response_type (must be 'code')");
        return error.error_response();
    }

    // Return authorization details (in real app, redirect to login/consent page)
    let response = AuthorizeResponse {
        message: "Authorization request received. User must authenticate and grant consent."
            .to_string(),
        client_id: query.client_id.clone(),
        requested_scopes: query.scope.clone(),
        redirect_uri: query.redirect_uri.clone(),
        requires_authentication: true,
    };

    HttpResponse::Ok().json(response)
}

/// POST /authorize/consent
///
/// Grant authorization consent and generate authorization code
pub async fn consent<UR, CR, AR, ATR, RTR>(
    state: web::Data<AppState<UR, CR, AR, ATR, RTR>>,
    req: web::Json<ConsentRequest>,
) -> HttpResponse
where
    UR: UserRepository,
    CR: ClientRepository,
    AR: AuthCodeRepository,
    ATR: AccessTokenRepository,
    RTR: RefreshTokenRepository,
{
    let client_id = match ClientId::parse(&req.client_id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse::bad_request("Invalid client ID format");
            return error.error_response();
        }
    };

    let user_id = match UserId::parse(&req.user_id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse::bad_request("Invalid user ID format");
            return error.error_response();
        }
    };

    let input = AuthorizeInput {
        client_id,
        user_id,
        redirect_uri: req.redirect_uri.clone(),
        scope: req.scope.clone(),
        code_challenge: req.code_challenge.clone(),
        code_challenge_method: None, // Determine from code_challenge format
        state: req.state.clone(),
    };

    match state.auth_service.authorize(input).await {
        Ok(output) => {
            let response = ConsentResponse {
                code: output.authorization_code,
                state: req.state.clone(),
                redirect_uri: req.redirect_uri.clone(),
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            let error = match e {
                application::errors::UseCaseError::Validation(msg) => {
                    ErrorResponse::bad_request(&msg)
                }
                application::errors::UseCaseError::NotFound => {
                    ErrorResponse::not_found("Client or user not found")
                }
                application::errors::UseCaseError::Forbidden => {
                    ErrorResponse::forbidden("PKCE required for public clients")
                }
                _ => ErrorResponse::internal_error("Authorization failed"),
            };
            error.error_response()
        }
    }
}
