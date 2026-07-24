"""``vedaksha`` command-line interface.

A thin wrapper over :class:`vedaksha.Vedaksha`, mostly useful for confirming an
install works and for quick one-off computations. All output is JSON on stdout.
"""
from __future__ import annotations

import argparse
import json
import sys
from typing import Any

from . import __version__
from .client import Vedaksha
from .errors import VedakshaError


def _emit(obj: Any) -> None:
    json.dump(obj, sys.stdout, indent=2, ensure_ascii=False)
    sys.stdout.write("\n")


def _add_location(p: argparse.ArgumentParser) -> None:
    p.add_argument("--julian-day", type=float, required=True, help="Julian Day (TT)")
    p.add_argument("--latitude", type=float, required=True)
    p.add_argument("--longitude", type=float, required=True)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="vedaksha", description=__doc__)
    parser.add_argument("--version", action="version", version=f"vedaksha {__version__}")
    sub = parser.add_subparsers(dest="command", required=True)

    sub.add_parser("tools", help="list the engine's tool catalog")

    p_chart = sub.add_parser("chart", help="compute a natal chart")
    _add_location(p_chart)
    p_chart.add_argument("--house-system")
    p_chart.add_argument("--ayanamsha")

    p_call = sub.add_parser(
        "call",
        help="call any of the 15 tools by name with JSON arguments "
        "(use `vedaksha tools` to see each tool's inputs)",
    )
    p_call.add_argument("name")
    p_call.add_argument("--args", default="{}", help="tool arguments as a JSON object")

    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    vk = Vedaksha()
    try:
        if args.command == "tools":
            _emit(vk.list_tools())
        elif args.command == "chart":
            _emit(
                vk.natal_chart(
                    args.julian_day,
                    args.latitude,
                    args.longitude,
                    house_system=args.house_system,
                    ayanamsha=args.ayanamsha,
                )
            )
        elif args.command == "call":
            _emit(vk.call_tool(args.name, **json.loads(args.args)))
        else:  # pragma: no cover - argparse enforces
            return 2
    except VedakshaError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
