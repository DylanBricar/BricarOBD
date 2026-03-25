from dataclasses import dataclass
from typing import Callable, Dict, Tuple, Optional


@dataclass
class PIDDefinition:
    pid: int
    name: str
    description: str
    formula: Callable[[bytes], float]
    unit: str
    min_val: float
    max_val: float
    num_bytes: int


def _engine_load_formula(data: bytes) -> float:
    return (data[0] * 100) / 255


def _coolant_temp_formula(data: bytes) -> float:
    return data[0] - 40


def _short_term_fuel_trim_formula(data: bytes) -> float:
    return ((data[0] - 128) * 100) / 128


def _long_term_fuel_trim_formula(data: bytes) -> float:
    return ((data[0] - 128) * 100) / 128


def _intake_manifold_pressure_formula(data: bytes) -> float:
    return float(data[0])


def _engine_rpm_formula(data: bytes) -> float:
    return ((data[0] * 256 + data[1]) / 4)


def _vehicle_speed_formula(data: bytes) -> float:
    return float(data[0])


def _timing_advance_formula(data: bytes) -> float:
    return (data[0] / 2) - 64


def _intake_air_temp_formula(data: bytes) -> float:
    return data[0] - 40


def _maf_rate_formula(data: bytes) -> float:
    return ((data[0] * 256 + data[1]) / 100)


def _throttle_position_formula(data: bytes) -> float:
    return (data[0] * 100) / 255


def _obd_standard_formula(data: bytes) -> float:
    return float(data[0])


def _run_time_formula(data: bytes) -> float:
    return float(data[0] * 256 + data[1])


def _distance_with_mil_formula(data: bytes) -> float:
    return float(data[0] * 256 + data[1])


def _fuel_level_formula(data: bytes) -> float:
    return (data[0] * 100) / 255


def _distance_since_clear_formula(data: bytes) -> float:
    return float(data[0] * 256 + data[1])


def _barometric_pressure_formula(data: bytes) -> float:
    return float(data[0])


def _control_module_voltage_formula(data: bytes) -> float:
    return ((data[0] * 256 + data[1]) / 1000)


def _ambient_air_temp_formula(data: bytes) -> float:
    return data[0] - 40


def _engine_oil_temp_formula(data: bytes) -> float:
    return data[0] - 40


def _fuel_system_status_formula(data: bytes) -> float:
    return float(data[0])


def _fuel_pressure_formula(data: bytes) -> float:
    return float(data[0] * 3)


def _o2_sensor_voltage_formula(data: bytes) -> float:
    return float(data[0]) / 200


def _commanded_secondary_air_formula(data: bytes) -> float:
    return float(data[0])


def _o2_sensors_present_formula(data: bytes) -> float:
    return float(data[0])


def _auxiliary_input_status_formula(data: bytes) -> float:
    return float(data[0])


def _fuel_rail_pressure_rel_formula(data: bytes) -> float:
    return ((data[0] * 256) + data[1]) * 0.079


def _fuel_rail_pressure_diesel_formula(data: bytes) -> float:
    return ((data[0] * 256) + data[1]) * 10


def _o2_sensor_lambda_formula(data: bytes) -> float:
    return ((data[0] * 256) + data[1]) * 2 / 65536


def _commanded_egr_formula(data: bytes) -> float:
    return (data[0] * 100) / 255


def _egr_error_formula(data: bytes) -> float:
    return ((data[0] - 128) * 100) / 128


def _commanded_evap_purge_formula(data: bytes) -> float:
    return (data[0] * 100) / 255


def _warm_ups_since_clear_formula(data: bytes) -> float:
    return float(data[0])


def _evap_vapor_pressure_formula(data: bytes) -> float:
    return ((data[0] * 256) + data[1]) / 4


def _o2_sensor_current_formula(data: bytes) -> float:
    return ((data[2] * 256) + data[3]) / 256 - 128


def _catalyst_temp_formula(data: bytes) -> float:
    return ((data[0] * 256) + data[1]) / 10 - 40


def _absolute_load_value_formula(data: bytes) -> float:
    return ((data[0] * 256) + data[1]) * 100 / 255


def _commanded_equiv_ratio_formula(data: bytes) -> float:
    return ((data[0] * 256) + data[1]) * 2 / 65536


def _relative_throttle_pos_formula(data: bytes) -> float:
    return (data[0] * 100) / 255


