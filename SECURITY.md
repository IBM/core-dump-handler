# Security Policy 

## Security Announcements
Watch this project for issues raised about security and major API announcements.

## Report a Vulnerability
We're extremely grateful for security researchers and users that report vulnerabilities to the core-dump-handler Community. All reports are thoroughly investigated by community volunteers.

To make a report email the `awhalley@ie.ibm.com` with the security details.

You may encrypt your email using the keys of the [core developers](https://keybase.io/antonwhalley). Encryption is NOT required to make a disclosure.

### When Should I Report a Vulnerability?

You think you discovered a potential security vulnerability in core-dump-handler
You are unsure how a vulnerability affects core-dump-handler
You think you discovered a vulnerability in another project that core-dump-handler depends on
For projects with their own vulnerability reporting and disclosure process, please report it directly there

### When Should I NOT Report a Vulnerability?
You need help tuning core-dump-handler components for security
You need help applying security related updates
Your issue is not security related

### Security Vulnerability Response
Each report is acknowledged and analyzed within 3 working days.

Any vulnerability information shared will stay within the project and will not be disseminated to other projects unless it is necessary to get the issue fixed.

As the security issue moves from triage, to identified fix, to release planning we will keep the reporter updated.

### Public Disclosure Timing
A public disclosure date is negotiated with the bug submitter. We prefer to fully disclose the bug as soon as possible once a user mitigation is available. It is reasonable to delay disclosure when the bug or the fix is not yet fully understood, the solution is not well-tested, or for vendor coordination. The timeframe for disclosure is from immediate (especially if it's already publicly known) to a few weeks. For a vulnerability with a straightforward mitigation, we expect report date to disclosure date to be on the order of 7 days. The Kubernetes Security Response Committee holds the final say when setting a disclosure date.

## Supported Versions

Versions are expressed as x.y.z, where x is the major version, y is the minor version, and z is the patch version, following [Semantic Versioning](https://semver.org/) terminology.

The project maintains release branches. Currently we only support the most recent release and fixes will be applied to the next release and **will not** be back ported. We target at least one release every 3 months but this is *not* a guaranteed candence.