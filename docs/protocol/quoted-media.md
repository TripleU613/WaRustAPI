# Incoming quoted media replies

2026-10-13. **Implemented/offline.** Incoming ordinary media (image, voice
note, sticker, video, video note, document) whose context quotes another
message now projects as a normal media event with `quoted` set, instead of an
opaque `unsupported_message`. No live quoted media has been received since.

## Evidence (reused)

Comparison 2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`: media
messages carry `contextInfo` at field 17, whose ContextInfo type has
`stanzaId`=1, `participant`=2, `quotedMessage`=3 and `expiration`=25. The
same ContextInfo fields attribute text replies, which were observed live.
**Class names (corrected 2026-10-13):** in 2.26.37.71 the image and audio
classes are `BZD` and `BZ0`, and both retained descriptors list
`contextInfo_` of type `BZL`
([image](rea/2.26.37.71-own-device-media/image-descriptor.json),
[audio](rea/2.26.37.71-own-device-media/audio-descriptor.json)). The names
`BWp`/`BWZ`/`BWx` in older comments come from another comparison release
([own-device media](own-device-media.md)); the field numbers are unchanged. The media
context and expiry handling are described in
[messaging-completion.md](../messaging-completion.md) (media context follow-up).
No new APK trace was made; this extends the existing text rule to the same
context type on media.

## Behavior

`media::event_content(.., allow_quote)` keeps refusing a context that carries
`quotedMessage` without `stanzaId` (it names no target), exactly as text does.
With a stanza ID, projection produces the media content and the shared
`inbox_events::quote_reference` attributes the quote: a participant naming one
of our login identities means our outgoing message; otherwise the participant
is the quoted author (often a third member in a group), falling back to the
stanza sender when absent or unparseable. An unrepresentable stanza ID leaves
the event opaque. The quote is a reference only: the quoted body is not
stored, verified against a saved parent, or fetched.

Own-device synchronized media uses the same projection key as its text path
for attribution and the original record key for sealing the media reference.
Caption-edit parent resolution still refuses quoted parents (`allow_quote`
false), so edit behavior is unchanged. Outgoing quoted media became sendable
the same day ([sending](#sending-2026-10-13)).

`MessageContent::Media` gains `quoted`, serialized only when present, so
existing journal records, saved intents and outbox semantic hashes are
unchanged. Expiry, view-once exclusion and retrieval checks are unchanged.

## Validation

`quoted_media_replies_project_with_text_reply_attribution` publishes five
images through the normal inbox event path: direct with no participant
(incoming, sender's own), participant naming our PN (outgoing), a group quote
of a third member (incoming, that member), `quotedMessage` without stanza ID
(opaque) and an empty stanza ID (opaque). Existing media tests now assert that
a stanza-less quote refuses even when quotes are allowed and that edit-parent
resolution still refuses quoted documents.

## Sending (2026-10-13)

**Implemented/offline.** An uploaded image or voice note can now reply to a
message: `MessageContent::Media { quoted: Some(..) }` in the direct and normal
group outbox. `quotes::media_reply_payload` inserts `contextInfo` (17) into the
encoded `imageMessage` (3) or `audioMessage` (8) with the same `BWx`
`stanzaId`/`participant`/`quotedMessage` a text reply carries, copying every
other field byte for byte. The attribution is the existing text rule:

- direct: `participant` is the contact's LID (or phone number) for their
  message and our LID (or phone number) for ours; `quotedMessage` is the saved
  incoming payload with `messageContextInfo` stripped, our own text intent, or
  an empty `conversation` (shared `quote_for`);
- group: the guarded reference path (`group_references`), so the parent and its
  author are re-checked in the reservation transaction (`reference_changed`
  blocks), and `quotedMessage` is the saved parent text only.

**Evidence:** the field placement is the one the receive path already reads
from the 2.26.37.71 descriptors (`BZD`/`BZ0` `contextInfo_`=17 of type `BZL`;
see [Evidence](#evidence-reused)). No native sender for quoted media was
traced in this session (no APK in the container); the encoding mirrors the
live-verified text reply, not a traced media-reply producer. **Not
established:** whether the native client adds other context fields (for
example `expiration` in a disappearing chat, which no outgoing message of ours
carries yet) or a quoted-media `quotedMessage` with the parent's media fields.

**Refused locally:** a view-once payload (wrapper 55; its quote semantics are
untraced), a payload without exactly one image/audio field, a media message
that already has a context, captions and file names (unchanged), and a quote
of another chat (outbox validation). Direct quotes (text and media) are built
at selection exactly as preparation builds them, before any key query is
reserved: a wrong-chat or ambiguous saved parent, a missing own identity, an
unsupported media shape, or a payload over the 32 KiB direct bound
(`direct::prepare_payload`; the quote copies the whole saved parent) marks the
intent failed locally (`unbuildable_quotes_fail_locally_before_any_query`).
Before the review fix only the shape was checked there, so the other cases
failed after the key query and ended the session with the intent stranded. In
groups an unsupported shape blocks the preparation as unsupported content.

**Exposure:** library `send_message`; `outbox-send` group JSON replies
(`quote` without `emoji`/`quoted_text`) with exactly one of `text`, `image` or
`voice_note`. The `reply` command stays text-only. (Later the same day, direct
private JSON replies were added to `outbox-send`: the author must be the contact,
in the same form as `to`, or one of our identities; the comparison path refuses
any direct quote via `require_comparison_support`.) The same change moved the comparison path's group text-only
refusal out of the shared JSON reader: it had also blocked the documented
`outbox-send` group image/voice note path, which was therefore unreachable from
the CLI. **Correction (review, same day):** as first committed, the reader
still required `text` on every reply, so image/voice-note replies were refused
by `MessageRequest::read` and only reachable through the library; the CLI test
had built the request with serde and bypassed the reader. A reply now needs
exactly one body, tested through `read()`. JSON reactions and `quoted_text`
are refused before the store opens and before the request ID is claimed for
the outbox path, so the same JSON can still go to `lab-transport`.

**Re-running the same JSON:** whether `participant` names our own message is
decided from the current own PN/LID. If our LID is learned between two runs,
the same reply JSON maps to a different target direction and `outbox-send`
refuses with an idempotency conflict instead of reporting the saved result.
This fails safe (nothing is resent); reuse the original saved outcome through
`status`.

**Tests:** `media_reply_context_round_trips_through_the_receive_parser`
(image/audio placement read back by the receive parser) and
`media_reply_payload_keeps_fields_and_refuses_ambiguous_shapes` (byte layout,
refusals);
`quoted_uploaded_media_carries_the_reply_context_to_the_peer` (outbox to a
decrypted peer plaintext, incoming and outgoing parents);
the `quoted` case of `uploaded_group_media_is_chat_bound_and_late_crypto_changes_roll_back_reservation`
(guarded group payload, sent as `mediatype=image`);
`group_json_replies_map_authors_and_refuse_comparison_only_quotes` (CLI
mapping, early refusals and request-ID reuse);
`a_group_quote_is_a_reply_or_a_reaction_but_not_both` (media replies through
the JSON reader). No live quoted media was sent.

## Remaining

Live receipt of a quoted media reply; a live quoted media send; quoted-parent
caption edits; verifying the quoted body against a saved parent.

**Known gap (review, 2026-10-13): quotes of another chat.** `quote_reference`
reads only `stanzaId`/`participant`; ContextInfo's `remoteJid` is never read. A
reply to a status (or any quote whose target lives in another chat) is
therefore projected as a reference into the current chat, under the status's
stanza ID: it points to nothing or to an unrelated message. Text replies have
always behaved this way; this change extends it to media. Not fixed because the
field number is **unverified here**: field 4 comes from general protocol
knowledge, and no committed 2.26.37.71 `BWx` dump records it. The fix, once
traced, is to leave the quote unattributed (or attribute it to that chat) when
`remoteJid` names a different chat.
