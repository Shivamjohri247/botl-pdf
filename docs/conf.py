"""Sphinx configuration for botl-pdf documentation."""

import os
import sys

# Add the python package to path
sys.path.insert(0, os.path.abspath("../python"))

# -- Project information -----------------------------------------------------

project = "botl-pdf"
copyright = "2024, botl-pdf contributors"
author = "botl-pdf contributors"

release = "0.1.0"
version = "0.1.0"

# -- General configuration ---------------------------------------------------

extensions = [
    "sphinx.ext.autodoc",
    "sphinx.ext.napoleon",
    "sphinx.ext.viewcode",
    "myst_parser",
]

templates_path = ["_templates"]
exclude_patterns = ["_build", "Thumbs.db", ".DS_Store"]

source_suffix = {
    ".rst": "restructuredtext",
    ".md": "markdown",
}

# -- Options for HTML output -------------------------------------------------

html_theme = "furo"
html_static_path = ["_static"]

# -- Options for autodoc -----------------------------------------------------

autodoc_typehints = "description"
autodoc_member_order = "bysource"
