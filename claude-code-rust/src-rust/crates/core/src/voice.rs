//! Voice mode availability checks

use crate::oauth::OAuthTokens;

#[derive(Debug, Clone, PartialEq)]
pub enum VoiceAvailability {
    Available,
    /// Not authenticated via first-party OAuth
    RequiresOAuth,
    /// OAuth token missing required scopes
    MissingScopes {
        required: Vec<String>,
        have: Vec<String>,
    },
    /// Feature disabled by kill-switch environment variable
    Disabled,
    /// Feature flag not enabled in this build
    NotEnabled,
}

/// Scopes required for voice mode to function
const VOICE_REQUIRED_SCOPES: &[&str] = &["user:inference", "user:profile"];

/// Environment variable that disables voice mode when set (any value)
const KILL_SWITCH_ENV: &str = "CLAUDE_CODE_VOICE_DISABLED";

/// Check whether voice mode is available given the current OAuth tokens.
///
/// Pass `None` when the user is not authenticated via OAuth (API-key-only auth).
pub fn check_voice_availability(tokens: Option<&OAuthTokens>) -> VoiceAvailability {
    // Check kill switch first — always wins
    if std::env::var(KILL_SWITCH_ENV).is_ok() {
        return VoiceAvailability::Disabled;
    }

    // Voice requires first-party OAuth; API key alone is not sufficient
    let tokens = match tokens {
        Some(t) => t,
        None => return VoiceAvailability::RequiresOAuth,
    };

    // OAuthTokens stores scopes as Vec<String>
    let have_scopes: &[String] = &tokens.scopes;

    let missing: Vec<String> = VOICE_REQUIRED_SCOPES
        .iter()
        .filter(|&&required| !have_scopes.iter().any(|h| h == required))
        .map(|s| s.to_string())
        .collect();

    if !missing.is_empty() {
        return VoiceAvailability::MissingScopes {
            required: VOICE_REQUIRED_SCOPES
                .iter()
                .map(|s| s.to_string())
                .collect(),
            have: have_scopes.to_vec(),
        };
    }

    VoiceAvailability::Available
}

impl VoiceAvailability {
    /// Returns `true` when voice mode can be started.
    pub fn is_available(&self) -> bool {
        matches!(self, VoiceAvailability::Available)
    }

    /// Returns a human-readable error message when voice is not available,
    /// or `None` when it is.
    pub fn error_message(&self) -> Option<String> {
        match self {
            VoiceAvailability::Available => None,
            VoiceAvailability::RequiresOAuth => Some(
                "Voice mode requires OAuth authentication. Run /login to authenticate.".to_string(),
            ),
            VoiceAvailability::MissingScopes { required, have } => Some(format!(
                "Voice mode requires scopes: {}. Your token has: {}",
                required.join(", "),
                if have.is_empty() {
                    "none".to_string()
                } else {
                    have.join(", ")
                }
            )),
            VoiceAvailability::Disabled => Some("Voice mode is currently disabled.".to_string()),
            VoiceAvailability::NotEnabled => {
                Some("Voice mode is not enabled in this build.".to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens_with_scopes(scopes: Vec<&str>) -> OAuthTokens {
        OAuthTokens {
            access_token: "test_token".to_string(),
            scopes: scopes.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        }
    }

    #[test]
    fn test_no_tokens_requires_oauth() {
        let result = check_voice_availability(None);
        assert_eq!(result, VoiceAvailability::RequiresOAuth);
        assert!(!result.is_available());
        assert!(result.error_message().is_some());
    }

    #[test]
    fn test_available_with_all_scopes() {
        let tokens = tokens_with_scopes(vec!["user:inference", "user:profile"]);
        let result = check_voice_availability(Some(&tokens));
        assert_eq!(result, VoiceAvailability::Available);
        assert!(result.is_available());
        assert!(result.error_message().is_none());
    }

    #[test]
    fn test_missing_one_scope() {
        let tokens = tokens_with_scopes(vec!["user:inference"]);
        let result = check_voice_availability(Some(&tokens));
        assert!(matches!(result, VoiceAvailability::MissingScopes { .. }));
        assert!(!result.is_available());
        let msg = result.error_message().unwrap();
        assert!(msg.contains("user:profile"));
    }

    #[test]
    fn test_missing_all_scopes() {
        let tokens = tokens_with_scopes(vec!["org:create_api_key"]);
        let result = check_voice_availability(Some(&tokens));
        assert!(matches!(result, VoiceAvailability::MissingScopes { .. }));
        assert!(!result.is_available());
    }

    #[test]
    fn test_empty_scopes_missing() {
        let tokens = tokens_with_scopes(vec![]);
        let result = check_voice_availability(Some(&tokens));
        assert!(
            matches!(result, VoiceAvailability::MissingScopes { ref have, .. } if have.is_empty())
        );
        let msg = result.error_message().unwrap();
        assert!(msg.contains("none"));
    }

    #[test]
    fn test_kill_switch_disables_voice() {
        // Temporarily set the kill-switch env var
        std::env::set_var(KILL_SWITCH_ENV, "1");
        let tokens = tokens_with_scopes(vec!["user:inference", "user:profile"]);
        let result = check_voice_availability(Some(&tokens));
        std::env::remove_var(KILL_SWITCH_ENV);
        assert_eq!(result, VoiceAvailability::Disabled);
        assert!(!result.is_available());
    }

    #[test]
    fn test_kill_switch_beats_no_auth() {
        std::env::set_var(KILL_SWITCH_ENV, "true");
        let result = check_voice_availability(None);
        std::env::remove_var(KILL_SWITCH_ENV);
        // Kill switch wins — returns Disabled, not RequiresOAuth
        assert_eq!(result, VoiceAvailability::Disabled);
    }

    #[test]
    fn test_not_enabled_error_message() {
        let v = VoiceAvailability::NotEnabled;
        assert!(!v.is_available());
        assert!(v.error_message().unwrap().contains("not enabled"));
    }

    #[test]
    fn test_extra_scopes_still_available() {
        // Having more scopes than required is fine
        let tokens = tokens_with_scopes(vec![
            "user:inference",
            "user:profile",
            "org:create_api_key",
            "user:file_upload",
        ]);
        let result = check_voice_availability(Some(&tokens));
        assert_eq!(result, VoiceAvailability::Available);
    }
}
