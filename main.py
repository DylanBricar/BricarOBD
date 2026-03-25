"""OBD Diagnostic Pro - Entry point.

A professional OBD-II diagnostic tool with UDS support.
Connects to vehicles via ELM327 USB/Bluetooth adapters.

Safety: Read-only by default. Write operations (DTC clearing) require
double confirmation. Dangerous UDS services are blocked.
"""

import sys
import logging
from pathlib import Path

# Ensure project root is in path
sys.path.insert(0, str(Path(__file__).parent))

from config import APP_NAME, APP_VERSION
from utils.logger import setup_logging

# Core modules
from obd_core.safety import SafetyGuard
from obd_core.obd_reader import OBDReader
from obd_core.uds_client import UDSClient
from obd_core.dtc_manager import DTCManager
from obd_core.anomaly_detector import AnomalyDetector
from obd_core.advanced_manager import AdvancedManager

# GUI
from gui.app import OBDApp
from gui.connection_frame import ConnectionFrame
from gui.dashboard_frame import DashboardFrame
from gui.live_data_frame import LiveDataFrame
from gui.dtc_frame import DTCFrame
from gui.ecu_info_frame import ECUInfoFrame
from gui.history_frame import HistoryFrame

logger = logging.getLogger(__name__)


def main():
    """Initialize and run the OBD Diagnostic application."""
    setup_logging()

    # Install dev console logging handler (captures everything for debug)
    from utils.dev_console import setup_dev_logging
    setup_dev_logging()

    logger.info(f"Starting {APP_NAME} v{APP_VERSION}")

    # Check for demo mode
    demo_mode = "--demo" in sys.argv

    # Initialize core components
    if demo_mode:
        from obd_core.demo_mode import DemoConnection
        connection = DemoConnection()
        logger.info("Running in DEMO mode (simulated vehicle data)")
    else:
        from obd_core.hybrid_reader import HybridConnection
        connection = HybridConnection()
        logger.info("Using hybrid connection (python-obd + custom UDS + safety guards)")
    safety = SafetyGuard()
    obd_reader = OBDReader(connection)
    uds_client = UDSClient(connection, safety)
    dtc_manager = DTCManager(obd_reader, uds_client, safety)
    anomaly_detector = AnomalyDetector()
    advanced_manager = AdvancedManager(connection, safety)

    # Create application
    app = OBDApp()

    # Store references on app for frames to access
    app.connection = connection
    app.obd_reader = obd_reader
    app.uds_client = uds_client
    app.dtc_manager = dtc_manager
    app.safety = safety
    app.anomaly_detector = anomaly_detector
    app.advanced_manager = advanced_manager

    # Create and register frames
    connection_frame = ConnectionFrame(app.content_area, app)
    app.register_frame("Connection", connection_frame)

    dashboard_frame = DashboardFrame(app.content_area, app)
    app.register_frame("Dashboard", dashboard_frame)

    live_data_frame = LiveDataFrame(app.content_area, app)
    app.register_frame("Live Data", live_data_frame)

    dtc_frame = DTCFrame(app.content_area, app)
    app.register_frame("DTC Codes", dtc_frame)

    ecu_info_frame = ECUInfoFrame(app.content_area, app)
    app.register_frame("ECU Info", ecu_info_frame)

    from gui.monitors_frame import MonitorsFrame
    monitors_frame = MonitorsFrame(app.content_area, app)
    app.register_frame("Monitors", monitors_frame)

    history_frame = HistoryFrame(app.content_area, app)
    app.register_frame("History", history_frame)

    from gui.advanced_frame import AdvancedFrame
    advanced_frame = AdvancedFrame(app.content_area, app)
    app.register_frame("Advanced", advanced_frame)

    from gui.database_frame import DatabaseFrame
    from obd_core.database_reader import ECUDatabase
    ecu_database = ECUDatabase()
    ecu_database.load()  # Load 4866 ECU definitions from ZIP
    app.ecu_database = ecu_database

    # Pre-load EV parser (cached on app for reuse)
    from obd_core.ev_data_parser import EVDataParser
    ev_parser = EVDataParser()
    ev_parser.load()
    app.ev_parser = ev_parser
    database_frame = DatabaseFrame(app.content_area, app)
    app.register_frame("Database", database_frame)

    # Show connection frame by default
    app.show_frame("Connection")

    # Handle window close
    def on_closing():
        logger.info("Shutting down...")
        # Stop any active monitoring and CSV recording
        try:
            if hasattr(dashboard_frame, 'stop_monitoring'):
                dashboard_frame.stop_monitoring()
            if hasattr(dashboard_frame, '_stop_csv') and dashboard_frame._csv_recording:
                dashboard_frame._stop_csv()
            if hasattr(live_data_frame, 'stop_monitoring'):
                live_data_frame.stop_monitoring()
            if hasattr(live_data_frame, '_stop_csv') and live_data_frame._csv_recording:
                live_data_frame._stop_csv()
            if hasattr(database_frame, 'stop_monitoring'):
                database_frame.stop_monitoring()
        except Exception:
            pass

        # Disconnect from vehicle
        try:
            if connection.is_connected():
                connection.disconnect()
        except Exception:
            pass

        # Close database
        if hasattr(ecu_database, 'close'):
            ecu_database.close()

        app.destroy()

    app.protocol("WM_DELETE_WINDOW", on_closing)

    logger.info("Application ready")
    app.mainloop()


if __name__ == "__main__":
    main()
