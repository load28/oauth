//! OAuth Token HTTP Handlers

use crate::app_state::AppState;
use crate::dtos::{
    ErrorResponse, IntrospectTokenRequest, IntrospectTokenResponse, RevokeTokenRequest,
    RevokeTokenResponse, TokenRequest, TokenResponse,
};
use crate::jwt::{decode_jwt, encode_jwt};
use actix_web::{web, HttpResponse, ResponseError};
use application::ports::input::{ExchangeCodeInput, TokenUseCase};
use application::ports::{
    AccessTokenRepository, AuthCodeRepository, ClientRepository, RefreshTokenRepository,
    UserRepository,
};
use domain::value_objects::{ClientId, Scopes, UserId};

/// POST /token
///
/// OAuth 2.0 token endpoint - exchange authorization code for access/refresh tokens
pub async fn token<UR, CR, AR, ATR, RTR>(
    state: web::Data<AppState<UR, CR, AR, ATR, RTR>>,
    req: web::Json<TokenRequest>,
) -> HttpResponse
where
    UR: UserRepository,
    CR: ClientRepository,
    AR: AuthCodeRepository,
    ATR: AccessTokenRepository,
    RTR: RefreshTokenRepository,
{
    // Validate grant type
    if req.grant_type != "authorization_code" {
        let error =
            ErrorResponse::bad_request("Unsupported grant_type (must be 'authorization_code')");
        return error.error_response();
    }

    let client_id = match ClientId::parse(&req.client_id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse::bad_request("Invalid client ID format");
            return error.error_response();
        }
    };

    let input = ExchangeCodeInput {
        code: req.code.clone(),
        client_id,
        client_secret: req.client_secret.clone(),
        redirect_uri: req.redirect_uri.clone(),
        code_verifier: req.code_verifier.clone(),
    };

    match state.token_service.exchange_code(input).await {
        Ok(output) => {
            // Generate JWT access token
            let user_id = match UserId::parse(&output.access_token) {
                Ok(id) => id,
                Err(_) => {
                    // Fallback: create new user ID (this shouldn't happen)
                    UserId::new()
                }
            };

            let scopes = match Scopes::parse(&output.scope) {
                Ok(s) => s,
                Err(_) => {
                    let error = ErrorResponse::internal_error("Invalid scopes");
                    return error.error_response();
                }
            };

            let access_token_jwt = match encode_jwt(
                &crate::jwt::Claims::new(
                    user_id,
                    client_id,
                    scopes,
                    state.config.token.access_token_expiration,
                ),
                state.config.jwt.secret_bytes(),
            ) {
                Ok(token) => token,
                Err(_) => {
                    let error = ErrorResponse::internal_error("Failed to generate access token");
                    return error.error_response();
                }
            };

            let response = TokenResponse {
                access_token: access_token_jwt,
                token_type: "Bearer".to_string(),
                expires_in: output.expires_in,
                refresh_token: output.refresh_token.unwrap_or_default(),
                scope: output.scope,
            };

            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            let error = match e {
                application::errors::UseCaseError::Validation(msg) => {
                    ErrorResponse::bad_request(&msg)
                }
                application::errors::UseCaseError::NotFound => {
                    ErrorResponse::not_found("Authorization code not found or expired")
                }
                application::errors::UseCaseError::Unauthorized => {
                    ErrorResponse::unauthorized("Invalid client credentials or code verifier")
                }
                _ => ErrorResponse::internal_error("Token exchange failed"),
            };
            error.error_response()
        }
    }
}

/// POST /revoke
///
/// OAuth 2.0 token revocation endpoint (RFC 7009)
pub async fn revoke_token<UR, CR, AR, ATR, RTR>(
    _state: web::Data<AppState<UR, CR, AR, ATR, RTR>>,
    _req: web::Json<RevokeTokenRequest>,
) -> HttpResponse
where
    UR: UserRepository,
    CR: ClientRepository,
    AR: AuthCodeRepository,
    ATR: AccessTokenRepository,
    RTR: RefreshTokenRepository,
{
    // TODO: Implement token revocation logic
    // For JWT tokens, revocation is challenging (typically use short expiration instead)
    // For refresh tokens, we can mark them as revoked in the database

    let response = RevokeTokenResponse {
        success: true,
        message: "Token revocation not yet implemented (use short expiration)".to_string(),
    };

    HttpResponse::Ok().json(response)
}

/// POST /introspect
///
/// OAuth 2.0 token introspection endpoint (RFC 7662)
pub async fn introspect_token<UR, CR, AR, ATR, RTR>(
    state: web::Data<AppState<UR, CR, AR, ATR, RTR>>,
    req: web::Json<IntrospectTokenRequest>,
) -> HttpResponse
where
    UR: UserRepository,
    CR: ClientRepository,
    AR: AuthCodeRepository,
    ATR: AccessTokenRepository,
    RTR: RefreshTokenRepository,
{
    // Decode JWT token
    match decode_jwt(&req.token, state.config.jwt.secret_bytes(), None) {
        Ok(claims) => {
            let response = IntrospectTokenResponse {
                active: true,
                scope: Some(claims.scope),
                client_id: Some(claims.aud),
                username: Some(claims.sub),
                token_type: Some("Bearer".to_string()),
                exp: Some(claims.exp),
                iat: Some(claims.iat),
            };
            HttpResponse::Ok().json(response)
        }
        Err(_) => {
            // Invalid or expired token
            let response = IntrospectTokenResponse {
                active: false,
                scope: None,
                client_id: None,
                username: None,
                token_type: None,
                exp: None,
                iat: None,
            };
            HttpResponse::Ok().json(response)
        }
    }
}
