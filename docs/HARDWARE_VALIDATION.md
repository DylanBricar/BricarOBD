# OBD hardware validation

The automated replay corpus covers ELM327 framing, noisy adapter output, generic PID parsing, multi-ECU isolation, VIN reassembly, DTC parsing, Mode 06, freeze frames, and manufacturer injector DIDs. It is deterministic and runs in `cargo test` without a vehicle.

Physical validation is still required before claiming compatibility with a specific adapter or vehicle. Record the adapter model and firmware, transport, vehicle, protocol, and observed ECU count for every run.

## Minimum bench matrix

| Transport | Adapter class | Required proof |
| --- | --- | --- |
| USB serial | genuine ELM327 or STN, plus one common clone | connection, voltage, VIN, all supported PID ranges, reconnect |
| Wi-Fi TCP | ELM327 hotspot adapter | endpoint scan, connection, VIN, sustained live data, reconnect |
| BLE | BLE ELM327 or OBDLink | permission flow, scan, connection, VIN, sustained live data, reconnect |
| CAN variants | 11-bit and 29-bit vehicles when available | correct protocol, distinct ECU responses, no frame mixing |

## Vehicle acceptance checklist

1. Read VIN and identify make/model without manual override.
2. Scan all responding ECUs and save the ECU count and addresses.
3. Discover generic PIDs through every advertised bitmap range.
4. Read active, pending, permanent, and mirror DTCs; confirm each ECU frame remains distinct.
5. Read Mode 06 and freeze-frame data when the vehicle advertises them.
6. For each manufacturer injector DID, record every returned cylinder value and every explicit negative response. A missing response must remain visible as unsupported or failed, never silently disappear.
7. Exercise live data for at least 15 minutes while watching CPU use, memory, disconnect recovery, and duplicate requests.
8. Clear DTCs only on a bench vehicle with explicit authorization, then re-read all statuses to prove the result.

Do not interpret a successful replay as physical compatibility. Keep the replay result, adapter-on-bench result, and real-vehicle result as separate evidence levels.
