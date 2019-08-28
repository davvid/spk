import os
import sys
import argparse

import structlog
from colorama import Fore

import spenv

_logger = structlog.get_logger()


def register(sub_parsers: argparse._SubParsersAction) -> None:

    init_cmd = sub_parsers.add_parser("init-runtime", help=argparse.SUPPRESS)
    init_cmd.add_argument("cmd", nargs=argparse.REMAINDER)
    init_cmd.set_defaults(func=_init)


def _init(args: argparse.Namespace) -> None:
    """This is a hidden command.

    This command is the entry point to new environments, and
    is executed ahead of any desired process to setup the
    environment variables and other configuration that can
    only be done from within the mount namespace.
    """

    print(f"Initializing spenv runtime...", end="", file=sys.stderr, flush=True)
    # TODO: setup the environment
    print(f"{Fore.GREEN}OK{Fore.RESET}", file=sys.stderr)
    os.execv(args.cmd[0], args.cmd)
