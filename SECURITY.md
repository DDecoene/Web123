# Security Policy

## Reporting a vulnerability

Please report security issues privately via
[GitHub's private vulnerability reporting](https://github.com/DDecoene/Web123/security/advisories/new)
— do not open a public GitHub issue for them. You'll get a reply as soon as possible,
and a fix will be released before the issue is disclosed.

## Security model

Web123 has no server and no account system. A worksheet is shared as a link
encoding a document ID and an encryption key; anyone with that link can open
the worksheet, and whether they can only view/compute locally or also
contribute edits back depends on whether the link was shared as read-only or
editable. As with any capability-style link, anyone who obtains it (or
guesses it — the ID and key are long enough to make guessing infeasible, not
enough to make the link itself un-sensitive) can access the worksheet. Don't
share a worksheet link anywhere you wouldn't share its contents.

Peer-to-peer sync uses a public, third-party signaling service only to
establish the initial connection between two browsers (a WebRTC handshake).
That service never receives the worksheet's document ID, encryption key, or
content — the sharing link is carried in the URL fragment, which browsers
never send in network requests.

There is no central copy of a worksheet anywhere. If every peer holding a
document becomes unreachable (all devices offline, or local browser storage
cleared) with no exported backup taken, the document cannot be recovered.
This is a deliberate trade-off of the peer-to-peer model, not a bug — export
a worksheet to a file if you need a durable backup.
