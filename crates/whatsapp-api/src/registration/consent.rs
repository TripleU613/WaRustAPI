//! Explicit comparison-build consent operations. Never selected by begin.

use super::*;

/// Private, explicit operator decision; IDs must match retained server context.
/// Result 5 is observed at Ird.invokeSuspend 0x0ffa (comparison 2.26.37.70).
/// This raw lab surface is not a replacement for presenting the consent UI.
#[derive(Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConsentDecision {
    /// Selector received from the server, not an invented default.
    pub consent_id: Option<i32>,
    /// Revision received from the server.
    pub consent_version: Option<i32>,
    /// Explicit observed result; unsupported values are rejected.
    pub consent_result: Option<i32>,
    /// User-supplied actual date of birth for the separate `context=dob` flow.
    /// Mutually exclusive with result selectors; never logged or persisted.
    pub dob: Option<Zeroizing<String>>,
    /// The operator has reviewed and chosen this consent result.
    pub confirmed: bool,
}

impl fmt::Debug for ConsentDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConsentDecision")
            .field("consent_id", &self.consent_id)
            .field("consent_version", &self.consent_version)
            .field("consent_result", &self.consent_result)
            .field("has_dob", &self.dob.is_some())
            .field("confirmed", &self.confirmed)
            .finish()
    }
}

fn valid_dob(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
    {
        return false;
    }
    let year: i64 = value[..4].parse().unwrap_or(0);
    let month: usize = value[5..7].parse().unwrap_or(0);
    let day: i64 = value[8..].parse().unwrap_or(0);
    if year == 0 || !(1..=12).contains(&month) {
        return false;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    if day < 1 || day > days[month - 1] {
        return false;
    }
    let previous = year - 1;
    let epoch_days = previous * 365 + previous / 4 - previous / 100
        + previous / 400
        + days[..month - 1].iter().sum::<i64>()
        + day
        - 1
        - 719_162;
    epoch_days <= Timestamp::now().as_millis().div_euclid(86_400_000)
}

impl ConsentDecision {
    /// Reject absent confirmation and unobserved selector/result shapes.
    pub fn validate(&self) -> Result<()> {
        let valid = if let Some(dob) = &self.dob {
            self.consent_id.is_none()
                && self.consent_version.is_none()
                && self.consent_result.is_none()
                && valid_dob(dob)
        } else {
            self.consent_id.is_some_and(|v| v >= 0)
                && self.consent_version.is_some_and(|v| v >= 0)
                && self.consent_result == Some(5)
        };
        if !self.confirmed || !valid {
            return Err(Error::invalid(
                "consent",
                "requires confirmed real DOB (YYYY-MM-DD) or reviewed server selectors with observed result 5; do not mix flows",
            ));
        }
        Ok(())
    }
}

impl RegistrationService {
    /// One reserved existence inspection or explicit consent submission, on a
    /// saved consent refusal only. Neither action sends SMS nor reuses an OTP.
    pub async fn continue_consent(
        &self,
        phone: PhoneNumber,
        decision: Option<ConsentDecision>,
    ) -> Result<RegistrationStep> {
        if self.profile_id != request::LAB_PROFILE_ID {
            return Err(Error::invalid(
                "consent",
                "requires the explicit comparison lab profile",
            ));
        }
        if let Some(value) = &decision {
            value.validate()?;
        }
        let selected = decision.clone();
        let profile = self.profile_id.clone();
        let attempt = self.store.run_blocking(move |store| store.write(|tx| {
            let mut attempt = Self::load_current(tx)?.ok_or_else(|| Error::invalid("consent", "no saved attempt"))?;
            if attempt.generation != tx.generation() || attempt.profile_id != profile || attempt.phone() != &phone {
                return Err(Error::invalid("consent", "saved attempt identity does not match"));
            }
            match &attempt.state {
                RegistrationState::Blocked { reason: BlockedReason::ConsentRequired { details, retry_after, sms_retry_after, voice_retry_after } } => {
                    if let Some(until) = [*retry_after, *sms_retry_after, *voice_retry_after].into_iter().flatten().max()
                        && Timestamp::now() < until {
                        return Err(Error::invalid("consent", "saved server wait has not expired"));
                    }
                    if let Some(value) = &selected
                        && (details.consent_id != value.consent_id || details.consent_version != value.consent_version || details.parent_consent_url.is_some()) {
                            return Err(Error::invalid("consent", "decision must match saved selectors; parent flow requires separate handling"));
                    }
                    if let Some(value) = &selected {
                        if !details.context_captured {
                            return Err(Error::invalid("consent", "saved parser did not capture pending selector; inspect context first"));
                        }
                        let pending = details.pending.as_deref().map(|v| v.as_str());
                        let allowed = if value.dob.is_some() {
                            pending.is_none() || pending.is_some_and(|v| v.eq_ignore_ascii_case("dob"))
                        } else {
                            pending.is_some_and(|v| v.eq_ignore_ascii_case("youth_consent"))
                        };
                        if !allowed { return Err(Error::invalid("consent", "decision does not match the APK pending branch")); }
                    }
                }
                RegistrationState::Blocked { reason: BlockedReason::ServerRefused { http_status: Some(200), reason: Some(wire::RegistrationResponseReason::Consent), .. } } if selected.is_none() => {}
                _ => return Err(Error::invalid("consent", "requires saved consent context; inspect legacy context before submitting")),
            }
            // v2 captures the APK's pending selector, omitted by the first
            // inspection parser. The old reservation remains untouched.
            let latch: &'static [u8] = if selected.is_some() { b"consent-submit-used" } else { b"consent-inspect-context-v2-used" };
            if tx.get(Table::RegistrationAttempts, latch, RECORD_V1)?.is_some() {
                return Err(Error::invalid("consent", "operation already reserved; no resend"));
            }
            let step = u32::try_from(attempt.history_len()).map_err(|_| Error::invalid("consent", "history overflow"))?;
            let context = attempt.clone();
            attempt.transition(RegistrationState::Verifying { step }, Timestamp::now())?;
            tx.put_json(Table::RegistrationAttempts, latch, RECORD_V1, &attempt.id)?;
            Self::save_current(tx, &attempt)?;
            Ok((context, latch))
        })).await?;
        let (attempt, latch) = attempt;
        match self.flow.consent(&attempt, decision.as_ref()).await {
            Ok(output) => {
                self.finish_transition(attempt.id, attempt.generation, "verifying", output)
                    .await
            }
            Err(error) => {
                // `attempt` is the pre-reservation copy, restored exactly if
                // the request provably did not leave.
                self.finish_failed(
                    &attempt,
                    "verifying",
                    RegistrationState::RegistrationOutcomeUnknown { step: 0 },
                    attempt.clone(),
                    // Consent prepares nothing, so its latch can go alone.
                    Undo {
                        latch: Some(latch),
                        material: None,
                    },
                    error,
                )
                .await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_date_input_is_validated_and_redacted() {
        for value in ["2000-02-29", "1999-12-31"] {
            assert!(valid_dob(value));
        }
        for value in [
            "1900-02-29",
            "2001-02-29",
            "2000-13-01",
            "2000-01-00",
            "9999-01-01",
            "2000",
            "0000-01-01",
            "2000-1-01",
        ] {
            assert!(!valid_dob(value));
        }
        let mut decision: ConsentDecision =
            serde_json::from_str(r#"{"dob":"2000-02-29","confirmed":true}"#).unwrap();
        assert!(decision.validate().is_ok());
        assert!(!format!("{decision:?}").contains("2000-02-29"));
        decision.consent_result = Some(5);
        assert!(decision.validate().is_err());
    }
}
