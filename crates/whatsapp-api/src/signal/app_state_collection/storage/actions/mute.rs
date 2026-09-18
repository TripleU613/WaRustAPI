//! Comparison 289.A0E / 28A: authenticated mute projection, not OS notification policy.
use super::*;

#[derive(Message)]
struct Value {
    #[prost(int64, optional, tag = "1")]
    timestamp: Option<i64>,
    #[prost(bytes = "vec", optional, tag = "4")]
    mute: Option<Vec<u8>>,
}
#[derive(Message)]
struct Mute {
    #[prost(bool, optional, tag = "1")]
    muted: Option<bool>,
    #[prost(int64, optional, tag = "2")]
    end: Option<i64>,
    #[prost(bool, optional, tag = "3")]
    auto_muted: Option<bool>,
    #[prost(int64, optional, tag = "4")]
    everyone_end: Option<i64>,
}

/// Authenticated mute settings in the server's timestamp domain. This records
/// sync state; local clock conversion and notification suppression are separate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMute {
    /// Explicit mute flag, including false for an unmute action.
    pub muted: bool,
    /// APK-normalized end time: -1 means indefinite; invalid/missing becomes 0.
    pub end_timestamp: i64,
    /// Optional everyone-mention end time, retained without policy conversion.
    pub everyone_end_timestamp: Option<i64>,
    /// Wire metadata retained; comparison 289.A0E does not consume this flag.
    pub auto_muted: Option<bool>,
    /// Original action timestamp, without conversion.
    pub timestamp: i64,
}

/// Resolved mute state; sentinel -1 cannot be confused with an expired deadline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MuteDeadline {
    /// No active mute at the supplied clock sample.
    Unmuted,
    /// Explicit indefinite mute.
    Indefinite,
    /// Future deadline in local Unix milliseconds.
    Until(i64),
}

/// Local policy projection, not a persisted clock estimate or notification send.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedChatMute {
    /// Ordinary chat mute at the supplied clock sample.
    pub chat: MuteDeadline,
    /// None means leave the existing everyone-mention setting unchanged, not
    /// unmute. This occurs when policy is disabled or a muted action omits it.
    pub everyone_mentions: Option<MuteDeadline>,
}

impl ChatMute {
    /// Resolve comparison 289.A0U/08B.A06 timer behavior. Caller must supply
    /// contemporaneous local and estimated-server millisecond clocks plus the
    /// resolved everyone-mention policy. Never infer these from action values.
    /// No mutation of stored sync state, clocks or notification policy occurs.
    /// Unlike Java overflow, unrepresentable local deadlines fail explicitly.
    pub fn resolve(
        &self,
        local_now_ms: i64,
        server_now_ms: i64,
        everyone_mentions_enabled: bool,
    ) -> Result<ResolvedChatMute> {
        if local_now_ms < 0 || server_now_ms < 0 {
            return Err(invalid("invalid mute clock sample"));
        }
        let deadline = |server_end: i64| -> Result<MuteDeadline> {
            if server_end == -1 {
                return Ok(MuteDeadline::Indefinite);
            }
            if server_end <= 0 || server_end <= server_now_ms {
                return Ok(MuteDeadline::Unmuted);
            }
            let local_end =
                i128::from(server_end) + i128::from(local_now_ms) - i128::from(server_now_ms);
            let local_end = i64::try_from(local_end)
                .map_err(|_| invalid("mute deadline outside local clock range"))?;
            Ok(MuteDeadline::Until(local_end))
        };
        let chat = if self.muted {
            deadline(self.end_timestamp)?
        } else {
            MuteDeadline::Unmuted
        };
        let everyone_mentions = if !everyone_mentions_enabled {
            None
        } else if let Some(end) = self.everyone_end_timestamp {
            Some(deadline(end)?)
        } else if self.muted {
            None
        } else {
            Some(MuteDeadline::Unmuted)
        };
        Ok(ResolvedChatMute {
            chat,
            everyone_mentions,
        })
    }
}

fn key(chat: &str) -> Vec<u8> {
    format!(
        "app-state-chat-mute:{}",
        hex::encode(Sha256::digest(chat.as_bytes()))
    )
    .into_bytes()
}

/// Read encrypted local mute projection without activating notification policy.
pub fn chat_mute(tx: &ReadTx<'_>, chat: &Jid) -> Result<Option<ChatMute>> {
    ChatId::from_jid(chat)?;
    tx.get_json(Table::Checkpoints, &key(&chat.to_string()), 1)
}

pub(super) fn decode(action: &PendingAction) -> Result<(String, ChatMute)> {
    if action.operation != 1 || action.schema_version != 2 {
        return Err(invalid(
            "unsupported mute operation or schema; batch remains pending",
        ));
    }
    let index: Vec<String> =
        serde_json::from_slice(&action.index).map_err(|_| invalid("invalid mute index"))?;
    if index.len() != 2 || index[0] != "mute" {
        return Err(invalid("invalid mute index"));
    }
    let chat = Jid::parse(&index[1]).map_err(|_| invalid("invalid mute chat"))?;
    ChatId::from_jid(&chat).map_err(|_| invalid("unsupported mute chat"))?;
    let bytes = action
        .value
        .as_deref()
        .ok_or_else(|| invalid("missing mute action"))?;
    only_fields(bytes, &[1, 4])?;
    let value = Value::decode(bytes).map_err(|_| invalid("invalid mute action"))?;
    let nested = value
        .mute
        .as_deref()
        .ok_or_else(|| invalid("missing mute action"))?;
    only_fields(nested, &[1, 2, 3, 4])?;
    let mute = Mute::decode(nested).map_err(|_| invalid("invalid mute fields"))?;
    let muted = mute.muted.ok_or_else(|| invalid("missing muted boolean"))?;
    let end_timestamp = if muted {
        mute.end.filter(|v| *v >= -1).unwrap_or(0)
    } else {
        0
    };
    Ok((
        chat.to_string(),
        ChatMute {
            muted,
            end_timestamp,
            everyone_end_timestamp: mute.everyone_end,
            auto_muted: mute.auto_muted,
            timestamp: value
                .timestamp
                .ok_or_else(|| invalid("missing mute timestamp"))?,
        },
    ))
}

