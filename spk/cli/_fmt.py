from typing import List
from colorama import Fore, Style

import spk


def format_ident(pkg: spk.api.Ident) -> str:

    out = f"{Style.BRIGHT}{pkg.name}{Style.RESET_ALL}"
    if pkg.version.parts or pkg.build is not None:
        out += f"/{Fore.LIGHTBLUE_EX}{pkg.version}{Fore.RESET}"
    if pkg.build is not None:
        out += f"/{Style.DIM}{pkg.build}{Style.RESET_ALL}"
    return out


def format_decision(decision: spk.Decision) -> str:

    if decision.get_error() is not None:
        return f"{Fore.RED}DEAD{Fore.RESET} {decision.get_error()}"
    out = ""
    if decision.get_resolved():
        values = list(format_ident(pkg) for pkg in decision.get_resolved().values())
        out += f"{Fore.GREEN}RESOLVE{Fore.RESET} {', '.join(values)} "
    if decision.get_requests():
        values = list(
            format_request(n, pkgs) for n, pkgs in decision.get_requests().items()
        )
        out += f"{Fore.BLUE}REQUEST{Fore.RESET} {', '.join(values)} "
    if decision.get_resolved():
        out += (
            f"{Fore.RED}UNRESOLVE{Fore.RESET} {', '.join(decision.get_unresolved())} "
        )
    return out


def format_request(name: str, pkgs: List[spk.api.Ident]) -> str:

    out = f"{Style.BRIGHT}{name}{Style.RESET_ALL} / ["
    versions = []
    for pkg in pkgs:
        ver = f"{Fore.LIGHTBLUE_EX}{str(pkg.version) or '*'}{Fore.RESET}"
        if pkg.build is not None:
            ver += f"/{Style.DIM}{pkg.build}{Style.RESET_ALL}"
        versions.append(ver)
    out += ",".join(versions) + "]"
    return out
