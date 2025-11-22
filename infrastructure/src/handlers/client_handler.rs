//! OAuth Client Management HTTP Handlers

use crate::app_state::AppState;
use crate::dtos::{ErrorResponse, GetClientResponse, RegisterClientRequest, RegisterClientResponse};
use actix_web::{web, HttpResponse, ResponseError};
use application::ports::input::{
    ClientUseCase, RegisterConfidentialClientInput, RegisterPublicClientInput,
};
use application::ports::{
    AccessTokenRepository, AuthCodeRepository, ClientRepository, RefreshTokenRepository,
    UserRepository,
};
use domain::value_objects::ClientId;

/// POST /clients/register
///
/// Register a new OAuth client (public or confidential based on client_secret)
pub async fn register_client<UR, CR, AR, ATR, RTR>(
    state: web::Data<AppState<UR, CR, AR, ATR, RTR>>,
    req: web::Json<RegisterClientRequest>,
) -> HttpResponse
where
    UR: UserRepository,
    CR: ClientRepository,
    AR: AuthCodeRepository,
    ATR: AccessTokenRepository,
    RTR: RefreshTokenRepository,
{
    if let Some(_secret) = &req.client_secret {
        // Confidential client
        let input = RegisterConfidentialClientInput {
            name: req.name.clone(),
            redirect_uris: req.redirect_uris.clone(),
            allowed_scopes: req.scopes.clone(),
        };

        match state
            .client_service
            .register_confidential_client(input)
            .await
        {
            Ok(output) => {
                let response = RegisterClientResponse {
                    client_id: output.client_id.to_string(),
                    client_type: "confidential".to_string(),
                    name: output.name,
                    redirect_uris: output.redirect_uris,
                    allowed_scopes: req.scopes.clone(),
                    client_secret: Some(output.client_secret),
                    created_at: chrono::Utc::now().to_rfc3339(),
                };
                HttpResponse::Created().json(response)
            }
            Err(e) => {
                let error = match e {
                    application::errors::UseCaseError::Validation(msg) => {
                        ErrorResponse::bad_request(&msg)
                    }
                    application::errors::UseCaseError::Domain(msg) => {
                        ErrorResponse::conflict(&msg)
                    }
                    _ => ErrorResponse::internal_error("Failed to register client"),
                };
                error.error_response()
            }
        }
    } else {
        // Public client
        let input = RegisterPublicClientInput {
            name: req.name.clone(),
            redirect_uris: req.redirect_uris.clone(),
            allowed_scopes: req.scopes.clone(),
        };

        match state.client_service.register_public_client(input).await {
            Ok(output) => {
                let response = RegisterClientResponse {
                    client_id: output.client_id.to_string(),
                    client_type: "public".to_string(),
                    name: output.name,
                    redirect_uris: output.redirect_uris,
                    allowed_scopes: req.scopes.clone(),
                    client_secret: None,
                    created_at: chrono::Utc::now().to_rfc3339(),
                };
                HttpResponse::Created().json(response)
            }
            Err(e) => {
                let error = match e {
                    application::errors::UseCaseError::Validation(msg) => {
                        ErrorResponse::bad_request(&msg)
                    }
                    application::errors::UseCaseError::Domain(msg) => {
                        ErrorResponse::conflict(&msg)
                    }
                    _ => ErrorResponse::internal_error("Failed to register client"),
                };
                error.error_response()
            }
        }
    }
}

/// GET /clients/{id}
///
/// Get client information by ID
pub async fn get_client<UR, CR, AR, ATR, RTR>(
    state: web::Data<AppState<UR, CR, AR, ATR, RTR>>,
    client_id_str: web::Path<String>,
) -> HttpResponse
where
    UR: UserRepository,
    CR: ClientRepository,
    AR: AuthCodeRepository,
    ATR: AccessTokenRepository,
    RTR: RefreshTokenRepository,
{
    let client_id = match ClientId::parse(&client_id_str.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse::bad_request("Invalid client ID format");
            return error.error_response();
        }
    };

    match state.client_service.get_public_client(client_id).await {
        Ok(client) => {
            let response = GetClientResponse {
                client_id: client.id().to_string(),
                client_type: "public".to_string(),
                name: client.name().to_string(),
                redirect_uris: client.redirect_uris().to_vec(),
                allowed_scopes: client.allowed_scopes().to_string(),
                created_at: client.created_at().to_rfc3339(),
            };
            HttpResponse::Ok().json(response)
        }
        Err(_) => {
            // Try confidential client
            match state.client_service.get_confidential_client(client_id).await {
                Ok(client) => {
                    let response = GetClientResponse {
                        client_id: client.id().to_string(),
                        client_type: "confidential".to_string(),
                        name: client.name().to_string(),
                        redirect_uris: client.redirect_uris().to_vec(),
                        allowed_scopes: client.allowed_scopes().to_string(),
                        created_at: client.created_at().to_rfc3339(),
                    };
                    HttpResponse::Ok().json(response)
                }
                Err(e) => {
                    let error = match e {
                        application::errors::UseCaseError::NotFound => {
                            ErrorResponse::not_found("Client not found")
                        }
                        application::errors::UseCaseError::Validation(msg) => {
                            ErrorResponse::bad_request(&msg)
                        }
                        _ => ErrorResponse::internal_error("Failed to get client"),
                    };
                    error.error_response()
                }
            }
        }
    }
}

/// DELETE /clients/{id}
///
/// Delete an OAuth client
pub async fn delete_client<UR, CR, AR, ATR, RTR>(
    state: web::Data<AppState<UR, CR, AR, ATR, RTR>>,
    client_id_str: web::Path<String>,
) -> HttpResponse
where
    UR: UserRepository,
    CR: ClientRepository,
    AR: AuthCodeRepository,
    ATR: AccessTokenRepository,
    RTR: RefreshTokenRepository,
{
    let client_id = match ClientId::parse(&client_id_str.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse::bad_request("Invalid client ID format");
            return error.error_response();
        }
    };

    match state.client_service.delete_client(client_id).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => {
            let error = match e {
                application::errors::UseCaseError::NotFound => {
                    ErrorResponse::not_found("Client not found")
                }
                _ => ErrorResponse::internal_error("Failed to delete client"),
            };
            error.error_response()
        }
    }
}
