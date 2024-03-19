from __future__ import annotations

import argparse
from typing import Callable, ClassVar

from rich.style import StyleType

class RichHelpFormatter(argparse.HelpFormatter):
    """An argparse HelpFormatter class that renders using rich."""

    group_name_formatter: ClassVar[Callable[[str], str]] = ...
    styles: ClassVar[dict[str, StyleType]] = ...
    highlights: ClassVar[list[str]] = ...
    usage_markup: ClassVar[bool] = ...

class RawDescriptionRichHelpFormatter(RichHelpFormatter):
    """Rich help message formatter which retains any formatting in descriptions."""

    ...

class RawTextRichHelpFormatter(RawDescriptionRichHelpFormatter):
    """Rich help message formatter which retains formatting of all help text."""

    ...

class ArgumentDefaultsRichHelpFormatter(
    argparse.ArgumentDefaultsHelpFormatter, RichHelpFormatter
):
    """Rich help message formatter which adds default values to argument help."""

    ...

class MetavarTypeRichHelpFormatter(
    argparse.MetavarTypeHelpFormatter, RichHelpFormatter
):
    """Rich help message formatter which uses the argument 'type' as the default
    metavar value (instead of the argument 'dest').
    """

    ...