def _absolute_throttle_b_formula(data: bytes) -> float:
    return (data[0] * 100) / 255


def _absolute_throttle_c_formula(data: bytes) -> float:
    return (data[0] * 100) / 255


def _accel_pedal_pos_d_formula(data: bytes) -> float:
    return (data[0] * 100) / 255


def _accel_pedal_pos_e_formula(data: bytes) -> float:
    return (data[0] * 100) / 255


def _accel_pedal_pos_f_formula(data: bytes) -> float:
    return (data[0] * 100) / 255


def _commanded_throttle_formula(data: bytes) -> float:
    return (data[0] * 100) / 255


def _time_with_mil_on_formula(data: bytes) -> float:
    return float(data[0] * 256 + data[1])


def _time_since_clear_formula(data: bytes) -> float:
    return float(data[0] * 256 + data[1])


def _fuel_type_formula(data: bytes) -> float:
    return float(data[0])


def _ethanol_fuel_pct_formula(data: bytes) -> float:
    return (data[0] * 100) / 255


def _relative_accel_pos_formula(data: bytes) -> float:
    return (data[0] * 100) / 255


def _hybrid_battery_life_formula(data: bytes) -> float:
    return (data[0] * 100) / 255


def _fuel_injection_timing_formula(data: bytes) -> float:
    return (((data[0] * 256) + data[1]) - 26880) / 128


def _engine_fuel_rate_formula(data: bytes) -> float:
    return ((data[0] * 256) + data[1]) * 0.05


def _driver_torque_demand_formula(data: bytes) -> float:
    return float(data[0] - 125)


def _actual_engine_torque_formula(data: bytes) -> float:
    return float(data[0] - 125)


def _engine_ref_torque_formula(data: bytes) -> float:
    return float(data[0] * 256 + data[1])


def _engine_coolant_temp_2_formula(data: bytes) -> float:
    return float(data[1] - 40)


def _intake_air_temp_2_formula(data: bytes) -> float:
    return float(data[1] - 40)


