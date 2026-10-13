# Incoming quoted media replies

2026-10-13. **Implemented/offline.** Incoming ordinary media (image, voice
note, sticker, video, video note, document) whose context quotes another
message now projects as a normal media event with `quoted` set, instead of an
opaque `unsupported_message`. No live quoted media has been received since.

## Evidence (reused)

Comparison 2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`: media
messages carry `contextInfo` at field 17 (`BWp`/`BWZ`), whose `BWx` type has
`stanzaId`=1, `participant`=2, `quotedMessage`=3 and `expiration`=25. The
same `BWx` fields attribute text replies, which were observed live. The media
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
false), so edit behavior is unchanged. Outgoing media intents with a quote are
not sendable; the outbox patterns require `quoted: None`.

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

## Remaining

Live receipt of a quoted media reply; quoted media sending; quoted-parent
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
