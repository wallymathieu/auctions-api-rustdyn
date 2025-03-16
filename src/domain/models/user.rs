use core::fmt;

use serde::{Deserialize, Serialize};

use super::errors::Error;


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(String);

impl UserId {
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self(id.into())
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum User {
    BuyerOrSeller { id: UserId, name: Option<String> },
    Support { id: UserId },
}

impl User {
    pub fn new_buyer_or_seller<S: Into<String>>(id: UserId, name: Option<S>) -> Self {
        Self::BuyerOrSeller {
            id,
            name: name.map(|n| n.into()),
        }
    }

    pub fn new_support(id: UserId) -> Self {
        Self::Support { id }
    }

    pub fn id(&self) -> &UserId {
        match self {
            Self::BuyerOrSeller { id, .. } => id,
            Self::Support { id } => id,
        }
    }

    pub fn from_string(s: &str) -> Result<Self, Error> {
        let parts: Vec<&str> = s.split('|').collect();

        if parts.is_empty() || parts[0].is_empty() {
            return Err(Error::InvalidUser("Invalid user string format".to_string()));
        }

        match parts[0] {
            "BuyerOrSeller" => {
                if parts.len() < 2 {
                    return Err(Error::InvalidUser("Missing BuyerOrSeller ID".to_string()));
                }
                let id = UserId::new(parts[1]);
                let name = if parts.len() > 2 {
                    Some(parts[2].to_string())
                } else {
                    None
                };
                Ok(Self::new_buyer_or_seller(id, name))
            }
            "Support" => {
                if parts.len() < 2 {
                    return Err(Error::InvalidUser("Missing Support ID".to_string()));
                }
                Ok(Self::new_support(UserId::new(parts[1])))
            }
            _ => Err(Error::InvalidUser(format!(
                "Unknown user type: {}",
                parts[0]
            ))),
        }
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BuyerOrSeller { id, name } => {
                if let Some(name) = name {
                    write!(f, "BuyerOrSeller|{}|{}", id, name)
                } else {
                    write!(f, "BuyerOrSeller|{}", id)
                }
            }
            Self::Support { id } => write!(f, "Support|{}", id),
        }
    }
}

#[cfg(test)]
mod user_tests {
    use super::{User, UserId};
    fn match_buyer_or_seller(user: &User, id: &str, name: &str) {
        match user {
            User::BuyerOrSeller { id: user_id, name: user_name } => {
                assert_eq!(user_id.value(), id);
                if let Some(user_name_value) = user_name {
                    assert_eq!(user_name_value, name);
                } else {
                    panic!("Expected user name to be Some, but it was None")
                }
            }
            _ => panic!("Expected BuyerOrSeller"),
        }
    }
    #[test]
    fn test_create_buyer_or_seller() {
        let user_id = UserId::new("user123");
        let name = Some("John Doe");

        let user = User::new_buyer_or_seller(user_id.clone(), name);
        match_buyer_or_seller(
            &user,
            "user123",
            "John Doe");
    }

    #[test]
    fn test_create_support() {
        let user_id = UserId::new("support456");

        let user = User::new_support(user_id.clone());

        match user {
            User::Support { id } => {
                assert_eq!(id.value(), "support456");
            }
            _ => panic!("Expected Support"),
        }
    }

    #[test]
    fn test_user_from_string_buyer_or_seller() {
        let user_str = "BuyerOrSeller|user123|John Doe";
        let user = User::from_string(user_str).unwrap();

        match_buyer_or_seller(
            &user,
            "user123",
            "John Doe");
    }

    #[test]
    fn test_user_from_string_support() {
        let user_str = "Support|support456";
        let user = User::from_string(user_str).unwrap();

        match user {
            User::Support { id } => {
                assert_eq!(id.value(), "support456");
            }
            _ => panic!("Expected Support"),
        }
    }

    #[test]
    fn test_user_from_string_invalid() {
        let invalid_strs = ["", "Unknown|id", "BuyerOrSeller", "Support"];

        for str in invalid_strs {
            let result = User::from_string(str);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_user_display() {
        let user1 = User::new_buyer_or_seller(UserId::new("user123"), Some("John Doe"));
        assert_eq!(user1.to_string(), "BuyerOrSeller|user123|John Doe");

        let user2 = User::new_buyer_or_seller(UserId::new("user456"), None::<String>);
        assert_eq!(user2.to_string(), "BuyerOrSeller|user456");

        let user3 = User::new_support(UserId::new("support789"));
        assert_eq!(user3.to_string(), "Support|support789");
    }
}