STANDARD_PIDS: Dict[int, PIDDefinition] = {
    0x03: PIDDefinition(
        pid=0x03,
        name="Fuel System Status",
        description="Fuel system operating mode",
        formula=_fuel_system_status_formula,
        unit="",
        min_val=0.0,
        max_val=255.0,
        num_bytes=1
    ),
    0x04: PIDDefinition(
        pid=0x04,
        name="Engine Load",
        description="Calculated engine load value",
        formula=_engine_load_formula,
        unit="%",
        min_val=0.0,
        max_val=100.0,
        num_bytes=1
    ),
    0x05: PIDDefinition(
        pid=0x05,
        name="Coolant Temperature",
        description="Engine coolant temperature",
        formula=_coolant_temp_formula,
        unit="°C",
        min_val=-40.0,
        max_val=215.0,
        num_bytes=1
    ),
    0x06: PIDDefinition(
        pid=0x06,
        name="Short Term Fuel Trim Bank 1",
        description="Short term fuel trim bank 1",
        formula=_short_term_fuel_trim_formula,
        unit="%",
        min_val=-100.0,
        max_val=99.2,
        num_bytes=1
    ),
    0x07: PIDDefinition(
        pid=0x07,
        name="Long Term Fuel Trim Bank 1",
        description="Long term fuel trim bank 1",
        formula=_long_term_fuel_trim_formula,
        unit="%",
        min_val=-100.0,
        max_val=99.2,
        num_bytes=1
    ),
    0x08: PIDDefinition(
        pid=0x08,
        name="Short Term Fuel Trim Bank 2",
        description="Short term fuel trim bank 2",
        formula=_short_term_fuel_trim_formula,
        unit="%",
        min_val=-100.0,
        max_val=99.2,
        num_bytes=1
    ),
    0x09: PIDDefinition(
        pid=0x09,
        name="Long Term Fuel Trim Bank 2",
        description="Long term fuel trim bank 2",
        formula=_long_term_fuel_trim_formula,
        unit="%",
        min_val=-100.0,
        max_val=99.2,
        num_bytes=1
    ),
    0x0A: PIDDefinition(
        pid=0x0A,
        name="Fuel Pressure",
        description="Fuel pressure gauge",
        formula=_fuel_pressure_formula,
        unit="kPa",
        min_val=0.0,
        max_val=765.0,
        num_bytes=1
    ),
    0x0B: PIDDefinition(
        pid=0x0B,
        name="Intake Manifold Pressure",
        description="Intake manifold absolute pressure",
        formula=_intake_manifold_pressure_formula,
        unit="kPa",
        min_val=0.0,
        max_val=255.0,
        num_bytes=1
    ),
    0x0C: PIDDefinition(
        pid=0x0C,
        name="Engine RPM",
        description="Engine speed",
        formula=_engine_rpm_formula,
        unit="RPM",
        min_val=0.0,
        max_val=16383.75,
        num_bytes=2
    ),
    0x0D: PIDDefinition(
        pid=0x0D,
        name="Vehicle Speed",
        description="Vehicle speed",
        formula=_vehicle_speed_formula,
        unit="km/h",
        min_val=0.0,
        max_val=255.0,
        num_bytes=1
    ),
    0x0E: PIDDefinition(
        pid=0x0E,
        name="Timing Advance",
        description="Ignition timing advance for #1 cylinder",
        formula=_timing_advance_formula,
        unit="°",
        min_val=-64.0,
        max_val=63.5,
        num_bytes=1
    ),
    0x0F: PIDDefinition(
        pid=0x0F,
        name="Intake Air Temperature",
        description="Intake air temperature",
        formula=_intake_air_temp_formula,
        unit="°C",
        min_val=-40.0,
        max_val=215.0,
        num_bytes=1
    ),
    0x10: PIDDefinition(
        pid=0x10,
        name="MAF Rate",
        description="Mass air flow sensor air flow rate",
        formula=_maf_rate_formula,
        unit="g/s",
        min_val=0.0,
        max_val=655.35,
        num_bytes=2
    ),
    0x11: PIDDefinition(
        pid=0x11,
        name="Throttle Position",
        description="Absolute throttle position",
        formula=_throttle_position_formula,
        unit="%",
        min_val=0.0,
        max_val=100.0,
        num_bytes=1
    ),
    0x12: PIDDefinition(
        pid=0x12,
        name="Commanded Secondary Air",
        description="Commanded secondary air status",
        formula=_commanded_secondary_air_formula,
        unit="",
        min_val=0.0,
        max_val=255.0,
        num_bytes=1
    ),
    0x13: PIDDefinition(
        pid=0x13,
        name="O2 Sensors Present",
        description="O2 sensors present (2 banks)",
        formula=_o2_sensors_present_formula,
        unit="",
        min_val=0.0,
        max_val=255.0,
        num_bytes=1
    ),
    0x14: PIDDefinition(
        pid=0x14,
        name="O2 Sensor B1S1 Voltage",
        description="O2 sensor bank 1 sensor 1",
        formula=_o2_sensor_voltage_formula,
        unit="V",
        min_val=0.0,
        max_val=1.275,
        num_bytes=1
    ),
    0x15: PIDDefinition(
        pid=0x15,
        name="O2 Sensor B1S2 Voltage",
        description="O2 sensor bank 1 sensor 2",
        formula=_o2_sensor_voltage_formula,
        unit="V",
        min_val=0.0,
        max_val=1.275,
        num_bytes=1
    ),
    0x16: PIDDefinition(
        pid=0x16,
        name="O2 Sensor B1S3 Voltage",
        description="O2 sensor bank 1 sensor 3",
        formula=_o2_sensor_voltage_formula,
        unit="V",
        min_val=0.0,
        max_val=1.275,
        num_bytes=1
    ),
    0x17: PIDDefinition(
        pid=0x17,
        name="O2 Sensor B1S4 Voltage",
        description="O2 sensor bank 1 sensor 4",
        formula=_o2_sensor_voltage_formula,
        unit="V",
        min_val=0.0,
        max_val=1.275,
        num_bytes=1
    ),
    0x18: PIDDefinition(
        pid=0x18,
        name="O2 Sensor B2S1 Voltage",
        description="O2 sensor bank 2 sensor 1",
        formula=_o2_sensor_voltage_formula,
        unit="V",
        min_val=0.0,
        max_val=1.275,
        num_bytes=1
    ),
    0x19: PIDDefinition(
        pid=0x19,
        name="O2 Sensor B2S2 Voltage",
        description="O2 sensor bank 2 sensor 2",
        formula=_o2_sensor_voltage_formula,
        unit="V",
        min_val=0.0,
        max_val=1.275,
        num_bytes=1
    ),
    0x1A: PIDDefinition(
        pid=0x1A,
        name="O2 Sensor B2S3 Voltage",
        description="O2 sensor bank 2 sensor 3",
        formula=_o2_sensor_voltage_formula,
        unit="V",
        min_val=0.0,
        max_val=1.275,
        num_bytes=1
    ),
    0x1B: PIDDefinition(
        pid=0x1B,
        name="O2 Sensor B2S4 Voltage",
        description="O2 sensor bank 2 sensor 4",
        formula=_o2_sensor_voltage_formula,
        unit="V",
        min_val=0.0,
        max_val=1.275,
        num_bytes=1
    ),
    0x1C: PIDDefinition(
        pid=0x1C,
        name="OBD Standard",
        description="OBD standard this vehicle conforms to",
        formula=_obd_standard_formula,
        unit="enum",
        min_val=0.0,
        max_val=255.0,
        num_bytes=1
    ),
    0x1E: PIDDefinition(
        pid=0x1E,
        name="Auxiliary Input Status",
        description="PTO status",
        formula=_auxiliary_input_status_formula,
        unit="",
        min_val=0.0,
        max_val=255.0,
        num_bytes=1
    ),
    0x1F: PIDDefinition(
        pid=0x1F,
        name="Run Time Since Start",
        description="Engine run time",
        formula=_run_time_formula,
        unit="s",
        min_val=0.0,
        max_val=65535.0,
        num_bytes=2
    ),
    0x21: PIDDefinition(
        pid=0x21,
        name="Distance with MIL On",
        description="Distance traveled with MIL on",
        formula=_distance_with_mil_formula,
        unit="km",
        min_val=0.0,
        max_val=65535.0,
        num_bytes=2
    ),
    0x22: PIDDefinition(
        pid=0x22,
        name="Fuel Rail Pressure (rel)",
        description="Fuel rail pressure relative to vacuum",
        formula=_fuel_rail_pressure_rel_formula,
        unit="kPa",
        min_val=0.0,
        max_val=5177.27,
        num_bytes=2
    ),
    0x23: PIDDefinition(
        pid=0x23,
        name="Fuel Rail Pressure (diesel)",
        description="Fuel rail gauge pressure (diesel)",
        formula=_fuel_rail_pressure_diesel_formula,
        unit="kPa",
        min_val=0.0,
        max_val=655350.0,
        num_bytes=2
    ),
    0x24: PIDDefinition(
        pid=0x24,
        name="O2 Sensor 1 Lambda",
        description="O2 sensor 1 equivalence ratio",
        formula=_o2_sensor_lambda_formula,
        unit="ratio",
        min_val=0.0,
        max_val=2.0,
        num_bytes=2
    ),
    0x2C: PIDDefinition(
        pid=0x2C,
        name="Commanded EGR",
        description="Commanded EGR valve position",
        formula=_commanded_egr_formula,
        unit="%",
        min_val=0.0,
        max_val=100.0,
        num_bytes=1
    ),
    0x2D: PIDDefinition(
        pid=0x2D,
        name="EGR Error",
        description="EGR error percentage",
        formula=_egr_error_formula,
        unit="%",
        min_val=-100.0,
        max_val=99.2,
        num_bytes=1
    ),
    0x2E: PIDDefinition(
        pid=0x2E,
        name="Commanded EVAP Purge",
        description="Commanded evaporative purge",
        formula=_commanded_evap_purge_formula,
        unit="%",
        min_val=0.0,
        max_val=100.0,
        num_bytes=1
    ),
    0x2F: PIDDefinition(
        pid=0x2F,
        name="Fuel Tank Level",
        description="Fuel tank level input",
        formula=_fuel_level_formula,
        unit="%",
        min_val=0.0,
        max_val=100.0,
        num_bytes=1
    ),
    0x30: PIDDefinition(
        pid=0x30,
        name="Warm-ups Since Clear",
        description="Warm-ups since codes cleared",
        formula=_warm_ups_since_clear_formula,
        unit="",
        min_val=0.0,
        max_val=255.0,
        num_bytes=1
    ),
    0x31: PIDDefinition(
        pid=0x31,
        name="Distance Since Clear",
        description="Distance traveled since codes cleared",
        formula=_distance_since_clear_formula,
        unit="km",
        min_val=0.0,
        max_val=65535.0,
        num_bytes=2
    ),
    0x32: PIDDefinition(
        pid=0x32,
        name="EVAP Vapor Pressure",
        description="Evap system vapor pressure",
        formula=_evap_vapor_pressure_formula,
        unit="Pa",
        min_val=-8192.0,
        max_val=8191.75,
        num_bytes=2
    ),
    0x33: PIDDefinition(
        pid=0x33,
        name="Barometric Pressure",
        description="Absolute barometric pressure",
        formula=_barometric_pressure_formula,
        unit="kPa",
        min_val=0.0,
        max_val=255.0,
        num_bytes=1
    ),
    0x34: PIDDefinition(
        pid=0x34,
        name="O2 Sensor 1 Current",
        description="O2 sensor 1 current",
        formula=_o2_sensor_current_formula,
        unit="mA",
        min_val=-128.0,
        max_val=127.99,
        num_bytes=4
    ),
    0x3C: PIDDefinition(
        pid=0x3C,
        name="Catalyst Temp B1S1",
        description="Catalyst temperature bank 1 sensor 1",
        formula=_catalyst_temp_formula,
        unit="°C",
        min_val=-40.0,
        max_val=6513.5,
        num_bytes=2
    ),
    0x3D: PIDDefinition(
        pid=0x3D,
        name="Catalyst Temp B2S1",
        description="Catalyst temperature bank 2 sensor 1",
        formula=_catalyst_temp_formula,
        unit="°C",
        min_val=-40.0,
        max_val=6513.5,
        num_bytes=2
    ),
    0x3E: PIDDefinition(
        pid=0x3E,
        name="Catalyst Temp B1S2",
        description="Catalyst temperature bank 1 sensor 2",
        formula=_catalyst_temp_formula,
        unit="°C",
        min_val=-40.0,
        max_val=6513.5,
        num_bytes=2
    ),
    0x3F: PIDDefinition(
        pid=0x3F,
        name="Catalyst Temp B2S2",
        description="Catalyst temperature bank 2 sensor 2",
        formula=_catalyst_temp_formula,
        unit="°C",
        min_val=-40.0,
        max_val=6513.5,
        num_bytes=2
    ),
    0x42: PIDDefinition(
        pid=0x42,
        name="Control Module Voltage",
        description="Control module voltage",
        formula=_control_module_voltage_formula,
        unit="V",
        min_val=0.0,
        max_val=65.535,
        num_bytes=2
    ),
    0x43: PIDDefinition(
        pid=0x43,
        name="Absolute Load Value",
        description="Absolute load value",
        formula=_absolute_load_value_formula,
        unit="%",
        min_val=0.0,
        max_val=25700.0,
        num_bytes=2
    ),
    0x44: PIDDefinition(
        pid=0x44,
        name="Commanded Equiv Ratio",
        description="Commanded air-fuel equivalence ratio",
        formula=_commanded_equiv_ratio_formula,
        unit="ratio",
        min_val=0.0,
        max_val=2.0,
        num_bytes=2
    ),
    0x45: PIDDefinition(
        pid=0x45,
        name="Relative Throttle Pos",
        description="Relative throttle position",
        formula=_relative_throttle_pos_formula,
        unit="%",
        min_val=0.0,
        max_val=100.0,
        num_bytes=1
    ),
    0x46: PIDDefinition(
        pid=0x46,
        name="Ambient Air Temperature",
        description="Ambient air temperature",
        formula=_ambient_air_temp_formula,
        unit="°C",
        min_val=-40.0,
        max_val=215.0,
        num_bytes=1
    ),
    0x47: PIDDefinition(
        pid=0x47,
        name="Absolute Throttle B",
        description="Absolute throttle position B",
        formula=_absolute_throttle_b_formula,
        unit="%",
        min_val=0.0,
        max_val=100.0,
        num_bytes=1
    ),
    0x48: PIDDefinition(
        pid=0x48,
        name="Absolute Throttle C",
        description="Absolute throttle position C",
        formula=_absolute_throttle_c_formula,
        unit="%",
        min_val=0.0,
        max_val=100.0,
        num_bytes=1
    ),
    0x49: PIDDefinition(
        pid=0x49,
        name="Accel Pedal Pos D",
        description="Accelerator pedal position D",
        formula=_accel_pedal_pos_d_formula,
        unit="%",
        min_val=0.0,
        max_val=100.0,
        num_bytes=1
    ),
    0x4A: PIDDefinition(
        pid=0x4A,
        name="Accel Pedal Pos E",
        description="Accelerator pedal position E",
        formula=_accel_pedal_pos_e_formula,
        unit="%",
        min_val=0.0,
        max_val=100.0,
        num_bytes=1
    ),
    0x4B: PIDDefinition(
        pid=0x4B,
        name="Accel Pedal Pos F",
        description="Accelerator pedal position F",
        formula=_accel_pedal_pos_f_formula,
        unit="%",
        min_val=0.0,
        max_val=100.0,
        num_bytes=1
    ),
    0x4C: PIDDefinition(
        pid=0x4C,
        name="Commanded Throttle",
        description="Commanded throttle actuator",
        formula=_commanded_throttle_formula,
        unit="%",
        min_val=0.0,
        max_val=100.0,
        num_bytes=1
    ),
    0x4D: PIDDefinition(
        pid=0x4D,
        name="Time with MIL On",
        description="Time run with MIL on",
        formula=_time_with_mil_on_formula,
        unit="min",
        min_val=0.0,
        max_val=65535.0,
        num_bytes=2
    ),
    0x4E: PIDDefinition(
        pid=0x4E,
        name="Time Since Clear",
        description="Time since trouble codes cleared",
        formula=_time_since_clear_formula,
        unit="min",
        min_val=0.0,
        max_val=65535.0,
        num_bytes=2
    ),
    0x51: PIDDefinition(
        pid=0x51,
        name="Fuel Type",
        description="Fuel type (gasoline, diesel, etc.)",
        formula=_fuel_type_formula,
        unit="",
        min_val=0.0,
        max_val=255.0,
        num_bytes=1
    ),
    0x52: PIDDefinition(
        pid=0x52,
        name="Ethanol Fuel %",
        description="Ethanol fuel percentage",
        formula=_ethanol_fuel_pct_formula,
        unit="%",
        min_val=0.0,
        max_val=100.0,
        num_bytes=1
    ),
    0x5A: PIDDefinition(
        pid=0x5A,
        name="Relative Accel Pos",
        description="Relative accelerator pedal position",
        formula=_relative_accel_pos_formula,
        unit="%",
        min_val=0.0,
        max_val=100.0,
        num_bytes=1
    ),
    0x5B: PIDDefinition(
        pid=0x5B,
        name="Hybrid Battery Life",
        description="Hybrid battery pack remaining life",
        formula=_hybrid_battery_life_formula,
        unit="%",
        min_val=0.0,
        max_val=100.0,
        num_bytes=1
    ),
    0x5C: PIDDefinition(
        pid=0x5C,
        name="Engine Oil Temperature",
        description="Engine oil temperature",
        formula=_engine_oil_temp_formula,
        unit="°C",
        min_val=-40.0,
        max_val=210.0,
        num_bytes=1
    ),
    0x5D: PIDDefinition(
        pid=0x5D,
        name="Fuel Injection Timing",
        description="Fuel injection timing",
        formula=_fuel_injection_timing_formula,
        unit="°",
        min_val=-210.0,
        max_val=301.99,
        num_bytes=2
    ),
    0x5E: PIDDefinition(
        pid=0x5E,
        name="Engine Fuel Rate",
        description="Engine fuel rate",
        formula=_engine_fuel_rate_formula,
        unit="L/h",
        min_val=0.0,
        max_val=3212.75,
        num_bytes=2
    ),
    0x61: PIDDefinition(
        pid=0x61,
        name="Driver Torque Demand",
        description="Driver's demand engine torque",
        formula=_driver_torque_demand_formula,
        unit="%",
        min_val=-125.0,
        max_val=130.0,
        num_bytes=1
    ),
    0x62: PIDDefinition(
        pid=0x62,
        name="Actual Engine Torque",
        description="Actual engine torque",
        formula=_actual_engine_torque_formula,
        unit="%",
        min_val=-125.0,
        max_val=130.0,
        num_bytes=1
    ),
    0x63: PIDDefinition(
        pid=0x63,
        name="Engine Ref Torque",
        description="Engine reference torque",
        formula=_engine_ref_torque_formula,
        unit="Nm",
        min_val=0.0,
        max_val=65535.0,
        num_bytes=2
    ),
    0x67: PIDDefinition(
        pid=0x67,
        name="Engine Coolant Temp 2",
        description="Engine coolant temperature sensors",
        formula=_engine_coolant_temp_2_formula,
        unit="°C",
        min_val=-40.0,
        max_val=215.0,
        num_bytes=2
    ),
    0x68: PIDDefinition(
        pid=0x68,
        name="Intake Air Temp 2",
        description="Intake air temperature sensor",
        formula=_intake_air_temp_2_formula,
        unit="°C",
        min_val=-40.0,
        max_val=215.0,
        num_bytes=2
    ),
}


