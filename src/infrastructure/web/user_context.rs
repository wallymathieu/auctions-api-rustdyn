pub mod jwt_payload_handling {
    use actix_web::HttpRequest;
    use base64::prelude::*;
    use serde::{Deserialize, Serialize};
    use crate::domain::models::UserId;

    const X_JWT_PAYLOAD: &str = "X-JWT-PAYLOAD";
    pub fn from_request(req: &HttpRequest) -> Option<UserId> {
        let user_id = req
            .headers()
            .get(X_JWT_PAYLOAD)
            .and_then(|header| header.to_str().ok())
            .and_then(|s| decode_jwt_payload(s).ok())
            .and_then(|payload| match payload.name {
                Some(name) => Some(UserId::new(name)),
                None => None,
            });
        user_id
    }
    pub fn decode_jwt_payload(payload: &str) -> Result<JwtPayload, Box<dyn std::error::Error>> {
        log::info!("Decoding JWT payload: {}", payload);
        let payload = BASE64_STANDARD.decode(payload)?;
        let payload = std::str::from_utf8(&payload)?;
        log::info!("Decoded from utf8: {}", payload);
        let payload: JwtPayload = serde_json::from_str(payload)?;
        Ok(payload)
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct JwtPayload {
        #[serde(rename = "sub")]
        pub sub: Option<String>,

        #[serde(rename = "name")]
        pub name: Option<String>,

        #[serde(rename = "u_typ")]
        pub u_typ: Option<String>,
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn get_token(sub: &str, email: &str) -> String {
            let json = format!(
                r#"{{
            "sub":"{}",
            "name":"{}",
            "u_typ":"0"
        }}"#,
                sub, email
            );
            BASE64_STANDARD.encode(json.as_bytes())
        }

        #[test]
        fn test_decode_jwt_payload() {
            let token = "eyJzdWIiOiJhMSIsICJuYW1lIjoiVGVzdCIsICJ1X3R5cCI6IjAifQo=";
            let payload = decode_jwt_payload(token).unwrap();
            assert_eq!(payload.sub, Some("a1".to_string()));
            assert_eq!(payload.name, Some("Test".to_string()));
            assert_eq!(payload.u_typ, Some("0".to_string()));
        }
        #[test]
        fn test_seller1() {
            let token = get_token("a1", "seller1@hotmail.com");
            let payload = decode_jwt_payload(&token).unwrap();
            assert_eq!(payload.sub, Some("a1".to_string()));
            assert_eq!(payload.name, Some("seller1@hotmail.com".to_string()));
            assert_eq!(payload.u_typ, Some("0".to_string()));
        }
        #[test]
        fn test_buyer1() {
            let token = get_token("a2", "buyer1@hotmail.com");
            let payload = decode_jwt_payload(&token).unwrap();
            assert_eq!(payload.sub, Some("a2".to_string()));
            assert_eq!(payload.name, Some("buyer1@hotmail.com".to_string()));
            assert_eq!(payload.u_typ, Some("0".to_string()));
        }
    }
}

mod claims_principal_handling {
    // Azure Entra ID claims principal handling
    use actix_web::HttpRequest;
    use base64::prelude::*;
    use serde::{Deserialize, Serialize};
    use crate::domain::models::UserId;

    const X_MS_CLIENT_PRINCIPAL: &str = "X-MS-CLIENT-PRINCIPAL";

    pub fn from_request(req: &HttpRequest) -> Option<UserId> {
        let user_id = req
            .headers()
            .get(X_MS_CLIENT_PRINCIPAL)
            .and_then(|header| header.to_str().ok())
            .and_then(|s| decode_jwt_payload(s).ok())
            .and_then(get_name_claim_value)
            .and_then(|name| Some(UserId::new(name)));
        user_id
    }

    pub fn get_name_claim_value(payload: ClientPrincipal) -> Option<String> {
        return payload
            .claims
            .map(|f| {
                f.iter()
                    .find(|c| c.typ == payload.name_claim_type)
                    .and_then(|c| c.val.clone())
            })
            .flatten();
    }

    pub fn decode_jwt_payload(
        payload: &str,
    ) -> Result<ClientPrincipal, Box<dyn std::error::Error>> {
        log::info!("Decoding JWT payload: {}", payload);
        let payload = BASE64_STANDARD.decode(payload)?;
        let payload = std::str::from_utf8(&payload)?;
        log::info!("Decoded from utf8: {}", payload);
        let payload: ClientPrincipal = serde_json::from_str(payload)?;
        Ok(payload)
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ClientPrincipalClaim {
        #[serde(rename = "typ")]
        pub typ: Option<String>,

        #[serde(rename = "val")]
        pub val: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ClientPrincipal {
        #[serde(rename = "auth_typ")]
        pub identity_provider: Option<String>,

        #[serde(rename = "name_typ")]
        pub name_claim_type: Option<String>,

        #[serde(rename = "role_typ")]
        pub role_claim_type: Option<String>,

        #[serde(rename = "claims")]
        pub claims: Option<Vec<ClientPrincipalClaim>>,
    }

    #[cfg(test)]
    mod client_principal_tests {
        use super::*;

        fn get_token(email: &str) -> String {
            let json = format!(
                r#"{{
    "auth_typ":"aad","claims":[
        {{"typ":"ver","val":"2.0"}},
        {{"typ":"iss","val":"https://login.microsoftonline.com/sdsdsd/v2.0"}},
        {{"typ":"http://schemas.xmlsoap.org/ws/2005/05/identity/claims/nameidentifier","val":"00000000-0000-0000-0000-000000000000"}},
        {{"typ":"aud","val":"868585685"}},
        {{"typ":"exp","val":"1681816329"}},
        {{"typ":"iat","val":"1681729629"}},
        {{"typ":"nbf","val":"1681729629"}},
        {{"typ":"name","val":"Oskar Gewalli"}},{{"typ":"preferred_username","val":"{}"}},
        {{"typ":"http://schemas.microsoft.com/identity/claims/objectidentifier","val":"00000000-0000-0000-0000-000000000000"}},
        {{"typ":"http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress","val":"{}"}},
        {{"typ":"http://schemas.microsoft.com/identity/claims/tenantid","val":"00000000-0000-0000-0000-000000000000"}},
        {{"typ":"c_hash","val":"-"}},
        {{"typ":"nonce","val":"_"}},
        {{"typ":"aio","val":"*"}}],
    "name_typ":"http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress",
    "role_typ":"http://schemas.microsoft.com/ws/2008/06/identity/claims/role"}}"#,
                email, email
            );
            BASE64_STANDARD.encode(json.as_bytes())
        }

        #[test]
        fn test_seller1() {
            let token = get_token("seller1@hotmail.com");
            let payload = get_name_claim_value(decode_jwt_payload(&token).unwrap());
            assert_eq!(payload, Some("seller1@hotmail.com".to_string()));
        }
        #[test]
        fn test_buyer1() {
            let token = get_token("buyer1@hotmail.com");
            let payload = get_name_claim_value(decode_jwt_payload(&token).unwrap());
            assert_eq!(payload, Some("buyer1@hotmail.com".to_string()));
        }
    }
}
