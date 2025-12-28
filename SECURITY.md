# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability, please report it by emailing the maintainer directly rather than opening a public issue.

**Please include:**

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Any suggested fixes (optional)

You can expect an initial response within 48 hours. We will work with you to understand and address the issue promptly.

## Security Considerations

This crate includes FFI bindings to UDPipe (C++ code). While we take care to ensure safe usage:

- All FFI calls are wrapped in safe Rust APIs
- Input validation is performed before passing data to native code
- Memory is managed through Rust's ownership system

For vulnerabilities in the underlying UDPipe library itself, please also report to the [UDPipe project](https://github.com/ufal/udpipe).