pub(super) fn apply(tx: &mut WriteTx<'_>, chat: &str, mute: ChatMute) -> Result<()> {
    let key = key(chat);
    if let Some(old) = tx.get_json::<ChatMute>(Table::Checkpoints, &key, 1)? {
        // Same conservative local policy as pin projection, not the complete
        // APK's cross-collection/local-action conflict resolution.
        if mute.timestamp < old.timestamp || (mute.timestamp == old.timestamp && mute != old) {
            return Err(invalid("mute timestamp conflict; batch remains pending"));
        }
    }
    tx.put_json(Table::Checkpoints, &key, 1, &mute)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resolves_clock_skew_expiry_and_independent_mention_policy() {
        let mut mute = ChatMute {
            muted: true,
            end_timestamp: 12_000,
            everyone_end_timestamp: None,
            auto_muted: Some(true),
            timestamp: 9,
        };
        // These are independently calculated clock examples, not round trips.
        for (local, server, expected) in [
            (10_000, 11_000, MuteDeadline::Until(11_000)),
            (11_000, 10_000, MuteDeadline::Until(13_000)),
            (10_000, 12_000, MuteDeadline::Unmuted),
            (10_000, 13_000, MuteDeadline::Unmuted),
        ] {
            let result = mute.resolve(local, server, true).unwrap();
            assert_eq!(result.chat, expected);
            assert_eq!(result.everyone_mentions, None);
        }
        mute.end_timestamp = -1;
        mute.everyone_end_timestamp = Some(12_000);
        let result = mute.resolve(10_000, 11_000, true).unwrap();
        assert_eq!(result.chat, MuteDeadline::Indefinite);
        assert_eq!(result.everyone_mentions, Some(MuteDeadline::Until(11_000)));
        assert_eq!(
            mute.resolve(10_000, 11_000, false)
                .unwrap()
                .everyone_mentions,
            None
        );
        for (end, expected) in [
            (-1, MuteDeadline::Indefinite),
            (-2, MuteDeadline::Unmuted),
            (0, MuteDeadline::Unmuted),
            (11_000, MuteDeadline::Unmuted),
        ] {
            mute.everyone_end_timestamp = Some(end);
            assert_eq!(
                mute.resolve(10_000, 11_000, true)
                    .unwrap()
                    .everyone_mentions,
                Some(expected)
            );
        }
        mute.muted = false;
        mute.everyone_end_timestamp = Some(-1);
        let result = mute.resolve(10_000, 11_000, true).unwrap();
        assert_eq!(result.chat, MuteDeadline::Unmuted);
        assert_eq!(result.everyone_mentions, Some(MuteDeadline::Indefinite));
        mute.everyone_end_timestamp = None;
        assert_eq!(
            mute.resolve(10_000, 11_000, true)
                .unwrap()
                .everyone_mentions,
            Some(MuteDeadline::Unmuted)
        );
        assert!(mute.resolve(-1, 0, true).is_err());
        assert!(mute.resolve(0, -1, true).is_err());
        mute.muted = true;
        mute.end_timestamp = i64::MAX;
        assert!(mute.resolve(2, 1, true).is_err());
        assert_eq!(
            mute.resolve(i64::MAX, i64::MAX, true).unwrap().chat,
            MuteDeadline::Unmuted
        );
        // Resolution does not rewrite the authenticated server-domain value.
        assert_eq!(mute.end_timestamp, i64::MAX);
    }
    fn action(nested: &[u8]) -> PendingAction {
        let mut value = vec![8, 9, 0x22, nested.len() as u8];
        value.extend_from_slice(nested);
        PendingAction {
            operation: 1,
            index: br#"["mute","15550000000@s.whatsapp.net"]"#.to_vec(),
            value: Some(value),
            schema_version: 2,
        }
    }
    #[test]
    fn literal_mute_presence_timers_and_unsupported_fields() {
        for (wire, muted, end) in [
            (vec![8, 1, 16, 120], true, 120),
            (vec![8, 0, 16, 120], false, 0),
            (vec![8, 1], true, 0),
            (
                vec![8, 1, 16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1],
                true,
                -1,
            ),
            (
                vec![8, 1, 16, 254, 255, 255, 255, 255, 255, 255, 255, 255, 1],
                true,
                0,
            ),
        ] {
            let result = decode(&action(&wire)).unwrap().1;
            assert_eq!((result.muted, result.end_timestamp), (muted, end));
        }
        let result = decode(&action(&[8, 1, 24, 1, 32, 120])).unwrap().1;
        assert_eq!(result.auto_muted, Some(true));
        assert_eq!(result.everyone_end_timestamp, Some(120));
        for wire in [vec![], vec![16, 1], vec![8, 1, 8, 0], vec![8, 1, 40, 1]] {
            assert!(decode(&action(&wire)).is_err());
        }
        let mut invalid = action(&[8, 1]);
        invalid.schema_version = 5;
        assert!(decode(&invalid).is_err());
        invalid.schema_version = 2;
        invalid.operation = 2;
        assert!(decode(&invalid).is_err());
        invalid.operation = 1;
        invalid.value = Some(vec![0x22, 2, 8, 1]);
        assert!(decode(&invalid).is_err());
    }
}
