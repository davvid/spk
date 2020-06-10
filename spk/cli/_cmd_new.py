from typing import Any
import argparse
import os
import textwrap

import structlog

import spfs
import spk

from spk.io import format_decision

_LOGGER = structlog.get_logger("cli")


def register(
    sub_parsers: argparse._SubParsersAction, **parser_args: Any
) -> argparse.ArgumentParser:

    new_cmd = sub_parsers.add_parser("new", help=_new.__doc__, **parser_args)
    new_cmd.add_argument(
        "name", metavar="NAME", nargs=1, help="The name of the new package"
    )
    new_cmd.set_defaults(func=_new)
    return new_cmd


def _new(args: argparse.Namespace) -> None:
    """Generate a new package spec file."""

    name = args.name[0]

    spec = f"""\
        pkg: {name}/0.1.0

        build:
          # variants declares the default set of variants to build and publish
          # using the spk build and make-* commands
          variants:
            - {{maya: 2020}}
            - {{maya: 2021}}
          # the build script is arbitrary bash script to be executed for the
          # build. It should be and install artifacts into /spfs
          script:
            # if you remove this it will try to run a build.sh script instead
            - echo "don't forget to add build logic!"
            - exit 1

        # opts defines the set of build options
        opts:
          # var options define environment/string values that affect the
          # package build process. The value is defined in the build environment
          # as SPK_OPT_{{name}}
          - var: arch    # rebuild if the arch changes
          - var: os      # rebuild if the os changes
          - var: centos  # rebuild if centos version changes
          # declaring options prefixed by this pacakges name signals
          # to others that they are not global build settings for any package
          # - var: {name}_debug # toggle a debug build of this package

          # pkg options request packages that need to be present
          # in the build environment. You can specify a version number
          # here as the default for when the option is not otherise specified
          - pkg: maya

        depends:
          # pkg dependencies request packages that need to be present at runtime
          - pkg: maya
        """
    # TODO: talk about pinning build env packages once supported
    spec = textwrap.dedent(spec)

    spec_file = f"{name}.yaml"
    with open(spec_file, "x") as writer:
        writer.write(spec)
    print("created:", spec_file)