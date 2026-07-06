# Security Policy

## Reporting a vulnerability

Please report security issues privately via
[GitHub's private vulnerability reporting](https://github.com/DDecoene/Web123/security/advisories/new)
— do not open a public GitHub issue for them. You'll get a reply as soon as possible,
and a fix will be released before the issue is disclosed.

## Security model

Web123 is **unauthenticated and private-by-URL**: a worksheet's UUID in the URL
is its only access control. Anyone who has (or guesses) a worksheet URL can read and
edit that worksheet. Do not host an instance on the public internet with sensitive
content, and do not share worksheet URLs you want to keep private.
