# Security policy

Thank you for helping keep Jun's Garden safe.

## Reporting a problem

Please **do not open a public issue** for a security problem. Instead, use GitHub's private
report: open the **Security** tab of this repository and choose **Report a vulnerability**.
Only the people who look after this project can see it.

Helpful things to include:

- what the problem is and where (a page, a file, a line),
- how to make it happen, step by step,
- what someone could do with it.

You will get an answer as soon as possible. Once it is fixed, the fix is described in the
commit that makes it.

## What is covered

The code in this repository: the Rust site in `rust/`, the files it serves in `site/`, the
`beds` service, the Docker files and the setup scripts.

## How the code is checked

Every change is scanned for leaked secrets (gitleaks) and checked by semgrep, trivy,
osv-scanner, clippy and others (see `.github/workflows/`). Dependabot opens update pull
requests every week.
