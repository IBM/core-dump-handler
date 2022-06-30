# Contributing

Our project welcomes external contributions. If you have an itch, please feel
free to scratch it.

To contribute minor code or documentation, please submit a [pull request](https://github.com/ibm/core-dump-handler/pulls).

A good way to familiarize yourself with the codebase and contribution process is
to look for and tackle low-hanging fruit in the [issue tracker](https://github.com/IBM/core-dump-handler/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22).
Before embarking on a more ambitious contribution, please quickly [get in touch](#communication) with us.

**Note: We appreciate your effort, and want to avoid a situation where a contribution
requires extensive rework (by you or by us), sits in backlog for a long time, or
cannot be accepted at all!**

## Setup

This project is based on Rust - The easiest way to get setup is to us the [rustup install system](https://rustup.rs/).

To perform an end to end integration test you may want to use a [free cluster](https://cloud.ibm.com/docs/containers?topic=containers-getting-started#clusters_gs) on IBM Cloud.

Instructions on how to install are available in the main [README.md](https://github.com/IBM/core-dump-handler#installing-the-chart)

## Testing
PR's that modify the codebase will be expected to run against a cluster using the `integration/run.sh` before being accepted.

## Coding style guidelines
Code contributions should be PR'd with `cargo fmt` ran


## Developer Certificate of Origin

This project used the [Developer Certificate of Origin](https://developercertificate.org/). It requires all commit messages to contain the Signed-off-by line with an email address that matches the commit author.

```
Developer Certificate of Origin
Version 1.1

Copyright (C) 2004, 2006 The Linux Foundation and its contributors.

Everyone is permitted to copy and distribute verbatim copies of this
license document, but changing it is not allowed.


Developer's Certificate of Origin 1.1

By making a contribution to this project, I certify that:

(a) The contribution was created in whole or in part by me and I
    have the right to submit it under the open source license
    indicated in the file; or

(b) The contribution is based upon previous work that, to the best
    of my knowledge, is covered under an appropriate open source
    license and I have the right under that license to submit that
    work with modifications, whether created in whole or in part
    by me, under the same open source license (unless I am
    permitted to submit under a different license), as indicated
    in the file; or

(c) The contribution was provided directly to me by some other
    person who certified (a), (b) or (c) and I have not modified
    it.

(d) I understand and agree that this project and the contribution
    are public and that a record of the contribution (including all
    personal information I submit with it, including my sign-off) is
    maintained indefinitely and may be redistributed consistent with
    this project or the open source license(s) involved.
```

In order to comply with the DCO please sign every commit as follows:
```
$ git commit -s -m 'This is my commit message'
```
or

```
This is my commit message

Signed-off-by: Random J Developer <random@developer.example.org>
```

## Releases

Declaring formal releases remains the prerogative of the project maintainer(s).

### Proposing new features

#### First time contributors 

If you would like to implement a new feature, please [raise an issue](https://github.com/ibm/core-dump-handler/issues)
before sending a pull request so the feature can be discussed. This is to avoid
you wasting your valuable time working on a feature that the project developers
are not interested in accepting into the code base.

#### On-boarded collaborators

1. **No `--force` pushes** or modifying the Git history in any way.
1. **Non-main branches** ought to be used for ongoing work.
1. **External API changes and significant modifications** ought to be subject to an **internal pull-request** to solicit feedback from other contributors.
1. Internal pull-requests to solicit feedback are *encouraged* for any other non-trivial contribution but left to the discretion of the contributor.

### Fixing bugs

If you would like to fix a bug, please feel free to open a [PR directly for a small change](https://github.com/ibm/core-dump-handler/pulls).
If you think the fix will be high impact then consider [opening an issue](https://github.com/ibm/core-dump-handler/issues) before sending a
pull request so it can be tracked.

### Merge approval

For first time PRs the project maintainers use LGTM (Looks Good To Me) in comments on the code
review to indicate acceptance.

For a list of the maintainers, see the [CONTRIBUTORS.md](CONTRIBUTORS.md) page.


# Legal conditions

- Any contributions (code, information etc) submitted will be subject to the same [license](LICENSE) as the rest of the code.
No new restrictions/conditions are permitted.
- As a contributor, you MUST have the legal right to grant permission for your contribution to be used under these conditions.

## Communication
Please use the [issue list] to keep communication transparent (https://github.com/ibm/core-dump-handler/issues)
