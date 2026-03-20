"""Web search helper for DTC code lookup."""

import webbrowser
from config import DTC_SEARCH_URL


def search_dtc(dtc_code: str, vehicle: str = "") -> None:
    """Open browser with DTC search query.

    Args:
        dtc_code: DTC code to search for
        vehicle: Optional vehicle model for more specific results
    """
    url = DTC_SEARCH_URL.format(code=dtc_code, vehicle=vehicle)
    webbrowser.open(url)


def search_dtc_custom(dtc_code: str, search_engine: str = "google") -> None:
    """Search DTC on specific search engine.

    Args:
        dtc_code: DTC code to search for
        search_engine: Search engine to use ("google" or "obd-codes")
    """
    engines = {
        "google": "https://www.google.com/search?q={code}+OBD+diagnostic",
        "obd-codes": "https://www.obd-codes.com/{code}",
    }
    url = engines.get(search_engine, engines["google"]).format(code=dtc_code.upper())
    webbrowser.open(url)