def get_pid(pid_code: int) -> Optional[PIDDefinition]:
    """Get a PID definition by its code."""
    return STANDARD_PIDS.get(pid_code)


def decode_pid(pid_code: int, data_bytes: bytes) -> Optional[Tuple[float, str]]:
    """Decode a PID value from raw bytes."""
    pid_def = get_pid(pid_code)
    if pid_def is None:
        return None

    if len(data_bytes) < pid_def.num_bytes:
        return None

    value = pid_def.formula(data_bytes)
    return (value, pid_def.unit)


# French translations for PID names
PID_NAMES_FR = {
    0x03: "État système carburant",
    0x04: "Charge moteur",
    0x05: "Température liquide refroidissement",
    0x06: "Correction carburant court terme B1",
    0x07: "Correction carburant long terme B1",
    0x08: "Correction carburant court terme B2",
    0x09: "Correction carburant long terme B2",
    0x0A: "Pression carburant",
    0x0B: "Pression collecteur admission",
    0x0C: "Régime moteur",
    0x0D: "Vitesse véhicule",
    0x0E: "Avance à l'allumage",
    0x0F: "Température air admission",
    0x10: "Débit air (MAF)",
    0x11: "Position papillon",
    0x12: "Air secondaire commandé",
    0x13: "Sondes O2 présentes",
    0x14: "Sonde O2 B1S1 tension",
    0x15: "Sonde O2 B1S2 tension",
    0x16: "Sonde O2 B1S3 tension",
    0x17: "Sonde O2 B1S4 tension",
    0x18: "Sonde O2 B2S1 tension",
    0x19: "Sonde O2 B2S2 tension",
    0x1A: "Sonde O2 B2S3 tension",
    0x1B: "Sonde O2 B2S4 tension",
    0x1C: "Norme OBD",
    0x1E: "Entrée auxiliaire",
    0x1F: "Durée fonctionnement moteur",
    0x21: "Distance avec témoin moteur",
    0x22: "Pression rampe (relative)",
    0x23: "Pression rampe (diesel)",
    0x24: "Lambda sonde O2 S1",
    0x2C: "EGR commandé",
    0x2D: "Erreur EGR",
    0x2E: "Purge EVAP commandée",
    0x2F: "Niveau réservoir carburant",
    0x30: "Mises en chauffe depuis effacement",
    0x31: "Distance depuis effacement",
    0x32: "Pression vapeur EVAP",
    0x33: "Pression barométrique",
    0x34: "Courant sonde O2 S1",
    0x3C: "Température catalyseur B1S1",
    0x3D: "Température catalyseur B2S1",
    0x3E: "Température catalyseur B1S2",
    0x3F: "Température catalyseur B2S2",
    0x42: "Tension module commande",
    0x43: "Charge absolue",
    0x44: "Rapport air/carburant commandé",
    0x45: "Position papillon relative",
    0x46: "Température air ambiant",
    0x47: "Position papillon B",
    0x48: "Position papillon C",
    0x49: "Position pédale D",
    0x4A: "Position pédale E",
    0x4B: "Position pédale F",
    0x4C: "Papillon commandé",
    0x4D: "Durée avec témoin moteur",
    0x4E: "Durée depuis effacement",
    0x51: "Type carburant",
    0x52: "Pourcentage éthanol",
    0x5A: "Position pédale relative",
    0x5B: "Vie batterie hybride",
    0x5C: "Température huile moteur",
    0x5D: "Calage injection",
    0x5E: "Consommation carburant",
    0x61: "Couple demandé conducteur",
    0x62: "Couple moteur réel",
    0x63: "Couple référence moteur",
    0x67: "Température refroidissement 2",
    0x68: "Température air admission 2",
}
