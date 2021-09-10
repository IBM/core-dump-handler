# Contributing

Our project welcomes external contributions. If you have an itch, please feel
free to scratch it.

It should also be noted that **[core-dump-handler](https://github.com/IBM/core-dump-handler/) is an [_OPEN Open Source Projects_](https://openopensource.org/).**

Individuals making significant and valuable contributions are given commit-access to a project to contribute as they see fit. A project is more like an open wiki than a standard guarded open source project.

To contribute minor code or documentation, please submit a [pull request](https://github.com/ibm/core-dump-handler/pulls).

A good way to familiarize yourself with the codebase and contribution process is
to look for and tackle low-hanging fruit in the [issue tracker](https://github.com/IBM/core-dump-handler/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22).
Before embarking on a more ambitious contribution, please quickly [get in touch](#communication) with us.

**Note: We appreciate your effort, and want to avoid a situation where a contribution
requires extensive rework (by you or by us), sits in backlog for a long time, or
cannot be accepted at all!**


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
If you think the fix will be high impact then consider [opening an issue](https://github.com/ibm/repo-template/issues) before sending a
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
Please use the [issue list] to keep communication transparent (https://github.com/ibm/repo-template/issues)

## Setup
The quickest way to get setup is to use a [free cluster](https://cloud.ibm.com/docs/containers?topic=containers-getting-started#clusters_gs) on IBM Cloud so you can test your work. 

Instructions on how to install are available in the main [README.md](https://github.com/IBM/core-dump-handler#installing-the-chart)

## Testing
Currently there are unit tests for the agent and composer projects.
The tests need to be ran as `root` on a Linux machine and will modify system settings.
PR's that modify the codebase will be expected to run against a cluster before being accepted.

## Coding style guidelines
Code contributions should be PR'd with `cargo fmt` ran 

**[core-dump-handler](https://github.com/IBM/core-dump-handler/) is an [_OPEN Open Source Projects_](https://openopensource.org/).**

Individuals making significant and valuable contributions are given commit-access to a project to contribute as they see fit. A project is more like an open wiki than a standard guarded open source project.